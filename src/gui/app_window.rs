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
use super::state::{AppState, MediaType, PlaybackState};
use super::status_bar::StatusBar;
use super::timeline::TimelinePanel;
use super::toolbar::Toolbar;
use crate::video::VideoDecoder;

pub fn create_main_window() -> (Window, Rc<RefCell<AppState>>) {
    let mut wind = Window::default()
        .with_size(1400, 900)
        .with_label("Linux Yukkuri Maker - LYM");
    wind.set_color(fltk::enums::Color::from_rgb(40, 40, 45));

    let state = Rc::new(RefCell::new(AppState::new()));

    // ===== Main vertical Flex =====
    let mut main_col = Flex::default_fill().column();
    main_col.set_margin(0);
    main_col.set_pad(0);

    // ---- BEGIN: main_col children ----
    main_col.begin();

    // MenuBar (28px fixed)
    let mut app_menu = AppMenu::new(1400, 28);
    app_menu.setup_menus(state.clone(), 1400);
    main_col.fixed(&app_menu.menu_bar, 28);

    // Toolbar (32px fixed)
    let toolbar = Toolbar::new(0, 0, 1400, 32, state.clone());
    main_col.fixed(&toolbar.group, 32);

    // Center row Flex (horizontal, 440px height)
    let mut center_row = Flex::default().row();
    center_row.set_pad(2);

    // ---- BEGIN: center_row children ----
    center_row.begin();

    // Left: Media browser (180px fixed width)
    let media_panel = MediaBrowserPanel::new(0, 0, 180, 440);
    center_row.fixed(&media_panel.group, 180);

    // Center: Preview (fill remaining space)
    let preview_panel = PreviewPanel::new(0, 0, 800, 440);
    // preview is NOT fixed so it expands

    // Right: Properties (250px fixed width)
    let props_panel = PropertiesPanel::new(0, 0, 250, 440, state.clone());
    center_row.fixed(&props_panel.group, 250);

    center_row.end();
    // ---- END: center_row children ----
    main_col.fixed(&center_row, 440);

    // Timeline (160px fixed height)
    let timeline_panel = Rc::new(RefCell::new(
        TimelinePanel::new(0, 0, 1400, 160, state.clone())
    ));
    {
        let tl = timeline_panel.borrow();
        main_col.fixed(&tl.group, 160);
    }

    // Serif input (80px fixed height, anchored at bottom)
    let serif_panel = SerifInputPanel::new(0, 0, 1400, 80);
    main_col.fixed(&serif_panel.group, 80);

    // Status bar (20px fixed height)
    let status_bar = StatusBar::new(0, 0, 1400, 20);
    main_col.fixed(&status_bar.group, 20);

    main_col.end();
    // ---- END: main_col children ----

    wind.end();
    wind.make_resizable(true);
    wind.resizable(&main_col);

    // ===== DnD handler =====
    let state_dnd = state.clone();
    let mut preview_frame = preview_panel.frame.clone();
    let timeline_dnd = timeline_panel.clone();
    let mut status_label = status_bar.group.clone();
    let mut prop_update = props_panel.group.clone();

    wind.handle(move |_w, ev| {
        match ev {
            Event::DndEnter | Event::DndLeave | Event::DndRelease => true,
            Event::Paste => {
                let text = app::event_text();
                for p in text.split('\n').filter(|s| !s.is_empty()) {
                    let clean = p.trim().strip_prefix("file://").unwrap_or(p.trim());
                    let lower = clean.to_lowercase();
                    let is_video = ["mp4","mov","avi","mkv","webm","wmv","flv","ts"]
                        .iter().any(|e| lower.ends_with(e));
                    let is_audio = ["mp3","wav","ogg","flac","aac","m4a","wma"]
                        .iter().any(|e| lower.ends_with(e));
                    if !is_video && !is_audio {
                        continue;
                    }

                    let mtype = if is_video {
                        MediaType::Video
                    } else {
                        MediaType::Audio
                    };
                    let fname = std::path::Path::new(clean)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".into());
                    let dur = probe_duration(clean).unwrap_or(10.0);

                    // Add to project state
                    {
                        let mut s = state_dnd.borrow_mut();
                        let pos = s.playback_position;
                        let tidx = s.project.tracks.first().map(|t| t.index).unwrap_or(0);
                        s.project.add_clip(
                            Some(clean.to_string()),
                            &fname,
                            pos,
                            dur,
                            mtype,
                            tidx,
                        );
                        s.playback_position += dur;
                        s.is_modified = true;
                    }

                    // Video preview (first frame)
                    if is_video {
                        if let Ok(dec) = VideoDecoder::open(clean) {
                            let mut decoder = dec;
                            if let Ok((rgb, w, h)) = decoder.first_frame() {
                                if let Ok(img) = fltk::image::RgbImage::new(
                                    &rgb,
                                    w as i32,
                                    h as i32,
                                    fltk::enums::ColorDepth::Rgb8,
                                ) {
                                    preview_frame.set_image(Some(img));
                                    preview_frame.set_label("");
                                    preview_frame.redraw();
                                }
                            }
                        }
                    }

                    // Update timeline
                    if let Ok(mut tl) = timeline_dnd.try_borrow_mut() {
                        tl.redraw();
                    }

                    // Update status bar
                    let s = state_dnd.borrow();
                    let clips: usize = s
                        .project
                        .tracks
                        .iter()
                        .map(|t| t.clips.len())
                        .sum();
                    let total = crate::utils::time::format_duration(s.project.duration);
                    let msg = format!(
                        "LYM  |  クリップ {}件  |  全体 {}  |  次の追加位置: {}",
                        clips,
                        total,
                        crate::utils::time::format_duration(s.playback_position)
                    );
                    drop(s);
                    status_label.set_label(&msg);
                    status_label.redraw();

                    // Force properties panel to re-layout
                    prop_update.redraw();

                    println!("DnD: {} ({:.1}s)", fname, dur);
                }
                true
            }
            Event::KeyDown => true,
            _ => false,
        }
    });

    (wind, state)
}

fn probe_duration(path: &str) -> Result<f64, String> {
    use std::process::Command;
    let out = Command::new("ffprobe")
        .args(["-v", "quiet", "-print_format", "json", "-show_format", path])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into());
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).map_err(|e| e.to_string())?;
    v["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| "no duration".into())
}