use fltk::{
    enums::{Color, Font, Shortcut},
    menu::MenuBar,
    prelude::*,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::gui::state::AppState;

pub struct AppMenu {
    pub menu_bar: MenuBar,
}

impl AppMenu {
    pub fn new(w: i32, h: i32) -> Self {
        let mut menu_bar = MenuBar::default().with_size(w, h).with_pos(0, 0);
        menu_bar.set_color(Color::from_rgb(40, 40, 45));
        menu_bar.set_text_color(Color::from_rgb(200, 200, 210));
        menu_bar.set_text_font(Font::Helvetica);
        menu_bar.set_text_size(13);
        Self { menu_bar }
    }

    pub fn setup_menus(&mut self, state: Rc<RefCell<AppState>>, extra_width: i32) {
        self.menu_bar.resize(0, 0, extra_width.max(400), 28);
        self.menu_bar.clear();

        // ---- File ----
        let s1 = state.clone();
        self.menu_bar.add(
            "ファイル/新規プロジェクト\t",
            Shortcut::Ctrl | 'n',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Ok(mut s) = s1.try_borrow_mut() {
                    *s = AppState::new();
                    println!("New project");
                }
            },
        );

        let s2 = state.clone();
        self.menu_bar.add(
            "ファイル/プロジェクトを開く...\t",
            Shortcut::Ctrl | 'o',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Some(path) = crate::gui::dialogs::open_file_dialog(
                    "プロジェクトを開く",
                    "LYM Project (*.lym)",
                ) {
                    if let Ok(mut s) = s2.try_borrow_mut() {
                        match crate::gui::state::Project::load_from_file(&path) {
                            Ok(proj) => {
                                s.project = proj;
                                s.project_file_path = Some(PathBuf::from(&path));
                                s.is_modified = false;
                                println!("Project loaded: {}", path);
                            }
                            Err(e) => {
                                eprintln!("Load error: {}", e);
                            }
                        }
                    }
                }
            },
        );

        let s3 = state.clone();
        self.menu_bar.add(
            "ファイル/プロジェクトを保存\t",
            Shortcut::Ctrl | 's',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                let path = {
                    let s = s3.borrow();
                    s.project_file_path.clone()
                };
                if let Some(p) = path {
                    if let Ok(mut s) = s3.try_borrow_mut() {
                        if let Err(e) = s.project.save_to_file(&p.to_string_lossy()) {
                            eprintln!("Save error: {}", e);
                        } else {
                            s.is_modified = false;
                            println!("Saved: {}", p.display());
                        }
                    }
                } else {
                    save_as(&s3);
                }
            },
        );

        let s4 = state.clone();
        self.menu_bar.add(
            "ファイル/名前を付けて保存...\t",
            Shortcut::Ctrl | Shortcut::Shift | 's',
            fltk::menu::MenuFlag::Normal,
            move |_| save_as(&s4),
        );

        let s5 = state.clone();
        self.menu_bar.add(
            "ファイル/エクスポート/動画を書き出し...\t",
            Shortcut::Ctrl | 'e',
            fltk::menu::MenuFlag::Normal,
            move |_| export_video(&s5),
        );

        self.menu_bar.add(
            "ファイル/終了\t",
            Shortcut::Ctrl | 'q',
            fltk::menu::MenuFlag::Normal,
            |_| std::process::exit(0),
        );

        // ---- Edit ----
        let s6 = state.clone();
        self.menu_bar.add(
            "編集/元に戻す\t",
            Shortcut::Ctrl | 'z',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Ok(mut s) = s6.try_borrow_mut() {
                    if let Some(cmd) = s.undo_stack.undo() {
                        apply_undo_redo(&mut s.project, &cmd);
                    }
                }
            },
        );

        let s7 = state.clone();
        self.menu_bar.add(
            "編集/やり直し\t",
            Shortcut::Ctrl | 'y',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Ok(mut s) = s7.try_borrow_mut() {
                    if let Some(cmd) = s.undo_stack.redo() {
                        apply_undo_redo(&mut s.project, &cmd);
                    }
                }
            },
        );

        let s8 = state.clone();
        self.menu_bar.add(
            "編集/選択クリップを削除\t",
            Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Ok(mut s) = s8.try_borrow_mut() {
                    if let Some(id) = s.selected_clip_id {
                        s.project.remove_clip(id);
                        s.selected_clip_id = None;
                        s.is_modified = true;
                    }
                }
            },
        );

        let s9 = state.clone();
        self.menu_bar.add(
            "編集/再生ヘッド位置で分割\t",
            Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Ok(mut s) = s9.try_borrow_mut() {
                    if let Some(id) = s.selected_clip_id {
                        split_clip_at_playhead(&mut s);
                    }
                }
            },
        );

        // ---- Track ----
        let sa = state.clone();
        self.menu_bar.add(
            "トラック/トラックを追加\t",
            Shortcut::Ctrl | 't',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Ok(mut s) = sa.try_borrow_mut() {
                    let idx = s.project.tracks.len() + 1;
                    s.project.add_track(&format!("トラック {}", idx));
                    s.is_modified = true;
                }
            },
        );

        // ---- View ----
        let sb = state.clone();
        self.menu_bar.add(
            "表示/ズームイン\t",
            Shortcut::Ctrl | '=',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Ok(mut s) = sb.try_borrow_mut() {
                    s.zoom_level = (s.zoom_level * 1.3).min(500.0);
                }
            },
        );

        let sc = state.clone();
        self.menu_bar.add(
            "表示/ズームアウト\t",
            Shortcut::Ctrl | '-',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Ok(mut s) = sc.try_borrow_mut() {
                    s.zoom_level = (s.zoom_level / 1.3).max(10.0);
                }
            },
        );

        // ---- Help ----
        self.menu_bar.add(
            "ヘルプ/LYMについて\t",
            Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            |_| {
                println!("Linux Yukkuri Maker - LYM v{}", env!("CARGO_PKG_VERSION"));
            },
        );
    }
}

