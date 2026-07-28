use fltk::{
    enums::{Color, FrameType, Align, Font},
    frame::Frame,
    image::RgbImage,
    prelude::*,
};
use std::cell::RefCell;
use std::rc::Rc;

/// 動画プレビュー表示パネル
pub struct PreviewPanel {
    pub frame: Frame,
    /// プレビュー画像の現在の幅
    image_width: u32,
    /// プレビュー画像の現在の高さ
    image_height: u32,
    /// 現在表示中のRGBデータ（キャッシュ用）
    current_data: Rc<RefCell<Option<(Vec<u8>, u32, u32)>>>,
}

impl PreviewPanel {
    /// 新しいプレビューパネルを作成
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let current_data: Rc<RefCell<Option<(Vec<u8>, u32, u32)>>> =
            Rc::new(RefCell::new(None));

        let mut frame = Frame::default()
            .with_size(w, h)
            .with_pos(x, y);

        // 黒背景 + 枠線
        frame.set_color(Color::Black);
        frame.set_frame(FrameType::BorderBox);

        // プレビュー領域のラベル（空の状態）
        frame.set_label("プレビュー");
        frame.set_label_color(Color::from_rgb(80, 80, 80));
        frame.set_label_size(16);
        frame.set_align(Align::Center);

        Self {
            frame,
            image_width: 0,
            image_height: 0,
            current_data,
        }
    }

    /// RGBデータでプレビューを更新する
    pub fn update_frame(&mut self, rgb_data: &[u8], width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        // データをコピーしてキャッシュ
        let data_copy = rgb_data.to_vec();
        self.image_width = width;
        self.image_height = height;

        // 現在のフレームサイズに合わせて表示
        let fw = self.frame.width() as u32;
        let fh = self.frame.height() as u32;

        if fw == 0 || fh == 0 {
            return;
        }

        // fltkのRgbImageを作成して表示
        if let Ok(rgb) = RgbImage::new(
            &data_copy,
            width as i32,
            height as i32,
            fltk::enums::ColorDepth::Rgb8,
        ) {
            self.frame.set_image(Some(rgb));
        }

        // ラベルをクリア
        self.frame.set_label("");

        // キャッシュを更新
        if let Ok(mut cache) = self.current_data.try_borrow_mut() {
            *cache = Some((data_copy, width, height));
        }
    }

    /// プレビューをクリア（黒画面に戻す）
    pub fn clear(&mut self) {
        self.frame.set_image::<RgbImage>(None);
        self.frame.set_label("プレビュー");
        self.frame.redraw();

        if let Ok(mut cache) = self.current_data.try_borrow_mut() {
            *cache = None;
        }
    }

    /// 現在のプレビュー画像情報を取得
    pub fn image_dimensions(&self) -> (u32, u32) {
        (self.image_width, self.image_height)
    }

    /// プレビュー画像がセットされているか
    pub fn has_image(&self) -> bool {
        self.image_width > 0 && self.image_height > 0
    }
}