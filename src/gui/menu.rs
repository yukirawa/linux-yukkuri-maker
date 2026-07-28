use fltk::{
    enums::{Color, Font, Shortcut},
    menu::MenuBar,
    prelude::*,
};
use std::cell::RefCell;
use std::rc::Rc;
use crate::gui::state::AppState;

pub struct AppMenu {
    pub menu_bar: MenuBar,
}

impl AppMenu {
    pub fn new(w: i32, h: i32) -> Self {
        let mut menu_bar = MenuBar::default()
            .with_size(w, h)
            .with_pos(0, 0);

        menu_bar.set_color(Color::from_rgb(40, 40, 45));
        menu_bar.set_text_color(Color::from_rgb(200, 200, 210));
        menu_bar.set_text_font(Font::Helvetica);
        menu_bar.set_text_size(13);

        Self { menu_bar }
    }

    pub fn setup_menus(
        &mut self,
        state: Rc<RefCell<AppState>>,
        extra_width: i32,
    ) {
        self.menu_bar.resize(0, 0, extra_width.max(400), 28);
        self.menu_bar.clear();

        // ファイルメニュー
        let s1 = state.clone();
        self.menu_bar.add(
            "ファイル/新規プロジェクト\t",
            Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Ok(mut s) = s1.try_borrow_mut() {
                    s.project = crate::gui::state::Project::new("新規プロジェクト");
                    s.playback_position = 0.0;
                    s.selected_clip_id = None;
                    s.is_modified = false;
                    s.project_file_path = None;
                    println!("新規プロジェクトを作成しました");
                }
            },
        );

        self.menu_bar.add(
            "ファイル/プロジェクトを開く...\t",
            Shortcut::Ctrl | 'o',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                println!("プロジェクトを開く: 未実装");
            },
        );

        let s3 = state.clone();
        self.menu_bar.add(
            "ファイル/プロジェクトを保存\t",
            Shortcut::Ctrl | 's',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Ok(s) = s3.try_borrow() {
                    if let Some(ref path) = s.project_file_path {
                        if let Err(e) = s.project.save_to_file(&path.to_string_lossy()) {
                            eprintln!("保存エラー: {}", e);
                        } else {
                            println!("プロジェクトを保存しました");
                        }
                    } else {
                        println!("名前を付けて保存してください (Shift+Ctrl+S)");
                    }
                }
            },
        );

        self.menu_bar.add(
            "ファイル/名前を付けて保存...\t",
            Shortcut::Ctrl | Shortcut::Shift | 's',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                println!("名前を付けて保存: ファイルダイアログ未実装");
            },
        );

        self.menu_bar.add(
            "ファイル/エクスポート/動画を書き出し...\t",
            Shortcut::Ctrl | 'e',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                println!("エクスポート: 未実装");
            },
        );

        self.menu_bar.add(
            "ファイル/終了\t",
            Shortcut::Ctrl | 'q',
            fltk::menu::MenuFlag::Normal,
            |_| {
                std::process::exit(0);
            },
        );

        // 編集メニュー
        self.menu_bar.add(
            "編集/元に戻す\t",
            Shortcut::Ctrl | 'z',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                println!("元に戻す: 未実装");
            },
        );

        self.menu_bar.add(
            "編集/やり直し\t",
            Shortcut::Ctrl | 'y',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                println!("やり直し: 未実装");
            },
        );

        self.menu_bar.add(
            "編集/削除\t",
            Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            move |_| {
                println!("削除: 未実装");
            },
        );

        self.menu_bar.add(
            "編集/分割\t",
            Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            move |_| {
                println!("分割: 未実装");
            },
        );

        // トラックメニュー
        let s5 = state.clone();
        self.menu_bar.add(
            "トラック/トラックを追加\t",
            Shortcut::Ctrl | 't',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                if let Ok(mut s) = s5.try_borrow_mut() {
                    let idx = s.project.tracks.len() + 1;
                    s.project.add_track(&format!("トラック {}", idx));
                    s.is_modified = true;
                    println!("トラックを追加しました: トラック {}", idx);
                }
            },
        );

        // 表示メニュー
        self.menu_bar.add(
            "表示/ズームイン\t",
            Shortcut::Ctrl | '=',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                println!("ズームイン: 未実装");
            },
        );

        self.menu_bar.add(
            "表示/ズームアウト\t",
            Shortcut::Ctrl | '-',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                println!("ズームアウト: 未実装");
            },
        );

        self.menu_bar.add(
            "表示/全体を表示\t",
            Shortcut::Ctrl | Shortcut::Shift | 'f',
            fltk::menu::MenuFlag::Normal,
            move |_| {
                println!("全体を表示: 未実装");
            },
        );

        // ヘルプメニュー
        self.menu_bar.add(
            "ヘルプ/LYMについて\t",
            Shortcut::None,
            fltk::menu::MenuFlag::Normal,
            |_| {
                println!("Linux Yukkuri Maker - LYM");
                println!("ゆっくり動画編集ソフトウェア");
            },
        );
    }
}