fn save_as(state: &Rc<RefCell<AppState>>) {
    if let Some(path) =
        crate::gui::dialogs::save_file_dialog(
            "名前を付けて保存",
            "LYM Project (*.lym)",
            "project.lym",
        )
    {
        let final_path = if !path.ends_with(".lym") {
            format!("{}.lym", path)
        } else {
            path
        };
        if let Ok(mut s) = state.try_borrow_mut() {
            if let Err(e) = s.project.save_to_file(&final_path) {
                eprintln!("Save error: {}", e);
            } else {
                s.project_file_path = Some(PathBuf::from(&final_path));
                s.is_modified = false;
                println!("Saved as: {}", final_path);
            }
        }
    }
}

fn export_video(state: &Rc<RefCell<AppState>>) {
    if let Some(path) =
        crate::gui::dialogs::save_file_dialog(
            "動画を書き出し",
            "MP4 Video (*.mp4)",
            "output.mp4",
        )
    {
        let final_path = if !path.ends_with(".mp4") {
            format!("{}.mp4", path)
        } else {
            path
        };

        let s = match state.try_borrow() {
            Ok(s) => s,
            Err(_) => return,
        };

        // Build ffmpeg concat command
        let video_clips: Vec<_> = s
            .project
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .filter(|c| c.media_type == crate::gui::state::MediaType::Video)
            .collect();

        if video_clips.is_empty() {
            eprintln!("No video clips to export");
            return;
        }

        println!("Exporting {} clips to {}...", video_clips.len(), final_path);

        // Use simple ffmpeg concat with filters for each clip
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.arg("-y"); // overwrite

        // Add inputs
        for clip in &video_clips {
            if let Some(ref fp) = clip.file_path {
                cmd.args(["-ss", &format!("{:.3}", clip.in_point)])
                    .args(["-t", &format!("{:.3}", clip.duration)])
                    .arg("-i")
                    .arg(fp);
            }
        }

        // Build filter complex for concat
        let mut filter_parts = Vec::new();
        let mut concat_inputs = Vec::new();
        for i in 0..video_clips.len() {
            filter_parts.push(format!("[{}:v]setpts=PTS-STARTPTS[v{}]", i, i));
            concat_inputs.push(format!("[v{}]", i));
        }
        let filter = format!(
            "{}; {}concat=n={}:v=1:a=0[outv]",
            filter_parts.join(";"),
            concat_inputs.join(""),
            video_clips.len()
        );

        cmd.args([
            "-filter_complex", &filter,
            "-map", "[outv]",
            "-c:v", "libx264",
            "-preset", "medium",
            "-crf", "23",
            &final_path,
        ]);

        match cmd.output() {
            Ok(out) => {
                if out.status.success() {
                    println!("Export complete: {}", final_path);
                } else {
                    eprintln!(
                        "Export failed: {}",
                        String::from_utf8_lossy(&out.stderr)
                    );
                }
            }
            Err(e) => eprintln!("ffmpeg error: {}", e),
        }
    }
}

