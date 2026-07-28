use fltk::{
    enums::{Color, Font, FrameType, Align},
    frame::Frame,
    group::{Group, Scroll},
    prelude::*,
};

/// メディアブラウザパネル
pub struct MediaBrowserPanel {
    pub group: Group,
    scroll: Scroll,
}

impl MediaBrowserPanel {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut group = Group::default()
            .with_size(w, h)
            .with_pos(x, y);

        group.set_frame(FrameType::BorderBox);
        group.set_color(Color::from_rgb(38, 38, 43));

        // タイトル
        let mut title = Frame::default()
            .with_size(w - 8, 22)
            .with_pos(4, 4);
        title.set_label("メディアブラウザ");
        title.set_label_color(Color::from_rgb(200, 200, 210));
        title.set_label_font(Font::HelveticaBold);
        title.set_label_size(12);
        title.set_align(Align::Left | Align::Inside);
        title.set_frame(FrameType::FlatBox);
        title.set_color(Color::from_rgb(30, 30, 35));

        // スクロール領域
        let mut scroll = Scroll::default()
            .with_size(w - 4, h - 30)
            .with_pos(2, 28);
        scroll.set_frame(FrameType::FlatBox);
        scroll.set_color(Color::from_rgb(40, 40, 45));
        scroll.end();

        // プレースホルダー
        let mut placeholder = Frame::default()
            .with_size(w - 8, 40)
            .with_pos(4, 28);
        placeholder.set_label("ファイルをドロップしてください");
        placeholder.set_label_color(Color::from_rgb(100, 100, 110));
        placeholder.set_label_size(11);
        placeholder.set_align(Align::Center | Align::Inside);
        placeholder.set_frame(FrameType::FlatBox);
        placeholder.set_color(Color::from_rgb(40, 40, 45));

        group.add(&title);
        group.add(&placeholder);

        group.end();

        Self { group, scroll }
    }

    /// メディアリストをクリアして再構築
    pub fn clear_items(&mut self) {
        self.scroll.clear();
    }

    /// アイテムを追加（プレースホルダー）
    pub fn add_item(&mut self, name: &str, duration: f64) {
        // 将来的に動的なアイテムリストを構築
    }
}