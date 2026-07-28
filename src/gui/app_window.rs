use fltk::{app, enums::Event, group::Flex, prelude::*, window::Window};
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
use crate::video::GstVideoEngine;

pub fn create_main_window() -> (Window, Rc<RefCell<AppState>>) {
    let mut wind = Window::default()
        .with_size(1400, 900)
        .with_label("Linux Yukkuri Maker - LYM");
    wind.set_color(fltk::enums::Color::from_rgb(40, 40, 45));

    let state = Rc::new(RefCell::new(AppState::new()));
    let engine: Rc<RefCell<Option<GstVideoEngine>>> = Rc::new(RefCell::new(None));

    // Main vertical Flex + begin
    let mut main_col = Flex::default_fill().column();
    main_col.set_margin(0);
    main_col.set_pad(0);
    main_col.begin();

    let mut app_menu = AppMenu::new(1400, 28);
    app_menu.setup_menus(state.clone(), 1400);
    main_col.fixed(&app_menu.menu_bar, 28);

    let toolbar = Toolbar::new(0, 0, 1400, 32, state.clone(), engine.clone());
    main_col.fixed(&toolbar.group, 32);

    let mut center_row = Flex::default().row();
    center_row.set_pad(2);
    center_row.begin();

    let media_panel = MediaBrowserPanel::new(0, 0, 180, 440);
    center_row.fixed(&media_panel.group, 180);

    let preview_panel = Rc::new(RefCell::new(PreviewPanel::new(0, 0, 800, 440)));

    let props_panel = Rc::new(RefCell::new(PropertiesPanel::new(
        0,
        0,
        250,
        440,
        state.clone(),
    )));
    center_row.fixed(&props_panel.borrow().group, 250);
    center_row.end();
    main_col.fixed(&center_row, 440);

    let timeline_panel = Rc::new(RefCell::new(TimelinePanel::new(
        0,
        0,
        1400,
        160,
        state.clone(),
    )));
    main_col.fixed(&timeline_panel.borrow().group, 160);

    let serif_panel = SerifInputPanel::new(0, 0, 1400, 80);
    main_col.fixed(&serif_panel.group, 80);

    let status_bar = StatusBar::new(0, 0, 1400, 20);
    main_col.fixed(&status_bar.group, 20);

    main_col.end();
    wind.end();
    wind.make_resizable(true);
    wind.resizable(&main_col);

    // ===== playback timer (runs every ~16ms) =====
    {
        let state_t = state.clone();
        let engine_t = engine.clone();
        let preview_t = preview_panel.clone();
        let timeline_t = timeline_panel.clone();
        let props_t = props_panel.clone();
        let mut status_t = status_bar.group.clone();

        fltk::app::add_timeout(1.0 / 60.0, move || {
            let mut playing = false;
            if let Ok(s) = state_t.try_borrow() {
                playing = s.playback_state == PlaybackState::Playing;
            }

            if playing {
                if let Ok(eng) = engine_t.try_borrow() {
                    if let Some(ref e) = *eng {
                        if let Some((rgb, w, h)) = e.poll_frame() {
                            if let Ok(mut pp) = preview_t.try_borrow_mut() {
                                pp.update_frame(&rgb, w, h);
                            }
                        }
                        let pos = e.current_position();
                        if let Ok(mut s) = state_t.try_borrow_mut() {
                            s.set_playback_position(pos);
                            if pos >= s.project.duration && s.project.duration > 0.0 {
                                s.playback_state = PlaybackState::Stopped;
                                e.stop();
                            }
                        }
                    } else if let Ok(mut s) = state_t.try_borrow_mut() {
                        s.playback_state = PlaybackState::Stopped;
                    }
                }
            }

            if let Ok(mut tl) = timeline_t.try_borrow_mut() {
                tl.redraw();
            }
            if let Ok(mut pp) = props_t.try_borrow_mut() {
                pp.update_from_selection();
            }
            if let Ok(s) = state_t.try_borrow() {
                let clips: usize = s.project.tracks.iter().map(|t| t.clips.len()).sum();
                let total = crate::utils::time::format_duration(s.project.duration);
                let pos = crate::utils::time::format_duration(s.playback_position);
                let state_text = match s.playback_state {
                    PlaybackState::Playing => "▶再生中",
                    PlaybackState::Paused => "⏸一時停止",
                    PlaybackState::Stopped => "■停止",
                };
                status_t.set_label(&format!(
                    "クリップ {}件 | 位置 {} / {} | {}",
                    clips, pos, total, state_text
                ));
                status_t.redraw();
            }
        });
    }

    // ===== DnD handler =====
    let state_dnd = state.clone();
    let engine_dnd = engine.clone();
    let preview_dnd = preview_panel.clone();
    let timeline_dnd = timeline_panel.clone();
    let props_dnd = props_panel.clone();

    wind.handle(move |_w, ev| match ev {
        Event::DndEnter | Event::DndLeave | Event::DndRelease => true,
        Event::Paste => {
            let text = app::event_text();
            for p in text.split('\n').filter(|s| !s.is_empty()) {
                let clean = p.trim().strip_prefix("file://").unwrap_or(p.trim());
                let lower = clean.to_lowercase();
                let is_video = ["mp4", "mov", "avi", "mkv", "webm", "wmv", "flv", "ts"]
                    .iter()
                    .any(|e| lower.ends_with(e));
                let is_audio = ["mp3", "wav", "ogg", "flac", "aac", "m4a", "wma"]
                    .iter()
                    .any(|e| lower.ends_with(e));
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

                // Add to project
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

                // Open GStreamer engine for the first video
                if is_video {
                    match GstVideoEngine::open(clean) {
                        Ok(eng) => {
                            // Show first frame in preview
                            if let Ok((rgb, w, h)) = eng.first_frame() {
                                if let Ok(mut pp) = preview_dnd.try_borrow_mut() {
                                    pp.update_frame(&rgb, w, h);
                                }
                            }
                            // Store engine for playback
                            if let Ok(mut e) = engine_dnd.try_borrow_mut() {
                                *e = Some(eng);
                            }
                        }
                        Err(e) => eprintln!("GStreamer open error: {}", e),
                    }
                }

                // Redraw timeline and properties
                if let Ok(mut tl) = timeline_dnd.try_borrow_mut() {
                    tl.redraw();
                }
                if let Ok(mut pp) = props_dnd.try_borrow_mut() {
                    pp.update_from_selection();
                }

                println!("DnD: {} ({:.1}s)", fname, dur);
            }
            true
        }
        Event::KeyDown => {
            let raw = fltk::app::event_key().bits();
            // Space: toggle playback
            if raw == 32 {
                if let Ok(mut s) = state_dnd.try_borrow_mut() {
                    if let Ok(eng) = engine_dnd.try_borrow() {
                        match s.playback_state {
                            PlaybackState::Stopped | PlaybackState::Paused => {
                                if let Some(ref e) = *eng {
                                    // Seek to current playhead position
                                    let pos = s.playback_position;
                                    drop(s);
                                    drop(eng);
                                    let _ = engine_dnd
                                        .try_borrow()
                                        .ok()
                                        .and_then(|e| e.as_ref().map(|eng| eng.seek(pos).ok()));
                                    if let Ok(mut s) = state_dnd.try_borrow_mut() {
                                        s.playback_state = PlaybackState::Playing;
                                    }
                                    if let Ok(eng) = engine_dnd.try_borrow() {
                                        if let Some(ref e) = *eng {
                                            e.play();
                                        }
                                    }
                                }
                            }
                            PlaybackState::Playing => {
                                s.playback_state = PlaybackState::Paused;
                                drop(s);
                                if let Ok(eng) = engine_dnd.try_borrow() {
                                    if let Some(ref e) = *eng {
                                        e.pause();
                                    }
                                }
                            }
                        }
                    }
                }
                return true;
            }
            // Delete: remove selected clip
            if raw == 0xFFFF {
                if let Ok(mut s) = state_dnd.try_borrow_mut() {
                    if let Some(id) = s.selected_clip_id {
                        s.project.remove_clip(id);
                        s.selected_clip_id = None;
                        s.is_modified = true;
                        drop(s);
                        if let Ok(mut tl) = timeline_dnd.try_borrow_mut() {
                            tl.redraw();
                        }
                        if let Ok(mut pp) = props_dnd.try_borrow_mut() {
                            pp.update_from_selection();
                        }
                    }
                }
                return true;
            }
            // Ctrl+S: save
            if raw == ('s' as i32) && (fltk::app::event_state().bits() & 4 != 0) {
                let path = state_dnd.borrow().project_file_path.clone();
                if let Some(p) = path {
                    if let Ok(s) = state_dnd.try_borrow() {
                        let _ = s.project.save_to_file(&p.to_string_lossy());
                    }
                }
                return true;
            }
            // Ctrl+Z: undo
            if raw == ('z' as i32) && (fltk::app::event_state().bits() & 4 != 0) {
                if let Ok(mut s) = state_dnd.try_borrow_mut() {
                    if let Some(cmd) = s.undo_stack.undo() {
                        super::menu::apply_undo_redo(&mut s.project, &cmd);
                    }
                }
                if let Ok(mut tl) = timeline_dnd.try_borrow_mut() {
                    tl.redraw();
                }
                return true;
            }
            false
        }
        _ => false,
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