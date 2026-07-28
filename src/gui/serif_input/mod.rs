use fltk::{
    enums::{Color, FrameType, Font},
    group::Group,
    text::{TextBuffer, TextEditor},
    prelude::*,
};

/// セリフ入力パネル
pub struct SerifInputPanel {
    pub group: Group,
    editor: TextEditor,
    buffer: TextBuffer,
}

impl SerifInputPanel {
    /// 新しいセリフ入力パネルを作成
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut group = Group::default()
            .with_size(w, h)
            .with_pos(x, y);

        group.set_frame(FrameType::BorderBox);
        group.set_color(Color::from_rgb(38, 38, 43));

        // テキストエディタ
        let buffer = TextBuffer::default();
        let mut editor = TextEditor::default()
            .with_size(w - 10, h - 10)
            .with_pos(5, 5);

        editor.set_buffer(buffer.clone());
        editor.set_color(Color::from_rgb(45, 45, 50));
        editor.set_text_color(Color::from_rgb(220, 220, 230));
        editor.set_text_font(Font::Helvetica);
        editor.set_text_size(13);
        editor.set_cursor_color(Color::from_rgb(200, 200, 210));
        editor.set_frame(FrameType::FlatBox);

        // スクロールバーを有効化
        editor.set_scrollbar_size(10);

        // プレースホルダーテキスト
        let mut buf = buffer.clone();
        buf.set_text("ここにセリフを入力してください...\n\n・1行1セリフで記述します\n・タイムライン上の位置に対応します");

        group.add(&editor);
        group.end();

        Self {
            group,
            editor,
            buffer,
        }
    }

    /// テキストを取得
    pub fn get_text(&self) -> String {
        self.buffer.text()
    }

    /// テキストを設定
    pub fn set_text(&mut self, text: &str) {
        let mut buf = self.buffer.clone();
        buf.set_text(text);
    }

    /// テキストを追加
    pub fn append_text(&mut self, text: &str) {
        let mut buf = self.buffer.clone();
        buf.append(text);
    }

    /// テキストをクリア
    pub fn clear(&mut self) {
        let mut buf = self.buffer.clone();
        buf.set_text("");
    }

    /// エディタウィジェットへの参照
    pub fn editor(&self) -> &TextEditor {
        &self.editor
    }
}