use fltk::{
    app,
    enums::Event,
    group::Flex,
    prelude::*,
    window::Window,
};
use std::cell::RefCell;
use std::rc::Rc;

use super::media_browser::MediaBrowserPanel;
use super::menu::AppMenu;
use super::preview::PreviewPanel;
use super::properties::PropertiesPanel;
use super::serif_input::SerifInputPanel;
use super::state::{AppState, MediaType};
use super::status_bar::StatusBar;
use super::timeline::TimelinePanel;
use super::toolbar::Toolbar;
use super::widgets::colors;
use crate::video::VideoDecoder;

/// メインウィンドウを作成する
pub fn create_main_window() -> (Window, Rc<RefCell<AppState>>) {
    let mut wind = Window::default()
        .with_size(1400, 900)
        .with_label("Linux Yukkuri Maker - LYM");

    wind.set_color(colors::BG_MAIN);

    // アプリ状態
    let state = Rc::new(RefCell::new(AppState::new()));
    let state_for_close = state.clone();

    // デフォルトサイズでパネル幅を計算
    let win_w = 1400;
    let _win_h = 900;
    let menu_h = 28;
    let toolbar_h = 36;
    let status_h = 22;
    let timeline_h = 220;
    let serif_h = 120;
    let media_w = 180;
    let props_w = 250;

    // ===== メインFlexレイアウト (縦) =====
    let mut main_col = Flex::default_fill().column();
    main_col.set_margin(0);
    main_col.set_pad(0);

    // ----- メニューバー -----
    let mut app_menu = AppMenu::new(win_w, menu_h);
    app_menu.setup_menus(state.clone(), win_w);

    // ----- ツールバー -----
    let toolbar = Toolbar::new(0, 0, win_w, toolbar_h, state.clone());

    // ----- 中央領域: メディアブラウザ | プレビュー | プロパティ -----
    let mut center_row = Flex::default().row();
    center_row.set_margin(2);
    center_row.set_pad(2);

    // メディアブラウザ (左)
    let media_panel = Rc::new(RefCell::new(
        MediaBrowserPanel::new(0, 0, media_w, 400)
    ));
    {
        let mb = media_panel.borrow();
        center_row.fixed(&mb.group, media_w);
    }

    // プレビュー (中央、可変)
    let preview_panel = Rc::new(RefCell::new(
        PreviewPanel::new(0, 0, 800, 400)
    ));
    {
        let _pp = preview_panel.borrow();
        // プレビューは可変サイズなので fixed は使わない
    }

    // プロパティ (右)
    let props_panel = Rc::new(RefCell::new(
        PropertiesPanel::new(0, 0, props_w, 400, state.clone())
    ));
    {
        let prop = props_panel.borrow();
        center_row.fixed(&prop.group, props_w);
    }

    center_row.end();

    // ----- タイムライン -----
    let timeline_panel = Rc::new(RefCell::new(
        TimelinePanel::new(0, 0, win_w, timeline_h, state.clone())
    ));
    {
        let tl = timeline_panel.borrow();
        main_col.fixed(&tl.group, timeline_h);
    }

    // ----- セリフ入力欄（固定・スクロール非追従） -----
    let serif_panel = Rc::new(RefCell::new(
        SerifInputPanel::new(0, 0, win_w, serif_h)
    ));
    {
        let sp = serif_panel.borrow();
        main_col.fixed(&sp.group, serif_h);
    }

    // ----- ステータスバー -----
    let mut status_bar = StatusBar::new(0, 0, win_w, status_h);
    status_bar.set_project_name("新規プロジェクト");

    main_col.fixed(&status_bar.group, status_h);

    // Flexレイアウトにメニューバーとツールバーを追加（fixed size）
    main_col.fixed(&app_menu.menu_bar, menu_h);
    main_col.fixed(&toolbar.group, toolbar_h);

    main_col.end();

    // ===== DnD (ドラッグアンドドロップ) 対応 =====
    let state_dnd = state.clone();
    let preview_dnd = preview_panel.clone();
    let timeline_dnd = timeline_panel.clone();

    wind.handle(move |_w, ev| match ev {
        Event::DndEnter => true,
        Event::DndLeave => true,
        Event::DndRelease => true,
        Event::Paste => {
            let text = app::event_text();
            let paths: Vec<&str> = text.split('\n').filter(|s| !s.is_empty()).collect();

            for path in &paths {
                let path_str = path.trim();
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

                    match VideoDecoder::open(clean_path) {
                        Ok(decoder) => {
                            let info = decoder.info().clone();
                            let file_name = std::path::Path::new(clean_path)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "不明".to_string());

                            // 状態を更新
                            if let Ok(mut s) = state_dnd.try_borrow_mut() {
                                let start_time = s.playback_position;
                                s.project.add_clip(
                                    Some(clean_path.to_string()),
                                    &format!("{} [{}x{} {}fps]",
                                        file_name, info.width, info.height,
                                        info.fps as i32),
                                    start_time,
                                    info.duration,
                                    MediaType::Video,
                                    0, // デフォルトはトラック0
                                );
                                s.is_modified = true;

                                // 先頭フレームのファイルパスを保存 (VideoDecoder はスコープ外に出せないため)
                                // デコードしてプレビュー表示
                            }

                            // 先頭フレームをプレビューに表示
                            let mut dec = decoder;
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

                            // タイムラインにクリップを追加
                            if let Ok(mut tl) = timeline_dnd.try_borrow_mut() {
                                tl.add_clip(
                                    &file_name,
                                    0.0,
                                    info.duration,
                                    crate::gui::state::MediaType::Video,
                                    0,
                                );
                            }

                            println!(
                                "動画読み込み完了: {} ({}x{}, {:.2}秒)",
                                file_name, info.width, info.height, info.duration
                            );
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
    });

    // リサイズ時に各パネルの内容を更新するためのコールバック
    wind.resize_callback(move |_w, _x, _y, _w2, _h2| {
        // FLTKのFlexが自動的にレイアウトを処理するので、
        // ここでは特別な処理は不要（Flexが子ウィジェットのサイズを調整する）
    });

    // 終了時にクリーンアップ
    wind.set_callback(move |_| {
        if let Ok(s) = state_for_close.try_borrow() {
            if s.is_modified {
                println!("未保存の変更があります");
            }
        }
    });

    (wind, state)
}