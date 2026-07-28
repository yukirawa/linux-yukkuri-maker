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
}

impl TimelineCanvas {
    pub fn new(x: i32, y: i32, w: i32, h: i32, state: Rc<RefCell<AppState>>) -> Self {
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

            // header
            draw::draw_rect_fill(x0, y0, ww, hdr_h, colors::HEADER_BG);

            // ruler
            let step = Self::ruler_step(ww as f64 / pps);
            let t0 = (scroll / step).floor() * step;
            let mut t = t0;
            while t <= scroll + ww as f64 / pps {
                let xx = x0 + ((t - scroll) * pps) as i32;
                draw::set_draw_color(Color::from_rgb(100,100,110));
                draw::draw_line(xx, y0 + hdr_h - 8, xx, y0 + hdr_h);
                draw::set_draw_color(Color::from_rgb(180,180,190));
                draw::set_font(Font::Helvetica, 10);
                let m = (t / 60.0) as i32;
                let s = (t % 60.0) as i32;
                draw::draw_text2(&format!("{}:{:02}", m, s), xx+2, y0+4, 0, 0, Align::Left);
                t += step;
            }

            // tracks
            for ti in 0..tracks {
                let ty = y0 + hdr_h + ti as i32 * track_h;
                let bg = if ti % 2 == 0 { colors::TRACK_ODD } else { colors::TRACK_EVEN };
                draw::draw_rect_fill(x0, ty, ww, track_h, bg);
                draw::set_draw_color(Color::from_rgb(60,60,65));
                draw::draw_line(x0, ty, x0 + ww, ty);
            }

            // clips
            for track in &s.project.tracks {
                if !track.visible { continue; }
                for clip in &track.clips {
                    let cx = x0 + ((clip.start_time - scroll) * pps) as i32;
                    let cw = ((clip.duration * pps) as i32).max(4);
                    let cy = y0 + hdr_h + track.index as i32 * track_h + 3;
                    let ch = track_h - 6;
                    if cx + cw < x0 || cx > x0 + ww { continue; }

                    let col = Color::from_u32(clip.media_type.default_color());
                    let fill = if s.selected_clip_id == Some(clip.id) {
                        Color::from_rgb(100,160,220)
                    } else { col };
                    draw::draw_rect_fill(cx, cy, cw, ch, fill);
                    draw::set_draw_color(Color::White);
                    draw::draw_rect(cx, cy, cw, ch);

                    if cw > 30 {
                        draw::set_draw_color(Color::White);
                        draw::set_font(Font::Helvetica, 10);
                        draw::draw_text2(&clip.label, cx+3, cy+2, cw-6, ch-4, Align::Left|Align::Clip);
                    }
                }
            }

            // playhead
            let head = s.playback_position;
            if head >= scroll && head <= scroll + ww as f64 / pps {
                let hx = x0 + ((head - scroll) * pps) as i32;
                draw::set_draw_color(colors::ACCENT_RED);
                draw::draw_line(hx, y0, hx, y0 + wh - sb_h);
                draw::draw_polygon(hx-5, y0+hdr_h-7, hx+5, y0+hdr_h-7, hx, y0+hdr_h);
            }

            // scrollbar
            let sy = y0 + wh - sb_h;
            draw::draw_rect_fill(x0, sy, ww, sb_h, Color::from_rgb(30,30,35));
            if s.project.duration > 0.0 {
                let total_px = s.project.duration * pps;
                let thumb_w = ((ww as f64 / total_px) * ww as f64).max(50.0);
                let thumb_x = x0 as f64 + (scroll / s.project.duration) * (ww as f64 - thumb_w);
                draw::draw_rect_fill(thumb_x as i32, sy+2, thumb_w as i32, sb_h-4, Color::from_rgb(100,100,110));
            }
        });

        // events
        let state_ev = state.clone();
        inner.handle(move |w, ev| match ev {
            Event::Push => {
                let s = match state_ev.try_borrow() { Ok(s) => s, Err(_) => return false };
                let mx = fltk::app::event_x() - w.x();
                let my = fltk::app::event_y() - w.y();
                let hdr_h = 24;
                let pps = s.zoom_level;
                let scroll = s.scroll_offset;

                if my < hdr_h {
                    let time = scroll + mx as f64 / pps;
                    drop(s);
                    if let Ok(mut s) = state_ev.try_borrow_mut() {
                        s.set_playback_position(time.max(0.0));
                    }
                    w.redraw();
                    return true;
                }

                let tracks = s.project.tracks.len().max(1);
                let sb_h = 16;
                let track_h = (w.height() - hdr_h - sb_h) / tracks as i32;
                let mut hit: Option<u64> = None;
                for track in &s.project.tracks {
                    for clip in &track.clips {
                        let cx = ((clip.start_time - scroll) * pps) as i32;
                        let cw = ((clip.duration * pps) as i32).max(4);
                        let cy = hdr_h + track.index as i32 * track_h + 3;
                        let ch = track_h - 6;
                        if mx >= cx && mx <= cx + cw && my >= cy && my <= cy + ch {
                            hit = Some(clip.id);
                            break;
                        }
                    }
                    if hit.is_some() { break; }
                }
                drop(s);
                if let Ok(mut s) = state_ev.try_borrow_mut() {
                    s.selected_clip_id = hit;
                }
                w.redraw();
                true
            }
            Event::MouseWheel => {
                if let Ok(mut s) = state_ev.try_borrow_mut() {
                    let dy = fltk::app::event_dy();
                    match dy {
                        fltk::app::MouseWheel::Up => s.zoom_level = (s.zoom_level * 1.15).min(500.0),
                        fltk::app::MouseWheel::Down => s.zoom_level = (s.zoom_level / 1.15).max(10.0),
                        fltk::app::MouseWheel::Left => s.scroll_offset = (s.scroll_offset - 10.0).max(0.0),
                        fltk::app::MouseWheel::Right => s.scroll_offset += 10.0,
                        _ => {}
                    }
                    w.redraw();
                }
                true
            }
            _ => false,
        });

        Self { inner, state }
    }

    fn ruler_step(vis: f64) -> f64 {
        for &s in &[0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0] {
            if s >= vis / 20.0 { return s; }
        }
        60.0
    }

    pub fn widget(&self) -> &Widget { &self.inner }
    pub fn widget_mut(&mut self) -> &mut Widget { &mut self.inner }
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
        group.set_color(Color::from_rgb(30,30,35));

        // header label
        let mut lbl = Frame::new(0, 0, 80, h, None);
        lbl.set_label("Tracks");
        lbl.set_label_color(Color::from_rgb(200,200,210));
        lbl.set_label_size(11);
        lbl.set_frame(FrameType::FlatBox);
        lbl.set_color(Color::from_rgb(25,25,30));
        lbl.set_align(Align::TopLeft | Align::Inside);

        let canvas = TimelineCanvas::new(82, 0, w - 84, h, state);

        group.add(canvas.widget());
        group.add(&lbl);
        group.end();

        Self { group, canvas }
    }

    pub fn add_clip(&mut self, label: &str, start: f64, dur: f64, mtype: MediaType, track: usize) {
        if let Ok(mut s) = self.canvas.state.try_borrow_mut() {
            s.project.add_clip(None, label, start, dur, mtype, track);
            s.is_modified = true;
        }
        self.canvas.widget_mut().redraw();
    }

    pub fn canvas_widget(&mut self) -> &mut Widget {
        self.canvas.widget_mut()
    }

    pub fn redraw(&mut self) {
        self.canvas.widget_mut().redraw();
    }
}