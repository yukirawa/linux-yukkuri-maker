use fltk::{
    draw,
    enums::{Color, Event, Font, FrameType, Align},
    frame::Frame,
    group::Group,
    prelude::*,
    widget::Widget,
};
use std::cell::RefCell;
use std::rc::Rc;

/// タイムラインクリップの内部表現
#[derive(Clone)]
pub struct TimelineClip {
    pub label: String,
    pub start_time: f64,
    pub duration: f64,
    pub color: Color,
    pub track_idx: usize,
}

/// タイムラインウィジェット（カスタム描画）
pub struct TimelineCanvas {
    inner: Widget,
    clips: Rc<RefCell<Vec<TimelineClip>>>,
    scroll_offset: Rc<RefCell<f64>>,
    playback_head: Rc<RefCell<f64>>,
    pixels_per_second: f64,
    track_count: usize,
}

impl TimelineCanvas {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let clips = Rc::new(RefCell::new(Vec::<TimelineClip>::new()));
        let scroll_offset = Rc::new(RefCell::new(0.0));
        let playback_head = Rc::new(RefCell::new(0.0));

        let clips_draw = clips.clone();
        let scroll_draw = scroll_offset.clone();
        let head_draw = playback_head.clone();

        let mut inner = Widget::default()
            .with_size(w, h)
            .with_pos(x, y);
        inner.set_color(Color::from_rgb(35, 35, 40));

        inner.draw(move |w| {
            let clips = clips_draw.borrow();
            let scroll = *scroll_draw.borrow();
            let head = *head_draw.borrow();

            let x_pos = w.x();
            let y_pos = w.y();
            let width = w.width();
            let height = w.height();

            // 背景
            draw::draw_rect_fill(x_pos, y_pos, width, height, Color::from_rgb(35, 35, 40));

            // タイムライン領域（上部にヘッダー30px、残りをトラック領域）
            let header_h = 30;
            let track_area_top = y_pos + header_h;
            let track_area_h = height - header_h - 20; // スクロールバー用の余白
            let track_h = if track_area_h > 0 { track_area_h / 4.max(1) as i32 } else { 40 };

            let pps = 100.0; // pixels per second
            let total_duration = clips.iter()
                .map(|c| c.start_time + c.duration)
                .fold(0.0f64, f64::max);

            // タイムライン目盛り（ヘッダー部分）
            let start_sec = scroll;
            let end_sec = scroll + (width as f64 / pps);

            draw::draw_rect_fill(x_pos, y_pos, width, header_h, Color::from_rgb(50, 50, 55));

            // 目盛り線と数値
            draw::set_font(Font::Helvetica, 10);
            let step = Self::calc_ruler_step(width as f64 / pps);
            let mut t = (start_sec / step).floor() * step;
            while t <= end_sec {
                let x = x_pos + ((t - start_sec) * pps) as i32;

                // 目盛り線
                draw::set_draw_color(Color::from_rgb(100, 100, 110));
                draw::draw_line(x, y_pos + header_h - 10, x, y_pos + header_h);

                // 時間ラベル
                draw::set_draw_color(Color::from_rgb(180, 180, 190));
                let mins = (t / 60.0) as i32;
                let secs = (t % 60.0) as i32;
                draw::draw_text2(
                    &format!("{}:{:02}", mins, secs),
                    x + 2,
                    y_pos + 4,
                    0,
                    0,
                    Align::Left,
                );
                t += step;
            }

            // トラック背景（交互に色を変える）
            for track_idx in 0..4 {
                let track_top = track_area_top + track_idx * track_h;
                let bg = if track_idx % 2 == 0 {
                    Color::from_rgb(40, 40, 45)
                } else {
                    Color::from_rgb(45, 45, 50)
                };
                draw::draw_rect_fill(x_pos, track_top, width, track_h, bg);

                // トラック区切り線
                draw::set_draw_color(Color::from_rgb(60, 60, 65));
                draw::draw_line(x_pos, track_top, x_pos + width, track_top);
            }

            // クリップ描画
            for clip in clips.iter() {
                let clip_x = x_pos + ((clip.start_time - start_sec) * pps) as i32;
                let clip_w = ((clip.duration * pps) as i32).max(4);
                let clip_top = track_area_top + clip.track_idx as i32 * track_h + 4;
                let clip_h = track_h - 8;

                // 画面外のクリップはスキップ
                if clip_x + clip_w < x_pos || clip_x > x_pos + width {
                    continue;
                }

                // クリップ本体
                draw::draw_rect_fill(clip_x, clip_top, clip_w, clip_h, clip.color);
                draw::set_draw_color(Color::from_rgb(255, 255, 255));
                draw::draw_rect(clip_x, clip_top, clip_w, clip_h);

                // クリップラベル（クリップの幅が十分ある場合のみ）
                if clip_w > 30 {
                    draw::set_draw_color(Color::White);
                    draw::set_font(Font::Helvetica, 11);
                    draw::draw_text2(
                        &clip.label,
                        clip_x + 4,
                        clip_top + 2,
                        clip_w - 8,
                        clip_h - 4,
                        Align::Left,
                    );
                }
            }

            // 再生ヘッド（赤い縦線）
            if head >= start_sec && head <= end_sec {
                let head_x = x_pos + ((head - start_sec) * pps) as i32;
                draw::set_draw_color(Color::Red);
                draw::draw_line(head_x, y_pos, head_x, y_pos + height - 20);
                // 上部の三角形マーカー
                draw::draw_polygon(
                    head_x - 6, y_pos + header_h - 8,
                    head_x + 6, y_pos + header_h - 8,
                    head_x, y_pos + header_h,
                );
            }

            // スクロールバー領域
            let sb_y = y_pos + height - 20;
            draw::draw_rect_fill(x_pos, sb_y, width, 20, Color::from_rgb(30, 30, 35));
            draw::set_draw_color(Color::from_rgb(80, 80, 85));
            draw::draw_line(x_pos, sb_y, x_pos + width, sb_y);

            // スクロールつまみ
            if total_duration > 0.0 {
                let total_pixels = total_duration * pps;
                let thumb_width = ((width as f64 / total_pixels) * width as f64).max(50.0);
                let thumb_x = x_pos as f64 + (scroll / total_duration) * (width as f64 - thumb_width);
                draw::draw_rect_fill(
                    thumb_x as i32, sb_y + 2,
                    thumb_width as i32, 16,
                    Color::from_rgb(100, 100, 110),
                );
            }
        });

