use fltk::{
    app,
    enums::Event,
    group::Flex,
    prelude::*,
    window::Window,
};
use std::cell::RefCell;
use std::rc::Rc;

use super::preview::PreviewPanel;
use super::text_input::TextInputPanel;
use super::timeline::TimelinePanel;
use crate::video::VideoDecoder;

/// アプリケーション全体の状態を保持
pub struct AppState {
    /// 現在ロードされている動画ファイルのパス
    pub video_path: Option<String>,
    /// タイムライン上のクリップ情報
    pub clips: Vec<ClipInfo>,
    /// 動画デコーダー
    pub decoder: Option<VideoDecoder>,
}

#[derive(Clone)]
pub struct ClipInfo {
    pub file_path: String,
    pub start_time: f64,  // タイムライン上の開始位置(秒)
    pub duration: f64,     // 動画の長さ(秒)
    pub label: String,     // 表示ラベル
}

impl AppState {
    pub fn new() -> Self {
        Self {
            video_path: None,
            clips: Vec::new(),
            decoder: None,
        }
    }
}

/// メインウィンドウを作成する
pub fn create_main_window() -> (
    Window,
    Rc<RefCell<PreviewPanel>>,
    Rc<RefCell<TimelinePanel>>,
    Rc<RefCell<AppState>>,
) {
    let mut wind = Window::default()
        .with_size(1280, 720)
        .with_label("ゆっくりメーカー Linux版");

    // アプリ状態 (Rc<RefCell<>> で共有)
    let state = Rc::new(RefCell::new(AppState::new()));

    // ----- メニューバー -----
    let menu_bar_y = 30;

    // ----- メインFlexレイアウト (縦) -----
    let mut main_flex = Flex::default_fill().column();
    main_flex.set_margin(2);
    main_flex.resize(0, menu_bar_y, 1280, 690);

    // ----- 上部領域: プレビュー + テキスト入力 (横Flex) -----
    let mut top_flex = Flex::default().row();

    // プレビューパネル (左側 70%)
    let preview_panel = Rc::new(RefCell::new(
        PreviewPanel::new(0, 0, 896, 460, "プレビュー")
    ));
    {
        let pp = preview_panel.borrow();
        top_flex.fixed(&pp.frame, 896);
    }

    // テキスト入力パネル (右側 30%)
    let text_panel = Rc::new(RefCell::new(
        TextInputPanel::new(0, 0, 384, 460, "セリフ入力")
    ));
    {
        let tp = text_panel.borrow();
        top_flex.fixed(&tp.group, 384);
    }

    top_flex.end();

    // ----- 下部領域: タイムライン (高さ200px固定) -----
    let timeline_panel = Rc::new(RefCell::new(
        TimelinePanel::new(0, 0, 1280, 200, "タイムライン")
    ));
    {
        let tl = timeline_panel.borrow();
        main_flex.fixed(&tl.group, 200);
    }

    main_flex.end();

    // DnD (ドラッグアンドドロップ) 対応
    let state_dnd = state.clone();
    let preview_dnd = preview_panel.clone();
    let timeline_dnd = timeline_panel.clone();

    wind.handle(move |_w, ev| {
        match ev {
            Event::DndEnter => {
                true
            }
            Event::DndLeave => {
                true
            }
            Event::DndRelease => {
                true
            }
            Event::Paste => {
                // DnDでのペースト = ファイルパスの取得
                let text = app::event_text();
                let paths: Vec<&str> = text.split('\n')
                    .filter(|s| !s.is_empty())
                    .collect();

                for path in &paths {
                    let path_str = path.trim();
                    // file:// プレフィックスを除去
                    let clean_path = path_str
                        .strip_prefix("file://")
                        .unwrap_or(path_str);

                    let lower = clean_path.to_lowercase();
                    if lower.ends_with(".mp4")
                        || lower.ends_with(".mov")
                        || lower.ends_with(".avi")
                        || lower.ends_with(".mkv")
                        || lower.ends_with(".webm")
                        || lower.ends_with(".wmv")
                        || lower.ends_with(".flv")
                        || lower.ends_with(".ts")
                    {
                        println!("DnD: 動画ファイル受付: {}", clean_path);

                        // VideoDecoderで動画を開く
                        match VideoDecoder::open(clean_path) {
                            Ok(decoder) => {
                                let info = decoder.info().clone();
                                let file_name = std::path::Path::new(clean_path)
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "不明".to_string());

                                // 状態を更新
                                if let Ok(mut s) = state_dnd.try_borrow_mut() {
                                    s.video_path = Some(clean_path.to_string());
                                    s.clips.push(ClipInfo {
                                        file_path: clean_path.to_string(),
                                        start_time: 0.0,
                                        duration: info.duration,
                                        label: format!("{} [{}x{} {}fps]",
                                            file_name, info.width, info.height,
                                            info.fps as i32),
                                    });
                                    s.decoder = Some(decoder);
                                }

                                // 先頭フレームをプレビューに表示
                                if let Ok(mut s) = state_dnd.try_borrow_mut() {
                                    if let Some(ref mut dec) = s.decoder {
                                        match dec.first_frame() {
                                            Ok((rgb, w, h)) => {
                                                if let Ok(mut pp) = preview_dnd.try_borrow_mut() {
                                                    pp.update_frame(&rgb, w, h);
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("先頭フレームのデコードに失敗: {}", e);
                                            }
                                        }
                                    }
                                }

                                // タイムラインにクリップを追加
                                if let Ok(mut tl) = timeline_dnd.try_borrow_mut() {
                                    tl.add_clip(&file_name, 0.0, info.duration);
                                }

                                println!("動画読み込み完了: {} ({}x{}, {:.2}秒)",
                                    file_name, info.width, info.height, info.duration);
                            }
                            Err(e) => {
                                eprintln!("動画を開けません: {} - エラー: {}", clean_path, e);
                            }
                        }
                    }
                }
                true
            }
            _ => false,
        }
    });

    (wind, preview_panel, timeline_panel, state)
}