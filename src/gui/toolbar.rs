use fltk::{
    enums::{Color, Font, FrameType, Align, Event},
    frame::Frame,
    group::Group,
    prelude::*,
    button::Button,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::gui::state::{AppState, PlaybackState};
use crate::gui::widgets::colors;

/// ツールバー
pub struct Toolbar {
    pub group: Group,
    play_button: Button,
    stop_button: Button,
    prev_frame_button: Button,
    next_frame_button: Button,
    add_track_button: Button,
    state_label: Frame,
}

impl Toolbar {
    pub fn new(x: i32, y: i32, w: i32, h: i32, state: Rc<RefCell<AppState>>) -> Self {
        let mut group = Group::default()
            .with_size(w, h)
            .with_pos(x, y);

        group.set_color(colors::BG_PANEL);

        let btn_h = h - 4;
        let btn_w = 40;
        let gap = 2;

        // 停止ボタン
        let mut stop = Button::default()
            .with_size(btn_w, btn_h)
            .with_pos(gap + 2, 2);
        stop.set_label("⏹");
        stop.set_color(Color::from_rgb(55, 55, 60));
        stop.set_label_color(Color::from_rgb(200, 200, 210));
        stop.set_label_size(16);
        stop.set_tooltip("停止");

        // 再生ボタン
        let mut play = Button::default()
            .with_size(btn_w, btn_h)
            .with_pos(gap + btn_w + 4, 2);
        play.set_label("▶");
        play.set_color(Color::from_rgb(55, 55, 60));
        play.set_label_color(Color::from_rgb(220, 220, 230));
        play.set_label_size(16);
        play.set_tooltip("再生/一時停止 (Space)");

        let s_play = state.clone();
        play.set_callback(move |p| {
            if let Ok(mut s) = s_play.try_borrow_mut() {
                match s.playback_state {
                    PlaybackState::Stopped | PlaybackState::Paused => {
                        s.playback_state = PlaybackState::Playing;
                        p.set_label("⏸");
                    }
                    PlaybackState::Playing => {
                        s.playback_state = PlaybackState::Paused;
                        p.set_label("▶");
                    }
                }
            }
        });

        let mut prev = Button::default()
            .with_size(btn_w, btn_h)
            .with_pos(gap + btn_w * 2 + 6, 2);
        prev.set_label("⏮");
        prev.set_color(Color::from_rgb(55, 55, 60));
        prev.set_label_color(Color::from_rgb(200, 200, 210));
        prev.set_label_size(16);
        prev.set_tooltip("前のフレーム");

        let mut next = Button::default()
            .with_size(btn_w, btn_h)
            .with_pos(gap + btn_w * 3 + 8, 2);
        next.set_label("⏭");
        next.set_color(Color::from_rgb(55, 55, 60));
        next.set_label_color(Color::from_rgb(200, 200, 210));
        next.set_label_size(16);
        next.set_tooltip("次のフレーム");

        // 区切り線
        let sep_x = gap + btn_w * 4 + 14;
        let mut sep = Frame::default()
            .with_size(1, btn_h)
            .with_pos(sep_x, 2);
        sep.set_frame(FrameType::FlatBox);
        sep.set_color(Color::from_rgb(70, 70, 75));

        // トラック追加ボタン
        let mut add_track = Button::default()
            .with_size(btn_w + 30, btn_h)
            .with_pos(sep_x + 6, 2);
        add_track.set_label("+ Track");
        add_track.set_color(Color::from_rgb(55, 55, 60));
        add_track.set_label_color(Color::from_rgb(180, 220, 180));
        add_track.set_label_size(12);
        add_track.set_tooltip("新しいトラックを追加");

        let s_add = state.clone();
        add_track.set_callback(move |_| {
            if let Ok(mut s) = s_add.try_borrow_mut() {
                let idx = s.project.tracks.len() + 1;
                s.project.add_track(&format!("トラック {}", idx));
                s.is_modified = true;
                println!("トラック追加: トラック {}", idx);
            }
        });

        // 状態ラベル
        let label_x = sep_x + btn_w + 44;
        let mut state_label = Frame::default()
            .with_size(w - label_x - 8, btn_h)
            .with_pos(label_x, 2);
        state_label.set_label("準備完了");
        state_label.set_label_color(Color::from_rgb(150, 150, 160));
        state_label.set_label_size(11);
        state_label.set_align(Align::Right | Align::Inside);

        group.add(&stop);
        group.add(&play);
        group.add(&prev);
        group.add(&next);
        group.add(&sep);
        group.add(&add_track);
        group.add(&state_label);

        group.end();

        Self {
            group,
            play_button: play,
            stop_button: stop,
            prev_frame_button: prev,
            next_frame_button: next,
            add_track_button: add_track,
            state_label,
        }
    }

    pub fn update_state(&mut self, state_label_text: &str) {
        self.state_label.set_label(state_label_text);
    }
}