        // イベント処理
        let scroll_event = scroll_offset.clone();

        inner.handle(move |w, ev| {
            match ev {
                Event::Push => {
                    let _ = w; // 将来的にクリック位置からクリップ選択/ヘッド移動
                    true
                }
                Event::MouseWheel => {
                    // マウスホイールで水平スクロール
                    if let Ok(mut scroll) = scroll_event.try_borrow_mut() {
                        let mut delta = 0.0;
                        // event_dy() で上下スクロール量を取得
                        match fltk::app::event_dy() {
                            fltk::app::MouseWheel::Up => delta = -2.0,
                            fltk::app::MouseWheel::Down => delta = 2.0,
                            fltk::app::MouseWheel::Left => delta = -10.0,
                            fltk::app::MouseWheel::Right => delta = 10.0,
                            _ => {}
                        }
                        *scroll = (*scroll + delta).max(0.0);
                        w.redraw();
                    }
                    true
                }
                _ => false,
            }
        });

        Self {
            inner,
            clips,
            scroll_offset,
            playback_head,
            pixels_per_second: 100.0,
            track_count: 4,
        }
    }

    /// クリップを追加
    pub fn add_clip(&mut self, clip: TimelineClip) {
        if let Ok(mut clips) = self.clips.try_borrow_mut() {
            clips.push(clip);
        }
        self.inner.redraw();
    }

    /// クリップを全てクリア
    pub fn clear_clips(&mut self) {
        if let Ok(mut clips) = self.clips.try_borrow_mut() {
            clips.clear();
        }
        self.inner.redraw();
    }

    /// 再生ヘッド位置を設定（秒）
    pub fn set_playback_head(&mut self, time: f64) {
        if let Ok(mut head) = self.playback_head.try_borrow_mut() {
            *head = time;
        }
        self.inner.redraw();
    }

    /// 適切なルーラーステップを計算
    fn calc_ruler_step(visible_duration: f64) -> f64 {
        let steps = [0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0];
        let max_ticks = 20.0;
        let ideal_step = visible_duration / max_ticks;
        steps.iter()
            .find(|&&s| s >= ideal_step)
            .copied()
            .unwrap_or(60.0)
    }

    pub fn widget(&self) -> &Widget {
        &self.inner
    }

    pub fn widget_mut(&mut self) -> &mut Widget {
        &mut self.inner
    }
}

/// タイムラインパネル全体
pub struct TimelinePanel {
    pub group: Group,
    canvas: TimelineCanvas,
}

impl TimelinePanel {
    pub fn new(x: i32, y: i32, w: i32, h: i32, label: &str) -> Self {
        let mut group = Group::default()
            .with_size(w, h)
            .with_pos(x, y);

        group.set_frame(FrameType::BorderBox);
        group.set_color(Color::from_rgb(30, 30, 35));

        // トラックヘッダー
        let header_w = 80;
        let mut header_frame = Frame::default()
            .with_size(header_w, h)
            .with_pos(0, 0);
        header_frame.set_label(label);
        header_frame.set_label_color(Color::from_rgb(200, 200, 210));
        header_frame.set_label_size(12);
        header_frame.set_frame(FrameType::FlatBox);
        header_frame.set_color(Color::from_rgb(25, 25, 30));
        header_frame.set_align(Align::TopLeft | Align::Inside);

        // トラックラベル
        let track_labels = ["映像", "音声", "字幕", "効果音"];
        for (i, tl) in track_labels.iter().enumerate() {
            let y_off = 30 + i as i32 * ((h - 50) / 4.max(1) as i32);
            let mut label = Frame::default()
                .with_size(header_w - 4, 20)
                .with_pos(2, y_off);
            label.set_label(*tl);
            label.set_label_color(Color::from_rgb(150, 150, 160));
            label.set_label_size(10);
            label.set_frame(FrameType::FlatBox);
            if i % 2 == 0 {
                label.set_color(Color::from_rgb(38, 38, 43));
            } else {
                label.set_color(Color::from_rgb(43, 43, 48));
            }
            group.add(&label);
        }

        // タイムラインキャンバス
        let canvas = TimelineCanvas::new(header_w + 2, 0, w - header_w - 4, h);

        group.add(canvas.widget());
        group.end();

        Self { group, canvas }
    }

    pub fn add_clip(&mut self, label: &str, start_time: f64, duration: f64) {
        self.canvas.add_clip(TimelineClip {
            label: label.to_string(),
            start_time,
            duration,
            color: Color::from_rgb(70, 130, 180),
            track_idx: 0,
        });
    }

    pub fn set_playback_head(&mut self, time: f64) {
        self.canvas.set_playback_head(time);
    }
}