fn apply_undo_redo(
    project: &mut crate::gui::state::Project,
    cmd: &crate::gui::state::EditCommand,
) {
    match cmd {
        crate::gui::state::EditCommand::AddClip { clip } => {
            project.remove_clip(clip.id);
        }
        crate::gui::state::EditCommand::RemoveClip { clip } => {
            let mut c = clip.clone();
            project.add_clip(
                c.file_path.take(),
                &c.label,
                c.start_time,
                c.duration,
                c.media_type,
                c.track_idx,
            );
        }
        crate::gui::state::EditCommand::MoveClip {
            clip_id,
            old_start_time,
            old_track_idx,
            ..
        } => {
            if let Some(clip) = project.find_clip_mut(*clip_id) {
                clip.start_time = *old_start_time;
                clip.track_idx = *old_track_idx;
            }
        }
        crate::gui::state::EditCommand::TrimClip {
            clip_id,
            old_in_point,
            old_out_point,
            old_duration,
            ..
        } => {
            if let Some(clip) = project.find_clip_mut(*clip_id) {
                clip.in_point = *old_in_point;
                clip.out_point = *old_out_point;
                clip.duration = *old_duration;
            }
        }
        crate::gui::state::EditCommand::SplitClip { original_clip, .. } => {
            let mut c = original_clip.clone();
            project.add_clip(
                c.file_path.take(),
                &c.label,
                c.start_time,
                c.duration,
                c.media_type,
                c.track_idx,
            );
        }
    }
    project.update_duration();
}

fn split_clip_at_playhead(state: &mut AppState) {
    let clip_id = match state.selected_clip_id {
        Some(id) => id,
        None => return,
    };
    let pos = state.playback_position;
    let (original, new_clip) = {
        let project = &state.project;
        let clip = match project.find_clip(clip_id) {
            Some(c) => c.clone(),
            None => return,
        };
        if pos <= clip.start_time || pos >= clip.start_time + clip.duration {
            return;
        }
        let split_point = pos - clip.start_time;
        let in_point = clip.in_point;
        let first_dur = split_point;
        let second_dur = clip.duration - split_point;
        let mut first = clip.clone();
        first.duration = first_dur;
        first.out_point = in_point + first_dur;
        let mut second = clip.clone();
        second.start_time = pos;
        second.duration = second_dur;
        second.in_point = in_point + split_point;
        second.out_point = clip.out_point;
        (first, second)
    };

    // Remove original and add two parts
    state.project.remove_clip(clip_id);
    let fid = state.project.add_clip(
        original.file_path.clone(),
        &original.label,
        original.start_time,
        original.duration,
        original.media_type,
        original.track_idx,
    );
    state.project.add_clip(
        new_clip.file_path.clone(),
        &format!("{} (2)", new_clip.label),
        new_clip.start_time,
        new_clip.duration,
        new_clip.media_type,
        new_clip.track_idx,
    );
    state.selected_clip_id = Some(fid);
    state.is_modified = true;
    state.project.update_duration();
}