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

use crate::gui::state::{AppState, ClipData, MediaType};
use crate::gui::widgets::colors;

/// タイムラインクリップの描画用内部表現
#[derive(Clone)]
pub struct TimelineClip {
    pub label: String,
    pub start_time: f64,
    pub duration: f64,
    pub color: Color,
    pub track_idx: usize,
    pub clip_id: u64,
    pub media_type: MediaType,
    pub in_point: f64,
    pub out_point: f64,
}

/// タイムラインウィジェット（カスタム描画）
pub struct TimelineCanvas {
    inner: Widget,
    state: Rc<RefCell<AppState>>,
    hovered_clip_id: Rc<RefCell<Option<u64>>>,
}

impl TimelineCanvas {
    pub fn new(x: i32, y: i32, w: i32, h: i32, state: Rc<RefCell<AppState>>) -> Self {
        let hovered_clip_id: Rc<RefCell<Option<u64>>> = Rc::new(RefCell::new(None));

        let state_draw = state.clone();
        let hovered_draw = hovered_clip_id.clone();

        let mut inner = Widget::default()
            .with_size(w, h)
            .with_pos(x, y);
        inner.set_color(colors::TL_BG);

        inner.draw(move |w| {
            let s = match state_draw.try_borrow() {
                Ok(s) => s,
                Err(_) => return,
            };

            let x_pos = w.x();
            let y_pos = w.y();
            let width = w.width();
            let height = w.height();

            // 背景
            draw::draw_rect_fill(x_pos, y_pos, width, height, colors::TL_BG);

            // ヘッダー
            let header_h = 24;
            draw::draw_rect_fill(x_pos, y_pos, width, header_h, colors::HEADER_BG);

            // トラック数
            let track_count = s.project.tracks.len().max(1);
            let scrollbar_h = 16;
            let track_area_top = y_pos + header_h;
            let track_area_h = height - header_h - scrollbar_h;
            let track_h = (track_area_h / track_count as i32).max(20);

            let pps = s.zoom_level;
            let scroll = s.scroll_offset;

            // タイムライン目盛り
            let start_sec = scroll;
            let end_sec = scroll + (width as f64 / pps);

            draw::set_font(Font::Helvetica, 10);

            // 目盛り線
            let step = Self::calc_ruler_step(width as f64 / pps);
            let mut t = (start_sec / step).floor() * step;
            while t <= end_sec {
                let x = x_pos + ((t - start_sec) * pps) as i32;
                draw::set_draw_color(Color::from_rgb(100, 100, 110));
                draw::draw_line(x, y_pos + header_h - 8, x, y_pos + header_h);

                draw::set_draw_color(Color::from_rgb(180, 180, 190));
                let mins = (t / 60.0) as i32;
                let secs = (t % 60.0) as i32;
                let label = if mins > 59 {
                    let hrs = mins / 60;
                    let m = mins % 60;
                    format!("{}:{:02}:{:02}", hrs, m, secs)
                } else {
                    format!("{}:{:02}", mins, secs)
                };
                draw::draw_text2(&label, x + 2, y_pos + 4, 0, 0, Align::Left);
                t += step;
            }

            // トラック背景
            for track_idx in 0..track_count {
                let track_top = track_area_top + track_idx as i32 * track_h;
                let bg = if let Some(track) = s.project.tracks.get(track_idx) {
                    if track.locked {
                        Color::from_rgb(50, 35, 35)
                    } else if track_idx % 2 == 0 {
                        colors::TRACK_ODD
                    } else {
                        colors::TRACK_EVEN
                    }
                } else if track_idx % 2 == 0 {
                    colors::TRACK_ODD
                } else {
                    colors::TRACK_EVEN
                };
                draw::draw_rect_fill(x_pos, track_top, width, track_h, bg);

                // トラック区切り線
                draw::set_draw_color(Color::from_rgb(60, 60, 65));
                draw::draw_line(x_pos, track_top, x_pos + width, track_top);
            }

            // クリップ描画
            let hovered = *hovered_draw.borrow();
            for track in &s.project.tracks {
                if !track.visible {
                    continue;
                }
                for clip in &track.clips {
                    let clip_x = x_pos + ((clip.start_time - start_sec) * pps) as i32;
                    let clip_w = ((clip.duration * pps) as i32).max(4);
                    let clip_top = track_area_top + track.index as i32 * track_h + 3;
                    let clip_h = track_h - 6;

                    if clip_x + clip_w < x_pos || clip_x > x_pos + width {
                        continue;
                    }

                    let clip_color = Color::from_u32(clip.media_type.default_color());
                    let is_selected = s.selected_clip_id == Some(clip.id);
                    let is_hovered = hovered == Some(clip.id);

                    // クリップ本体
                    let fill_color = if is_selected {
                        Color::from_rgb(100, 160, 220)
                    } else if is_hovered {
                        Color::from_rgb(80, 140, 200)
                    } else {
                        clip_color
                    };
                    draw::draw_rect_fill(clip_x, clip_top, clip_w, clip_h, fill_color);

                    let border_color = if is_selected {
                        Color::White
                    } else if is_hovered {
                        Color::from_rgb(200, 200, 220)
                    } else {
                        Color::from_rgb(200, 200, 200)
                    };
                    draw::set_draw_color(border_color);
                    draw::draw_rect(clip_x, clip_top, clip_w, clip_h);

                    // クリップラベル
                    if clip_w > 30 {
                        draw::set_draw_color(if is_selected {
                            Color::White
                        } else {
                            Color::from_rgb(230, 230, 240)
                        });
                        draw::set_font(Font::Helvetica, 10);
                        draw::draw_text2(
                            &clip.label,
                            clip_x + 3,
                            clip_top + 2,
                            clip_w - 6,
                            clip_h - 4,
                            Align::Left | Align::Clip,
                        );
                    }
                }
            }

            // 再生ヘッド
            let head = s.playback_position;
            if head >= start_sec && head <= end_sec {
                let head_x = x_pos + ((head - start_sec) * pps) as i32;
                draw::set_draw_color(colors::ACCENT_RED);
                draw::draw_line(head_x, y_pos, head_x, y_pos + height - scrollbar_h);

                // 三角形マーカー
                draw::draw_polygon(
                    head_x - 5,
                    y_pos + header_h - 7,
                    head_x + 5,
                    y_pos + header_h - 7,
                    head_x,
                    y_pos + header_h,
                );
            }

            // スクロールバー領域
            let sb_y = y_pos + height - scrollbar_h;
            draw::draw_rect_fill(x_pos, sb_y, width, scrollbar_h, Color::from_rgb(30, 30, 35));
            draw::set_draw_color(Color::from_rgb(80, 80, 85));
            draw::draw_line(x_pos, sb_y, x_pos + width, sb_y);

            // スクロールつまみ
            if s.project.duration > 0.0 {
                let total_pixels = s.project.duration * pps;
                let thumb_width = ((width as f64 / total_pixels) * width as f64).max(50.0);
                let thumb_x =
                    x_pos as f64 + (scroll / s.project.duration) * (width as f64 - thumb_width);
                draw::draw_rect_fill(
                    thumb_x as i32,
                    sb_y + 2,
                    thumb_width as i32,
                    scrollbar_h - 4,
                    Color::from_rgb(100, 100, 110),
                );
            }
        });

        // イベント処理
        let state_event = state.clone();
        let hovered_event = hovered_clip_id.clone();

        inner.handle(move |w, ev| match ev {
            Event::Push => {
                let s = match state_event.try_borrow() {
                    Ok(s) => s,
                    Err(_) => return false,
                };

                let mouse_x = fltk::app::event_x() - w.x();
                let mouse_y = fltk::app::event_y() - w.y();

                let pps = s.zoom_level;
                let scroll = s.scroll_offset;
                let header_h = 24;

                // ヘッダークリック → 再生ヘッド移動
                if mouse_y >= 0 && mouse_y < header_h {
                    let time = scroll + mouse_x as f64 / pps;
                    drop(s);
                    if let Ok(mut s) = state_event.try_borrow_mut() {
                        s.set_playback_position(time.max(0.0));
                    }
                    w.redraw();
                    return true;
                }

                // クリップクリック検出
                let track_count = s.project.tracks.len().max(1);
                let scrollbar_h = 16;
                let track_area_top = header_h;
                let track_area_h = w.height() - header_h - scrollbar_h;
                let track_h = (track_area_h / track_count as i32).max(20);

                let mut clicked_clip_id = None;

                for track in &s.project.tracks {
                    if !track.visible {
                        continue;
                    }
                    for clip in &track.clips {
                        let clip_x = ((clip.start_time - scroll) * pps) as i32;
                        let clip_w = ((clip.duration * pps) as i32).max(4);
                        let clip_top = track_area_top + track.index as i32 * track_h + 3;
                        let clip_h = track_h - 6;

                        if mouse_x >= clip_x
                            && mouse_x <= clip_x + clip_w
                            && mouse_y >= clip_top
                            && mouse_y <= clip_top + clip_h
                        {
                            clicked_clip_id = Some(clip.id);
                            break;
                        }
                    }
                    if clicked_clip_id.is_some() {
                        break;
                    }
                }

                drop(s);
                if let Ok(mut s) = state_event.try_borrow_mut() {
                    s.selected_clip_id = clicked_clip_id;
                }
                w.redraw();
                true
            }
            Event::MouseWheel => {
                if let Ok(mut s) = state_event.try_borrow_mut() {
                    let dy = fltk::app::event_dy();
                    // Ctrlキーで水平スクロール、それ以外はズーム
                    let ctrl = fltk::app::event_state().bits() & 4 != 0; // Ctrl

                    match dy {
                        fltk::app::MouseWheel::Up => {
                            if ctrl {
                                s.scroll_offset = (s.scroll_offset - 1.0).max(0.0);
                            } else {
                                s.zoom_level = (s.zoom_level * 1.15).min(500.0);
                            }
                        }
                        fltk::app::MouseWheel::Down => {
                            if ctrl {
                                s.scroll_offset += 1.0;
                            } else {
                                s.zoom_level = (s.zoom_level / 1.15).max(10.0);
                            }
                        }
                        fltk::app::MouseWheel::Left => {
                            s.scroll_offset = (s.scroll_offset - 10.0).max(0.0);
                        }
                        fltk::app::MouseWheel::Right => {
                            s.scroll_offset += 10.0;
                        }
                        _ => {}
                    }
                    w.redraw();
                }
                true
            }
            Event::Move => {
                let s = match state_event.try_borrow() {
                    Ok(s) => s,
                    Err(_) => return false,
                };

                let mouse_x = fltk::app::event_x() - w.x();
                let mouse_y = fltk::app::event_y() - w.y();

                let pps = s.zoom_level;
                let scroll = s.scroll_offset;
                let header_h = 24;
                let track_count = s.project.tracks.len().max(1);
                let scrollbar_h = 16;
                let track_area_top = header_h;
                let track_area_h = w.height() - header_h - scrollbar_h;
                let track_h = (track_area_h / track_count as i32).max(20);

                let mut new_hovered: Option<u64> = None;

                for track in &s.project.tracks {
                    if !track.visible {
                        continue;
                    }
                    for clip in &track.clips {
                        let clip_x = ((clip.start_time - scroll) * pps) as i32;
                        let clip_w = ((clip.duration * pps) as i32).max(4);
                        let clip_top = track_area_top + track.index as i32 * track_h + 3;
                        let clip_h = track_h - 6;

                        if mouse_x >= clip_x
                            && mouse_x <= clip_x + clip_w
                            && mouse_y >= clip_top
                            && mouse_y <= clip_top + clip_h
                        {
                            new_hovered = Some(clip.id);
                            break;
                        }
                    }
                    if new_hovered.is_some() {
                        break;
                    }
                }

                drop(s);
                if let Ok(mut h) = hovered_event.try_borrow_mut() {
                    if *h != new_hovered {
                        *h = new_hovered;
                        w.redraw();
                    }
                }
                true
            }
            Event::Leave => {
                if let Ok(mut h) = hovered_event.try_borrow_mut() {
                    *h = None;
                }
                w.redraw();
                true
            }
            _ => false,
        });

        Self {
            inner,
            state,
            hovered_clip_id,
        }
    }

