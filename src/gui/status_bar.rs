use fltk::{
    enums::{Color, Font, FrameType, Align},
    frame::Frame,
    group::Group,
    prelude::*,
};
use crate::gui::widgets::colors;

/// ステータスバー
pub struct StatusBar {
    pub group: Group,
    info_label: Frame,
    position_label: Frame,
    zoom_label: Frame,
}

impl StatusBar {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut group = Group::default()
            .with_size(w, h)
            .with_pos(x, y);
        group.set_color(colors::BG_PANEL);

        let part_w = w / 3;

        // プロジェクト情報
        let mut info = Frame::default()
            .with_size(part_w, h)
            .with_pos(0, 0);
        info.set_label("プロジェクト: 新規プロジェクト");
        info.set_label_color(Color::from_rgb(150, 150, 160));
        info.set_label_size(10);
        info.set_label_font(Font::Helvetica);
        info.set_align(Align::Left | Align::Inside);
        info.set_frame(FrameType::FlatBox);

        // 再生位置情報
        let mut pos = Frame::default()
            .with_size(part_w, h)
            .with_pos(part_w, 0);
        pos.set_label("0:00 / 0:00");
        pos.set_label_color(Color::from_rgb(150, 150, 160));
        pos.set_label_size(10);
        pos.set_label_font(Font::Helvetica);
        pos.set_align(Align::Center | Align::Inside);
        pos.set_frame(FrameType::FlatBox);

        // ズーム情報
        let mut zoom = Frame::default()
            .with_size(part_w, h)
            .with_pos(part_w * 2, 0);
        zoom.set_label("100%");
        zoom.set_label_color(Color::from_rgb(150, 150, 160));
        zoom.set_label_size(10);
        zoom.set_label_font(Font::Helvetica);
        zoom.set_align(Align::Right | Align::Inside);
        zoom.set_frame(FrameType::FlatBox);

        group.add(&info);
        group.add(&pos);
        group.add(&zoom);

        group.end();

        Self {
            group,
            info_label: info,
            position_label: pos,
            zoom_label: zoom,
        }
    }

    /// プロジェクト名更新
    pub fn set_project_name(&mut self, name: &str) {
        self.info_label.set_label(&format!("プロジェクト: {}", name));
    }

    /// 再生位置更新
    pub fn set_position(&mut self, current: f64, total: f64) {
        self.position_label.set_label(&format!(
            "{} / {}",
            crate::utils::time::format_duration(current),
            crate::utils::time::format_duration(total)
        ));
    }

    /// ズームレベル更新
    pub fn set_zoom(&mut self, level: f64) {
        self.zoom_label.set_label(&format!("{:.0}%", level));
    }
}