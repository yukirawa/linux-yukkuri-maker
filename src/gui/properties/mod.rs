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

/// プロパティ編集パネル
pub struct PropertiesPanel {
    pub group: Group,
    // ラベル表示用
    info_frame: Frame,
    // 編集フィールド
    start_input: FloatInput,
    duration_input: FloatInput,
    in_point_input: FloatInput,
    out_point_input: FloatInput,
    // 再生位置スライダー
    position_slider: HorNiceSlider,
    position_label: Frame,
    // 操作対象のAppState
    state: Rc<RefCell<AppState>>,
}

impl PropertiesPanel {
    pub fn new(x: i32, y: i32, w: i32, h: i32, state: Rc<RefCell<AppState>>) -> Self {
        let mut group = Group::default()
            .with_size(w, h)
            .with_pos(x, y);

        group.set_frame(FrameType::BorderBox);
        group.set_color(Color::from_rgb(38, 38, 43));

        let y_gap = 30;
        let margin = 8;
        let field_h = 26;
        let label_w = 60;
        let field_w = w - margin * 2 - label_w;

        // タイトル
        let mut title = Frame::default()
            .with_size(w - margin * 2, 22)
            .with_pos(margin, margin);
        title.set_label("プロパティ");
        title.set_label_color(Color::from_rgb(200, 200, 210));
        title.set_label_font(Font::HelveticaBold);
        title.set_label_size(13);
        title.set_align(Align::Left | Align::Inside);
        title.set_frame(FrameType::FlatBox);
        title.set_color(Color::from_rgb(30, 30, 35));

        // 情報フレーム
        let info_y = margin + y_gap;
        let mut info = Frame::default()
            .with_size(w - margin * 2, 50)
            .with_pos(margin, info_y);
        info.set_label("クリップが選択されていません");
        info.set_label_color(Color::from_rgb(150, 150, 160));
        info.set_label_size(11);
        info.set_align(Align::Left | Align::Top | Align::Inside);
        info.set_frame(FrameType::FlatBox);
        info.set_color(Color::from_rgb(45, 45, 50));

        // 再生位置
        let pos_y = info_y + 55;
        let mut pos_label = Frame::default()
            .with_size(100, 18)
            .with_pos(margin, pos_y);
        pos_label.set_label("再生位置: 0:00");
        pos_label.set_label_color(Color::from_rgb(180, 180, 190));
        pos_label.set_label_size(11);
        pos_label.set_align(Align::Left | Align::Inside);

        let mut pos_slider = HorNiceSlider::default()
            .with_size(w - margin * 2, 20)
            .with_pos(margin, pos_y + 18);
        pos_slider.set_minimum(0.0);
        pos_slider.set_maximum(1.0);
        pos_slider.set_value(0.0);

        // クリップ編集フィールド
        let field_start_y = pos_y + 50;

        // 開始時間
        let mut label_start = Frame::default()
            .with_size(label_w, field_h)
            .with_pos(margin, field_start_y);
        label_start.set_label("開始:");
        label_start.set_label_color(Color::from_rgb(150, 150, 160));
        label_start.set_label_size(11);
        label_start.set_align(Align::Right | Align::Inside);

        let start_input = FloatInput::default()
            .with_size(field_w, field_h)
            .with_pos(margin + label_w, field_start_y);

        // 長さ
        let dur_y = field_start_y + y_gap;
        let mut label_dur = Frame::default()
            .with_size(label_w, field_h)
            .with_pos(margin, dur_y);
        label_dur.set_label("長さ:");
        label_dur.set_label_color(Color::from_rgb(150, 150, 160));
        label_dur.set_label_size(11);
        label_dur.set_align(Align::Right | Align::Inside);

        let duration_input = FloatInput::default()
            .with_size(field_w, field_h)
            .with_pos(margin + label_w, dur_y);

        // In点
        let in_y = dur_y + y_gap;
        let mut label_in = Frame::default()
            .with_size(label_w, field_h)
            .with_pos(margin, in_y);
        label_in.set_label("In点:");
        label_in.set_label_color(Color::from_rgb(150, 150, 160));
        label_in.set_label_size(11);
        label_in.set_align(Align::Right | Align::Inside);

        let in_point_input = FloatInput::default()
            .with_size(field_w, field_h)
            .with_pos(margin + label_w, in_y);

        // Out点
        let out_y = in_y + y_gap;
        let mut label_out = Frame::default()
            .with_size(label_w, field_h)
            .with_pos(margin, out_y);
        label_out.set_label("Out点:");
        label_out.set_label_color(Color::from_rgb(150, 150, 160));
        label_out.set_label_size(11);
        label_out.set_align(Align::Right | Align::Inside);

        let out_point_input = FloatInput::default()
            .with_size(field_w, field_h)
            .with_pos(margin + label_w, out_y);

        group.add(&title);
        group.add(&info);
        group.add(&pos_label);
        group.add(&pos_slider);
        group.add(&label_start);
        group.add(&start_input);
        group.add(&label_dur);
        group.add(&duration_input);
        group.add(&label_in);
        group.add(&in_point_input);
        group.add(&label_out);
        group.add(&out_point_input);

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

    /// 選択されたクリップの情報でパネルを更新
    pub fn update_from_selection(&mut self) {
        let state = self.state.borrow();
        let dur = state.project.duration;

        // 再生位置更新
        self.position_slider.set_maximum(dur.max(1.0));
        self.position_slider.set_value(state.playback_position);
        let pos_str = crate::utils::time::format_duration(state.playback_position);
        self.position_label.set_label(&format!("再生位置: {}", pos_str));

        if let Some(clip) = state.selected_clip() {
            let info_text = format!(
                "クリップ: {}\nタイプ: {} | トラック: {}",
                clip.label,
                clip.media_type.label(),
                clip.track_idx + 1
            );
            self.info_frame.set_label(&info_text);
            self.info_frame.set_label_color(Color::from_rgb(200, 200, 210));

            self.start_input.set_value(&format!("{:.2}", clip.start_time));
            self.duration_input.set_value(&format!("{:.2}", clip.duration));
            self.in_point_input.set_value(&format!("{:.2}", clip.in_point));
            self.out_point_input.set_value(&format!("{:.2}", clip.out_point));
        } else {
            self.info_frame.set_label("クリップが選択されていません");
            self.info_frame.set_label_color(Color::from_rgb(150, 150, 160));
            self.start_input.set_value("");
            self.duration_input.set_value("");
            self.in_point_input.set_value("");
            self.out_point_input.set_value("");
        }
    }

    /// プロパティの変更をクリップに反映
    pub fn apply_changes(&mut self) {
        let clip_id = match self.state.borrow().selected_clip_id {
            Some(id) => id,
            None => return,
        };

        let mut state = self.state.borrow_mut();

        if let Some(clip) = state.project.find_clip_mut(clip_id) {
            if let Ok(val) = self.start_input.value().parse::<f64>() {
                clip.start_time = val;
            }
            if let Ok(val) = self.duration_input.value().parse::<f64>() {
                clip.duration = val.max(0.01);
                clip.out_point = clip.in_point + clip.duration;
            }
            if let Ok(val) = self.in_point_input.value().parse::<f64>() {
                clip.in_point = val.max(0.0);
                clip.duration = clip.out_point - clip.in_point;
            }
            if let Ok(val) = self.out_point_input.value().parse::<f64>() {
                clip.out_point = val.max(clip.in_point);
                clip.duration = clip.out_point - clip.in_point;
            }
            state.project.update_duration();
            state.is_modified = true;
        }
    }
}