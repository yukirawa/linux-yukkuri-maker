use fltk::{
    enums::{Color, FrameType, Align, Font},
    frame::Frame,
    group::Group,
    prelude::*,
};

/// 共通ウィジェットユーティリティ
pub struct Widgets;

impl Widgets {
    /// ダークテーマのセパレーターラインを作成
    pub fn create_separator(x: i32, y: i32, w: i32, h: i32) -> Frame {
        let mut frame = Frame::default()
            .with_size(w, h)
            .with_pos(x, y);
        frame.set_frame(FrameType::FlatBox);
        frame.set_color(Color::from_rgb(60, 60, 65));
        frame
    }

    /// セクションラベルを作成
    pub fn create_section_label(x: i32, y: i32, w: i32, h: i32, text: &str) -> Frame {
        let mut frame = Frame::default()
            .with_size(w, h)
            .with_pos(x, y);
        frame.set_label(text);
        frame.set_label_color(Color::from_rgb(200, 200, 210));
        frame.set_label_size(12);
        frame.set_label_font(Font::HelveticaBold);
        frame.set_align(Align::Left | Align::Inside);
        frame.set_frame(FrameType::FlatBox);
        frame.set_color(Color::from_rgb(30, 30, 35));
        frame
    }

    /// ツールバーボタンを作成
    pub fn create_tool_button(
        x: i32, y: i32, w: i32, h: i32,
        label: &str, tooltip: &str,
    ) -> Frame {
        let mut frame = Frame::default()
            .with_size(w, h)
            .with_pos(x, y);
        frame.set_label(label);
        frame.set_label_color(Color::from_rgb(200, 200, 210));
        frame.set_label_size(11);
        frame.set_frame(FrameType::ThinUpBox);
        frame.set_color(Color::from_rgb(50, 50, 55));
        frame.set_tooltip(tooltip);
        frame
    }

    /// ステータスバーを作成
    pub fn create_status_bar(x: i32, y: i32, w: i32, h: i32) -> Frame {
        let mut frame = Frame::default()
            .with_size(w, h)
            .with_pos(x, y);
        frame.set_frame(FrameType::FlatBox);
        frame.set_color(Color::from_rgb(35, 35, 40));
        frame.set_label_color(Color::from_rgb(150, 150, 160));
        frame.set_label_size(11);
        frame.set_align(Align::Left | Align::Inside);
        frame.set_label("準備完了");
        frame
    }

    /// ダークテーマのグループを作成
    pub fn create_dark_group(x: i32, y: i32, w: i32, h: i32, label: &str) -> Group {
        let mut group = Group::default()
            .with_size(w, h)
            .with_pos(x, y);
        group.set_frame(FrameType::BorderBox);
        group.set_color(Color::from_rgb(38, 38, 43));
        group.set_label(label);
        group
    }
}

/// 色定数 - ダークテーマ
pub mod colors {
    use fltk::enums::Color;

    /// メイン背景色
    pub const BG_MAIN: Color = Color::from_u32(0x28282D);

    /// パネル背景色
    pub const BG_PANEL: Color = Color::from_u32(0x32323A);

    /// 入力エリア背景色
    pub const BG_INPUT: Color = Color::from_u32(0x2D2D35);

    /// テキスト色（通常）
    pub const TEXT_NORMAL: Color = Color::from_u32(0xDCDCE6);

    /// テキスト色（薄め）
    pub const TEXT_DIM: Color = Color::from_u32(0x9696A0);

    /// アクセントカラー（青系）
    pub const ACCENT_BLUE: Color = Color::from_u32(0x4682B4);

    /// アクセントカラー（赤系 - 再生ヘッドなど）
    pub const ACCENT_RED: Color = Color::from_u32(0xFF4040);

    /// アクセントカラー（緑系 - 成功表示）
    pub const ACCENT_GREEN: Color = Color::from_u32(0x40C040);

    /// 境界線色
    pub const BORDER: Color = Color::from_u32(0x50505A);
}

/// フォント定数
pub mod fonts {
    use fltk::enums::Font;

    /// UI用標準フォント
    pub const UI_NORMAL: Font = Font::Helvetica;

    /// UI用ボールドフォント
    pub const UI_BOLD: Font = Font::HelveticaBold;

    /// 等幅フォント
    pub const MONO: Font = Font::Courier;
}