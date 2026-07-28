use fltk::{
    enums::{Color, Font, FrameType, Align},
    frame::Frame,
    group::Group,
    input::FloatInput,
    prelude::*,
    valuator::HorNiceSlider,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::gui::state::AppState;

pub struct PropertiesPanel {
    pub group: Group,
    info_frame: Frame,
    start_input: FloatInput,
    duration_input: FloatInput,
    in_point_input: FloatInput,
    out_point_input: FloatInput,
    position_slider: HorNiceSlider,
    position_label: Frame,
    state: Rc<RefCell<AppState>>,
}

impl PropertiesPanel {
    pub fn new(x: i32, y: i32, w: i32, h: i32, state: Rc<RefCell<AppState>>) -> Self {
        let mut group = Group::default().with_size(w, h).with_pos(x, y);
        group.set_frame(FrameType::BorderBox);
        group.set_color(Color::from_rgb(38, 38, 43));
        group.begin();

        let y_gap = 30;
        let margin = 8;
        let field_h = 26;
        let label_w = 55;
        let field_w = w - margin * 2 - label_w;

        // Title
        let mut title = Frame::default().with_size(w - margin * 2, 22).with_pos(margin, margin);
        title.set_label("プロパティ");
        title.set_label_color(Color::from_rgb(200, 200, 210));
        title.set_label_font(Font::HelveticaBold);
        title.set_label_size(13);
        title.set_align(Align::Left | Align::Inside);
        title.set_frame(FrameType::FlatBox);
        title.set_color(Color::from_rgb(30, 30, 35));

        // Info frame
        let info_y = margin + y_gap;
        let mut info = Frame::default().with_size(w - margin * 2, 50).with_pos(margin, info_y);
        info.set_label("クリップが選択されていません");
        info.set_label_color(Color::from_rgb(150, 150, 160));
        info.set_label_size(11);
        info.set_align(Align::Left | Align::Top | Align::Inside);
        info.set_frame(FrameType::FlatBox);
        info.set_color(Color::from_rgb(45, 45, 50));

        // Position slider
        let pos_y = info_y + 55;
        let mut pos_label = Frame::default().with_size(100, 18).with_pos(margin, pos_y);
        pos_label.set_label("再生位置: 0:00");
        pos_label.set_label_color(Color::from_rgb(180, 180, 190));
        pos_label.set_label_size(11);
        pos_label.set_align(Align::Left | Align::Inside);

        let mut pos_slider = HorNiceSlider::default().with_size(w - margin * 2, 20).with_pos(margin, pos_y + 18);
        pos_slider.set_minimum(0.0);
        pos_slider.set_maximum(1.0);
        pos_slider.set_value(0.0);

        let state_slider = state.clone();
        pos_slider.set_callback(move |slider| {
            if let Ok(mut s) = state_slider.try_borrow_mut() {
                s.set_playback_position(slider.value());
            }
        });

        // Fields
        let field_start_y = pos_y + 50;
        let make_label = |text: &str, y: i32| -> Frame {
            let mut l = Frame::default().with_size(label_w, field_h).with_pos(margin, y);
            l.set_label(text);
            l.set_label_color(Color::from_rgb(150, 150, 160));
            l.set_label_size(11);
            l.set_align(Align::Right | Align::Inside);
            l
        };

        let lbl_start = make_label("開始:", field_start_y);
        let start_input = FloatInput::default().with_size(field_w, field_h).with_pos(margin + label_w, field_start_y);

        let dur_y = field_start_y + y_gap;
        let lbl_dur = make_label("長さ:", dur_y);
        let duration_input = FloatInput::default().with_size(field_w, field_h).with_pos(margin + label_w, dur_y);

        let in_y = dur_y + y_gap;
        let lbl_in = make_label("In点:", in_y);
        let in_point_input = FloatInput::default().with_size(field_w, field_h).with_pos(margin + label_w, in_y);

        let out_y = in_y + y_gap;
        let lbl_out = make_label("Out点:", out_y);
        let out_point_input = FloatInput::default().with_size(field_w, field_h).with_pos(margin + label_w, out_y);

        group.end();

        Self {
            group,
            info_frame: info,
            start_input,
            duration_input,
            in_point_input,
            out_point_input,
            position_slider: pos_slider,
            position_label: pos_label,
            state,
        }
    }

    pub fn update_from_selection(&mut self) {
        let s = self.state.borrow();
        let dur = s.project.duration;
        self.position_slider.set_maximum(dur.max(1.0));
        self.position_slider.set_value(s.playback_position);
        let pos_str = crate::utils::time::format_duration(s.playback_position);
        self.position_label.set_label(&format!("再生位置: {}", pos_str));

        if let Some(clip) = s.selected_clip() {
            self.info_frame.set_label(&format!(
                "クリップ: {}\nタイプ: {} | Trk{} | ID:{}",
                clip.label, clip.media_type.label(), clip.track_idx + 1, clip.id
            ));
            self.info_frame.set_label_color(Color::from_rgb(200, 200, 210));
            self.start_input.set_value(&format!("{:.2}", clip.start_time));
            self.duration_input.set_value(&format!("{:.2}", clip.duration));
            self.in_point_input.set_value(&format!("{:.2}", clip.in_point));
            self.out_point_input.set_value(&format!("{:.2}", clip.out_point));
        } else {
            self.info_frame.set_label("クリップが選択されていません");
            self.info_frame.set_label_color(Color::from_rgb(150, 150, 160));
        }
    }

    pub fn apply_changes(&mut self) {
        let clip_id = match self.state.borrow().selected_clip_id {
            Some(id) => id,
            None => return,
        };
        let mut s = self.state.borrow_mut();
        if let Some(clip) = s.project.find_clip_mut(clip_id) {
            if let Ok(v) = self.start_input.value().parse::<f64>() {
                clip.start_time = v;
            }
            if let Ok(v) = self.duration_input.value().parse::<f64>() {
                clip.duration = v.max(0.01);
                clip.out_point = clip.in_point + clip.duration;
            }
            if let Ok(v) = self.in_point_input.value().parse::<f64>() {
                clip.in_point = v.max(0.0);
                clip.duration = clip.out_point - clip.in_point;
            }
            if let Ok(v) = self.out_point_input.value().parse::<f64>() {
                clip.out_point = v.max(clip.in_point);
                clip.duration = clip.out_point - clip.in_point;
            }
            s.project.update_duration();
            s.is_modified = true;
        }
    }
}