    /// 適切なルーラーステップを計算
    fn calc_ruler_step(visible_duration: f64) -> f64 {
        let steps = [0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0];
        let max_ticks = 20.0;
        let ideal_step = visible_duration / max_ticks;
        steps
            .iter()
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
    state: Rc<RefCell<AppState>>,
    header_frame: Frame,
}

impl TimelinePanel {
    pub fn new(x: i32, y: i32, w: i32, h: i32, state: Rc<RefCell<AppState>>) -> Self {
        let mut group = Group::default()
            .with_size(w, h)
            .with_pos(x, y);

        group.set_frame(FrameType::BorderBox);
        group.set_color(Color::from_rgb(30, 30, 35));

        // トラック名表示エリア
        let header_w = 80;
        let mut header_frame = Frame::default()
            .with_size(header_w, h)
            .with_pos(0, 0);
        header_frame.set_label("タイムライン");
        header_frame.set_label_color(Color::from_rgb(200, 200, 210));
        header_frame.set_label_size(11);
        header_frame.set_frame(FrameType::FlatBox);
        header_frame.set_color(Color::from_rgb(25, 25, 30));
        header_frame.set_align(Align::TopLeft | Align::Inside);

        // タイムラインキャンバス
        let canvas = TimelineCanvas::new(header_w + 2, 0, w - header_w - 4, h, state.clone());

        group.add(canvas.widget());
        group.add(&header_frame);
        group.end();

        Self {
            group,
            canvas,
            state,
            header_frame,
        }
    }

    /// クリップを追加
    pub fn add_clip(
        &mut self,
        label: &str,
        start_time: f64,
        duration: f64,
        media_type: MediaType,
        track_idx: usize,
    ) {
        if let Ok(mut s) = self.state.try_borrow_mut() {
            s.project.add_clip(
                None,
                label,
                start_time,
                duration,
                media_type,
                track_idx,
            );
            s.is_modified = true;
        }
        self.canvas.widget_mut().redraw();
    }

    /// 再生ヘッド位置を設定
    pub fn set_playback_head(&mut self, time: f64) {
        if let Ok(mut s) = self.state.try_borrow_mut() {
            s.set_playback_position(time);
        }
        self.canvas.widget_mut().redraw();
    }

    /// タイムラインを再描画
    pub fn redraw(&mut self) {
        self.canvas.widget_mut().redraw();
    }

    /// ヘッダーのトラックラベルを更新
    pub fn update_track_labels(&mut self) {
        self.canvas.widget_mut().redraw();
    }
}