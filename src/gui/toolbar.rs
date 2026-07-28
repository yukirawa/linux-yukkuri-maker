use fltk::{
    enums::{Color, Font, FrameType, Align},
    frame::Frame,
    group::Group,
    prelude::*,
    button::Button,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::gui::state::{AppState, PlaybackState};
use crate::gui::widgets::colors;
use crate::video::GstVideoEngine;

pub struct Toolbar {
    pub group: Group,
    state_label: Frame,
}

impl Toolbar {
    pub fn new(
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        state: Rc<RefCell<AppState>>,
        engine: Rc<RefCell<Option<GstVideoEngine>>>,
    ) -> Self {
        let mut group = Group::default()
            .with_size(w, h)
            .with_pos(x, y);
        group.set_color(colors::BG_PANEL);
        group.begin();

        let btn_h = h - 4;
        let btn_w = 40;
        let gap = 2;

        // Stop button
        let mut stop_btn = Button::default()
            .with_size(btn_w, btn_h)
            .with_pos(gap + 2, 2);
        stop_btn.set_label("@square");
        stop_btn.set_color(Color::from_rgb(55, 55, 60));
        stop_btn.set_label_color(Color::from_rgb(200, 200, 210));
        stop_btn.set_label_size(16);
        stop_btn.set_tooltip("停止");

        let s_stop = state.clone();
        let e_stop = engine.clone();
        stop_btn.set_callback(move |_| {
            if let Ok(mut s) = s_stop.try_borrow_mut() {
                s.playback_state = PlaybackState::Stopped;
                s.playback_position = 0.0;
            }
            if let Ok(eng) = e_stop.try_borrow() {
                if let Some(ref e) = *eng {
                    e.stop();
                }
            }
            println!("⏹ 停止");
        });

        // Play/Pause button
        let mut play_btn = Button::default()
            .with_size(btn_w, btn_h)
            .with_pos(gap + btn_w + 4, 2);
        play_btn.set_label("@>");
        play_btn.set_color(Color::from_rgb(55, 55, 60));
        play_btn.set_label_color(Color::from_rgb(220, 220, 230));
        play_btn.set_label_size(16);
        play_btn.set_tooltip("再生/一時停止");

        let s_play = state.clone();
        let e_play = engine.clone();
        play_btn.set_callback(move |_| {
            if let Ok(mut s) = s_play.try_borrow_mut() {
                match s.playback_state {
                    PlaybackState::Stopped | PlaybackState::Paused => {
                        let pos = s.playback_position;
                        drop(s);
                        if let Ok(eng) = e_play.try_borrow() {
                            if let Some(ref e) = *eng {
                                let _ = e.seek(pos);
                                e.play();
                            }
                        }
                        if let Ok(mut s) = s_play.try_borrow_mut() {
                            s.playback_state = PlaybackState::Playing;
                        }
                        println!("▶ 再生");
                    }
                    PlaybackState::Playing => {
                        s.playback_state = PlaybackState::Paused;
                        drop(s);
                        if let Ok(eng) = e_play.try_borrow() {
                            if let Some(ref e) = *eng {
                                e.pause();
                            }
                        }
                        println!("⏸ 一時停止");
                    }
                }
            }
        });

        // Add track button
        let mut add_track = Button::default()
            .with_size(btn_w + 30, btn_h)
            .with_pos(gap + btn_w * 2 + 12, 2);
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

        // State label
        let mut state_label = Frame::default()
            .with_size(w - 230, btn_h)
            .with_pos(230, 2);
        state_label.set_label("準備完了");
        state_label.set_label_color(Color::from_rgb(150, 150, 160));
        state_label.set_label_size(11);
        state_label.set_align(Align::Right | Align::Inside);

        group.end();

        Self {
            group,
            state_label,
        }
    }

    pub fn update_state(&mut self, text: &str) {
        self.state_label.set_label(text);
    }
}