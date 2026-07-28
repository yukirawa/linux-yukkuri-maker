use anyhow::{Context, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use std::sync::{Arc, Mutex};

/// 動画のメタ情報
#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub total_frames: u64,
    pub duration: f64,
    pub codec_name: String,
}

/// GStreamerフレームバッファ
struct FrameBuffer {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

/// GStreamerベースの動画再生エンジン
pub struct GstVideoEngine {
    /// 動画情報
    info: VideoInfo,
    /// GStreamerパイプライン
    pipeline: Option<gst::Pipeline>,
    /// 最新フレームのバッファ（共有）
    latest_frame: Arc<Mutex<Option<FrameBuffer>>>,
    /// 現在の再生位置（秒）
    current_position: Arc<Mutex<f64>>,
    /// ファイルパス
    path: String,
}

impl GstVideoEngine {
    /// 動画ファイルを開き、GStreamerパイプラインを構築
    pub fn open(path: &str) -> Result<Self> {
        gst::init().context("GStreamer の初期化に失敗しました")?;

        let path_owned = path.to_string();

        // まずメタ情報を取得
        let info = Self::probe_video_info(&path_owned)?;

        let latest_frame: Arc<Mutex<Option<FrameBuffer>>> = Arc::new(Mutex::new(None));
        let current_position: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));

        // パイプライン構築
        let pipeline = gst::Pipeline::new();

        let src = gst::ElementFactory::make("uridecodebin")
            .property("uri", format!("file://{}", path_owned))
            .build()
            .context("uridecodebinの作成に失敗")?;

        let convert = gst::ElementFactory::make("videoconvert")
            .build()
            .context("videoconvertの作成に失敗")?;
        let appsink = gst_app::AppSink::builder()
            .caps(
                &gst::Caps::builder("video/x-raw")
                    .field("format", "RGB")
                    .field("width", info.width as i32)
                    .field("height", info.height as i32)
                    .build(),
            )
            .build();

        // appsinkの設定
        appsink.set_drop(true);
        appsink.set_max_buffers(2);

        pipeline.add_many([&src.upcast_ref(), &convert, appsink.upcast_ref()])?;

        // uridecodebinのpad-addedシグナル
        let convert_clone = convert.clone();
        src.connect_pad_added(move |_src, src_pad| {
            let caps = src_pad.current_caps().unwrap();
            let structure = caps.structure(0).unwrap();
            let name = structure.name();
            if name.starts_with("video/") {
                let sink_pad = convert_clone.static_pad("sink").unwrap();
                if sink_pad.is_linked() {
                    return;
                }
                src_pad.link(&sink_pad).ok();
            }
        });

        convert.link(&appsink)?;

        // フレームコールバック設定
        let frame_buffer = latest_frame.clone();
        let pos_tracker = current_position.clone();
        let info_clone = info.clone();
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = match appsink.pull_sample() {
                        Ok(s) => s,
                        Err(_) => return Err(gst::FlowError::Error),
                    };
                    let buffer = match sample.buffer() {
                        Some(b) => b,
                        None => return Err(gst::FlowError::Error),
                    };

                    let map = match buffer.map_readable() {
                        Ok(m) => m,
                        Err(_) => return Err(gst::FlowError::Error),
                    };
                    let data = map.to_vec();

                    // 位置情報を更新
                    let pts = buffer.pts().unwrap_or(gst::ClockTime::ZERO);
                    let pos_secs = pts.nseconds() as f64 / 1_000_000_000.0;
                    if let Ok(mut p) = pos_tracker.lock() {
                        *p = pos_secs;
                    }

                    if let Ok(mut fb) = frame_buffer.lock() {
                        *fb = Some(FrameBuffer {
                            data,
                            width: info_clone.width,
                            height: info_clone.height,
                        });
                    }

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        // バスメッセージ監視 (簡易: watch追加のみ、メッセージ処理はpollで行う)
        let bus = pipeline.bus().unwrap();
        bus.add_signal_watch();

