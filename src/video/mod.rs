use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// 動画のメタ情報
#[derive(Debug, Clone)]
pub struct VideoInfo {
    /// 幅（ピクセル）
    pub width: u32,
    /// 高さ（ピクセル）
    pub height: u32,
    /// フレームレート（fps）
    pub fps: f64,
    /// 総フレーム数
    pub total_frames: u64,
    /// 動画の長さ（秒）
    pub duration: f64,
    /// コーデック名
    pub codec_name: String,
}

/// 動画デコーダー（ffmpeg CLI ラッパー）
pub struct VideoDecoder {
    /// 動画情報
    info: VideoInfo,
    /// 動画ファイルのパス
    path: String,
    /// 前回デコード結果のキャッシュ（同一フレーム連続リクエスト用）
    cached_frame: Option<(u64, Vec<u8>)>,
}

impl VideoDecoder {
    /// 動画ファイルを開き、ffprobeでメタ情報を取得
    pub fn open(path: &str) -> Result<Self> {
        let path = Path::new(path);
        let path_str = path.to_string_lossy().to_string();

        // ffprobe で動画情報を取得
        let info = Self::get_video_info(&path_str)?;

        Ok(Self {
            info,
            path: path_str,
            cached_frame: None,
        })
    }

    /// ffprobe を使って動画情報を取得
    fn get_video_info(path: &str) -> Result<VideoInfo> {
        // ffprobe -v quiet -print_format json -show_format -show_streams <file>
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

        // ビデオストリームを探す
        let streams = json["streams"]
            .as_array()
            .context("ストリーム情報がありません")?;

        let video_stream = streams
            .iter()
            .find(|s| s["codec_type"].as_str() == Some("video"))
            .context("ビデオストリームが見つかりません")?;

        let width = video_stream["width"].as_u64().unwrap_or(640) as u32;
        let height = video_stream["height"].as_u64().unwrap_or(480) as u32;

        // FPSを取得（様々な形式に対応）
        let fps = Self::parse_fps(video_stream);

        // コーデック名
        let codec_name = video_stream["codec_name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        // 動画の長さ（format または stream から）
        let duration = json["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| video_stream["duration"].as_str().and_then(|s| s.parse::<f64>().ok()))
            .unwrap_or(0.0);

        // 総フレーム数
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

    /// FPSをパース（"r_frame_rate" または "avg_frame_rate" から）
    fn parse_fps(stream: &serde_json::Value) -> f64 {
        // r_frame_rate を優先（実際のフレームレート）
        if let Some(fps_str) = stream["r_frame_rate"].as_str() {
            if let Some(result) = Self::parse_fraction(fps_str) {
                return result;
            }
        }

        // avg_frame_rate にフォールバック
        if let Some(fps_str) = stream["avg_frame_rate"].as_str() {
            if let Some(result) = Self::parse_fraction(fps_str) {
                return result;
            }
        }

        30.0 // デフォルト
    }

    /// "30000/1001" 形式の分数をパース
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

    /// 指定フレーム番号のフレームをRGB8バイト列として取得
    /// ffmpeg CLI を使って1フレームを抽出し、PPM形式で受け取ってRGBに変換
    /// 戻り値: (RGBデータ, 幅, 高さ)
    pub fn seek_frame(&mut self, frame_idx: u64) -> Result<(Vec<u8>, u32, u32)> {
        // キャッシュヒットチェック
        if let Some((cached_idx, cached_data)) = &self.cached_frame {
            if *cached_idx == frame_idx {
                return Ok((
                    cached_data.clone(),
                    self.info.width,
                    self.info.height,
                ));
            }
        }

        let time_secs = if self.info.fps > 0.0 {
            frame_idx as f64 / self.info.fps
        } else {
            frame_idx as f64 / 30.0
        };

        self.extract_frame_at_time(time_secs)
    }

    /// 指定時間（秒）のフレームをRGB8バイト列として取得
    pub fn seek_time(&mut self, secs: f64) -> Result<(Vec<u8>, u32, u32)> {
        self.extract_frame_at_time(secs)
    }

    /// 先頭フレームを取得
    pub fn first_frame(&mut self) -> Result<(Vec<u8>, u32, u32)> {
        self.extract_frame_at_time(0.0)
    }

    /// ffmpeg CLI で特定時間のフレームを抽出
    fn extract_frame_at_time(&mut self, time_secs: f64) -> Result<(Vec<u8>, u32, u32)> {
        let width = self.info.width;
        let height = self.info.height;

        // ffmpeg -ss <time> -i <file> -vframes 1 -f rawvideo -pix_fmt rgb24 pipe:1
        let output = Command::new("ffmpeg")
            .args([
                "-ss", &format!("{:.6}", time_secs),
                "-i", &self.path,
                "-vframes", "1",
                "-f", "rawvideo",
                "-pix_fmt", "rgb24",
                "-an",       // 音声なし
                "-sn",       // 字幕なし
                "-hide_banner",
                "-loglevel", "error",
                "pipe:1",    // stdout出力
            ])
            .output()
            .context("ffmpeg を実行できませんでした")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("ffmpegがエラーを返しました: {}", stderr);
        }

        let rgb_data = output.stdout;
        let expected_size = (width * height * 3) as usize;

        if rgb_data.len() < expected_size {
            anyhow::bail!(
                "フレームデータが不足しています。期待={}, 実際={}",
                expected_size,
                rgb_data.len()
            );
        }

        let rgb_data = rgb_data[..expected_size].to_vec();

        // キャッシュに保存（簡易的なフレーム番号キャッシュ）
        let frame_idx = (time_secs * self.info.fps).round() as u64;
        self.cached_frame = Some((frame_idx, rgb_data.clone()));

        Ok((rgb_data, width, height))
    }
}