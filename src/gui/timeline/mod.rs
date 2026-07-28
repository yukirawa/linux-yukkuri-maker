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

use crate::gui::state::{AppState, MediaType};
use crate::gui::widgets::colors;

pub struct TimelineCanvas {
    inner: Widget,
    state: Rc<RefCell<AppState>>,
    dragging_clip_id: Rc<RefCell<Option<u64>>>,
    drag_start_x: Rc<RefCell<i32>>,
    drag_orig_start: Rc<RefCell<f64>>,
    trimming_edge: Rc<RefCell<Option<TrimEdge>>>,
}

#[derive(Clone, Copy, PartialEq)]
enum TrimEdge {
    Left,
    Right,
}

impl TimelineCanvas {
    pub fn new(x: i32, y: i32, w: i32, h: i32, state: Rc<RefCell<AppState>>) -> Self {
        let dragging_clip_id: Rc<RefCell<Option<u64>>> = Rc::new(RefCell::new(None));
        let drag_start_x: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));
        let drag_orig_start: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.0));
        let trimming_edge: Rc<RefCell<Option<TrimEdge>>> = Rc::new(RefCell::new(None));

        let state_draw = state.clone();
        let mut inner = Widget::default().with_size(w, h).with_pos(x, y);
        inner.set_color(colors::TL_BG);

        inner.draw(move |w| {
            let s = match state_draw.try_borrow() {
                Ok(s) => s,
                Err(_) => return,
            };
            let x0 = w.x();
            let y0 = w.y();
            let ww = w.width();
            let wh = w.height();

            draw::draw_rect_fill(x0, y0, ww, wh, colors::TL_BG);

            let hdr_h = 24;
            let sb_h = 16;
            let pps = s.zoom_level;
            let scroll = s.scroll_offset;
            let tracks = s.project.tracks.len().max(1);
            let track_area_h = wh - hdr_h - sb_h;
            let track_h = (track_area_h / tracks as i32).max(20);

            // Header bg
            draw::draw_rect_fill(x0, y0, ww, hdr_h, colors::HEADER_BG);

            // Ruler
            draw::set_font(Font::Helvetica, 10);
            let step = Self::ruler_step(ww as f64 / pps);
            let mut t = (scroll / step).floor() * step;
            while t <= scroll + ww as f64 / pps {
                let xx = x0 + ((t - scroll) * pps) as i32;
                draw::set_draw_color(Color::from_rgb(100, 100, 110));
                draw::draw_line(xx, y0 + hdr_h - 8, xx, y0 + hdr_h);
                draw::set_draw_color(Color::from_rgb(180, 180, 190));
                let m = (t / 60.0) as i32;
                let sec = (t % 60.0) as i32;
                draw::draw_text2(&format!("{}:{:02}", m, sec), xx + 2, y0 + 4, 0, 0, Align::Left);
                t += step;
            }

            // Track lanes
            for ti in 0..tracks {
                let ty = y0 + hdr_h + ti as i32 * track_h;
                let bg = if ti % 2 == 0 {
                    colors::TRACK_ODD
                } else {
                    colors::TRACK_EVEN
                };
                draw::draw_rect_fill(x0, ty, ww, track_h, bg);
                draw::set_draw_color(Color::from_rgb(60, 60, 65));
                draw::draw_line(x0, ty, x0 + ww, ty);
            }

            // Clips
            for track in &s.project.tracks {
                if !track.visible {
                    continue;
                }
                for clip in &track.clips {
                    let cx = x0 + ((clip.start_time - scroll) * pps) as i32;
                    let cw = ((clip.duration * pps) as i32).max(4);
                    let cy = y0 + hdr_h + track.index as i32 * track_h + 3;
                    let ch = track_h - 6;

                    if cx + cw < x0 || cx > x0 + ww {
                        continue;
                    }

                    let base_color = clip.media_type.default_color();
                    let fill = if s.selected_clip_id == Some(clip.id) {
                        Color::from_rgb(100, 160, 220)
                    } else {
                        Color::from_u32(base_color)
                    };
                    draw::draw_rect_fill(cx, cy, cw, ch, fill);
                    draw::set_draw_color(Color::White);
                    draw::draw_rect(cx, cy, cw, ch);

                    if cw > 30 {
                        draw::set_draw_color(Color::White);
                        draw::set_font(Font::Helvetica, 10);
                        draw::draw_text2(
                            &clip.label,
                            cx + 3,
                            cy + 2,
                            cw - 6,
                            ch - 4,
                            Align::Left | Align::Clip,
                        );
                    }
                }
            }

            // Playhead
            let head = s.playback_position;
            if head >= scroll && head <= scroll + ww as f64 / pps {
                let hx = x0 + ((head - scroll) * pps) as i32;
                draw::set_draw_color(colors::ACCENT_RED);
                draw::draw_line(hx, y0, hx, y0 + wh - sb_h);
                draw::draw_polygon(hx - 5, y0 + hdr_h - 7, hx + 5, y0 + hdr_h - 7, hx, y0 + hdr_h);
            }

            // Scrollbar
            let sy = y0 + wh - sb_h;
            draw::draw_rect_fill(x0, sy, ww, sb_h, Color::from_rgb(30, 30, 35));
            if s.project.duration > 0.0 {
                let total_px = s.project.duration * pps;
                let thumb_w = ((ww as f64 / total_px) * ww as f64).max(50.0);
                let thumb_x = x0 as f64 + (scroll / s.project.duration) * (ww as f64 - thumb_w);
                draw::draw_rect_fill(
                    thumb_x as i32,
                    sy + 2,
                    thumb_w as i32,
                    sb_h - 4,
                    Color::from_rgb(100, 100, 110),
                );
            }
        });

        // Events
        let state_ev = state.clone();
        let drag_id = dragging_clip_id.clone();
        let drag_x = drag_start_x.clone();
        let drag_orig = drag_orig_start.clone();
        let trim_edge = trimming_edge.clone();

        inner.handle(move |w, ev| {
            let s = match state_ev.try_borrow() {
                Ok(s) => s,
                Err(_) => return false,
            };
            let mx = fltk::app::event_x() - w.x();
            let my = fltk::app::event_y() - w.y();
            let hdr_h = 24;
            let sb_h = 16;
            let pps = s.zoom_level;
            let scroll = s.scroll_offset;
            let tracks = s.project.tracks.len().max(1);
            let track_h = (w.height() - hdr_h - sb_h) / tracks as i32;

            let find_clip_at = |x: i32, y: i32| -> (Option<u64>, Option<TrimEdge>) {
                for track in &s.project.tracks {
                    for clip in &track.clips {
                        let cx = ((clip.start_time - scroll) * pps) as i32;
                        let cw = ((clip.duration * pps) as i32).max(6);
                        let cy = hdr_h + track.index as i32 * track_h + 3;
                        let ch = track_h - 6;
                        if y >= cy && y <= cy + ch {
                            if x >= cx && x <= cx + cw {
                                // Check trim edges (8px edge zones)
                                if x <= cx + 8 && cw > 30 {
                                    return (Some(clip.id), Some(TrimEdge::Left));
                                }
                                if x >= cx + cw - 8 && cw > 30 {
                                    return (Some(clip.id), Some(TrimEdge::Right));
                                }
                                return (Some(clip.id), None);
                            }
                        }
                    }
                }
                (None, None)
            };

            match ev {
                Event::Push => {
                    if my < hdr_h {
                        let time = scroll + mx as f64 / pps;
                        drop(s);
                        if let Ok(mut s) = state_ev.try_borrow_mut() {
                            s.set_playback_position(time.max(0.0));
                        }
                        w.redraw();
                        return true;
                    }

                    let (hit, edge) = find_clip_at(mx, my);
                    if let Some(id) = hit {
                        drop(s);
                        if let Ok(mut s) = state_ev.try_borrow_mut() {
                            s.selected_clip_id = Some(id);
                        }
                        *drag_id.borrow_mut() = Some(id);
                        *drag_x.borrow_mut() = mx;
                        *trim_edge.borrow_mut() = edge;
                        if let Ok(s) = state_ev.try_borrow() {
                            if let Some(clip) = s.project.find_clip(id) {
                                *drag_orig.borrow_mut() = clip.start_time;
                            }
                        }
                    } else {
                        drop(s);
                        if let Ok(mut s) = state_ev.try_borrow_mut() {
                            s.selected_clip_id = None;
                        }
                    }
                    w.redraw();
                    true
                }
                Event::Drag => {
                    if let Some(id) = *drag_id.borrow() {
                        let edge = *trim_edge.borrow();
                        let dx = (mx - *drag_x.borrow()) as f64 / pps;
                        drop(s);
                        if let Ok(mut s) = state_ev.try_borrow_mut() {
                            if let Some(clip) = s.project.find_clip_mut(id) {
                                match edge {
                                    Some(TrimEdge::Left) => {
                                        let new_start = (*drag_orig.borrow() + dx).max(0.0);
                                        if new_start < clip.start_time + clip.duration - 0.1 {
                                            let shift = new_start - clip.start_time;
                                            clip.start_time = new_start;
                                            clip.in_point += shift;
                                            clip.duration -= shift;
                                            if clip.duration < 0.1 {
                                                clip.duration = 0.1;
                                            }
                                        }
                                    }
                                    Some(TrimEdge::Right) => {
                                        let new_dur = (clip.duration + dx).max(0.1);
                                        clip.duration = new_dur;
                                        clip.out_point = clip.in_point + new_dur;
                                    }
                                    None => {
                                        let new_start = (*drag_orig.borrow() + dx).max(0.0);
                                        clip.start_time = new_start;
                                    }
                                }
                                s.project.update_duration();
                                s.is_modified = true;
                            }
                        }
                        w.redraw();
                        return true;
                    }
                    false
                }
                Event::Released => {
                    *drag_id.borrow_mut() = None;
                    *trim_edge.borrow_mut() = None;
                    false
                }
                Event::MouseWheel => {
                    drop(s);
                    if let Ok(mut s) = state_ev.try_borrow_mut() {
                        match fltk::app::event_dy() {
                            fltk::app::MouseWheel::Up => {
                                s.zoom_level = (s.zoom_level * 1.15).min(500.0);
                            }
                            fltk::app::MouseWheel::Down => {
                                s.zoom_level = (s.zoom_level / 1.15).max(10.0);
                            }
                            fltk::app::MouseWheel::Left => {
                                s.scroll_offset = (s.scroll_offset - 30.0).max(0.0);
                            }
                            fltk::app::MouseWheel::Right => {
                                s.scroll_offset += 30.0;
                            }
                            _ => {}
                        }
                        w.redraw();
                    }
                    true
                }
                _ => false,
            }
        });

        Self {
            inner,
            state,
            dragging_clip_id,
            drag_start_x,
            drag_orig_start,
            trimming_edge,
        }
    }

    fn ruler_step(vis: f64) -> f64 {
        for &s in &[
            0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0,
        ] {
            if s >= vis / 20.0 {
                return s;
            }
        }
        60.0
    }

    pub fn widget(&self) -> &Widget {
        &self.inner
    }
    pub fn widget_mut(&mut self) -> &mut Widget {
        &mut self.inner
    }
}

// -------------------------------------------------------
// TimelinePanel
// -------------------------------------------------------
pub struct TimelinePanel {
    pub group: Group,
    canvas: TimelineCanvas,
}

impl TimelinePanel {
    pub fn new(x: i32, y: i32, w: i32, h: i32, state: Rc<RefCell<AppState>>) -> Self {
        let mut group = Group::default().with_size(w, h).with_pos(x, y);
        group.set_frame(FrameType::BorderBox);
        group.set_color(Color::from_rgb(30, 30, 35));
        group.begin();

        let mut lbl = Frame::new(0, 0, 80, h, None);
        lbl.set_label("Tracks");
        lbl.set_label_color(Color::from_rgb(200, 200, 210));
        lbl.set_label_size(11);
        lbl.set_frame(FrameType::FlatBox);
        lbl.set_color(Color::from_rgb(25, 25, 30));
        lbl.set_align(Align::TopLeft | Align::Inside);

        let canvas = TimelineCanvas::new(82, 0, w - 84, h, state);

        group.end();

        Self { group, canvas }
    }

    pub fn redraw(&mut self) {
        self.canvas.widget_mut().redraw();
    }

    pub fn widget(&self) -> &Widget {
        self.canvas.widget()
    }
}