        Ok(Self {
            info,
            pipeline: Some(pipeline),
            latest_frame,
            current_position,
            path: path_owned,
        })
    }

    /// ffprobeを使って動画情報を取得
    fn probe_video_info(path: &str) -> Result<VideoInfo> {
        use std::process::Command;

        let output = Command::new("ffprobe")
            .args([
                "-v", "quiet",
                "-print_format", "json",
                "-show_format",
                "-show_streams",
                path,
            ])
            .output()
            .context("ffprobe を実行できませんでした。ffmpegがインストールされているか確認してください。")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("ffprobeがエラーを返しました: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .context("ffprobeの出力をパースできませんでした")?;

        let streams = json["streams"]
            .as_array()
            .context("ストリーム情報がありません")?;

        let video_stream = streams
            .iter()
            .find(|s| s["codec_type"].as_str() == Some("video"))
            .context("ビデオストリームが見つかりません")?;

        let width = video_stream["width"].as_u64().unwrap_or(640) as u32;
        let height = video_stream["height"].as_u64().unwrap_or(480) as u32;
        let fps = Self::parse_fps(video_stream);
        let codec_name = video_stream["codec_name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let duration = json["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| {
                video_stream["duration"]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .unwrap_or(0.0);

        let total_frames = video_stream["nb_frames"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| (duration * fps).round() as u64);

        Ok(VideoInfo {
            width,
            height,
            fps,
            total_frames,
            duration,
            codec_name,
        })
    }

    fn parse_fps(stream: &serde_json::Value) -> f64 {
        if let Some(fps_str) = stream["r_frame_rate"].as_str() {
            if let Some(result) = Self::parse_fraction(fps_str) {
                return result;
            }
        }
        if let Some(fps_str) = stream["avg_frame_rate"].as_str() {
            if let Some(result) = Self::parse_fraction(fps_str) {
                return result;
            }
        }
        30.0
    }

    fn parse_fraction(s: &str) -> Option<f64> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 2 {
            let num: f64 = parts[0].parse().ok()?;
            let den: f64 = parts[1].parse().ok()?;
            if den != 0.0 {
                return Some(num / den);
            }
        }
        None
    }

    /// 動画情報を取得
    pub fn info(&self) -> &VideoInfo {
        &self.info
    }

    /// 再生を開始
    pub fn play(&self) {
        if let Some(ref pipeline) = self.pipeline {
            let _ = pipeline.set_state(gst::State::Playing);
        }
    }

    /// 一時停止
    pub fn pause(&self) {
        if let Some(ref pipeline) = self.pipeline {
            let _ = pipeline.set_state(gst::State::Paused);
        }
    }

    /// 停止
    pub fn stop(&self) {
        if let Some(ref pipeline) = self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
    }

    /// 指定時間にシーク
    pub fn seek(&self, secs: f64) -> Result<()> {
        if let Some(ref pipeline) = self.pipeline {
            pipeline.set_state(gst::State::Paused)?;
            let pos = gst::ClockTime::from_seconds_f64(secs);
            pipeline.seek_simple(
                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                pos,
            )?;
        }
        Ok(())
    }

    /// 先頭フレームを取得（静止画用、パイプラインをPAUSEDにして1フレーム取得）
    pub fn first_frame(&self) -> Result<(Vec<u8>, u32, u32)> {
        if let Some(ref pipeline) = self.pipeline {
            pipeline.set_state(gst::State::Paused)?;

            for _ in 0..50 {
                std::thread::sleep(std::time::Duration::from_millis(50));
                if let Ok(fb) = self.latest_frame.lock() {
                    if let Some(ref frame) = *fb {
                        return Ok((frame.data.clone(), frame.width, frame.height));
                    }
                }
            }
        }
        anyhow::bail!("先頭フレームを取得できませんでした")
    }

    /// 最新のフレームデータを取得（ノンブロッキング）
    pub fn poll_frame(&self) -> Option<(Vec<u8>, u32, u32)> {
        if let Ok(fb) = self.latest_frame.lock() {
            if let Some(ref frame) = *fb {
                return Some((frame.data.clone(), frame.width, frame.height));
            }
        }
        None
    }

    /// 現在の再生位置を取得（秒）
    pub fn current_position(&self) -> f64 {
        if let Ok(p) = self.current_position.lock() {
            *p
        } else {
            0.0
        }
    }
}

impl Drop for GstVideoEngine {
    fn drop(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
    }
}

/// 後方互換用のエイリアス（古いコードが VideoDecoder を参照しているため）
pub type VideoDecoder = GstVideoEngine;