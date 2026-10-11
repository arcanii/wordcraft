//! The caret glides to its new place instead of jumping there, like the caret in Word for
//! Windows: type, press an arrow key or click, and the bar slides over in about a tenth of a
//! second, staying lit while it moves.
//!
//! The behaviour comes from watching Word's caret; the timing and the easing curve are our own
//! estimate ([`GLIDE_SECS`]). `WORDCRAFT_CARET_GLIDE_MS` overrides the duration (0 turns the
//! glide off; a long one makes it easy to look at in `ui_shot`).
//!
//! What glides and what doesn't:
//! - the caret moved because the selection or the document changed (typing, arrows, a click,
//!   Enter, a formatting change that reflows the line): it glides from wherever it is drawn
//!   right now, so quick typing reads as one continuous motion;
//! - the caret moved because the view changed (zoom, a resized window, scrolling, another
//!   document): it stays glued to the text;
//! - the caret wasn't drawn last frame (a selection, the canvas unfocused), or either end of
//!   the move is off screen: it appears in place.
//!
//! The glide runs in content space (screen points from the top-left of the scrolled content), so
//! scrolling under a moving caret doesn't bend its path.

use std::hash::{Hash, Hasher};

use egui::{Pos2, Rect, Ui};

use crate::WordApp;
use crate::canvas::Geometry;

/// How long a glide takes, in seconds.
pub const GLIDE_SECS: f64 = 0.12;
/// The longest glide `WORDCRAFT_CARET_GLIDE_MS` can ask for.
const MAX_GLIDE_SECS: f64 = 10.0;
/// Moves shorter than this (screen points) aren't worth a glide.
const MIN_MOVE: f32 = 0.5;
/// How far outside the visible canvas a glide may start or end (screen points).
const OFFSCREEN_SLACK: f32 = 48.0;

/// The caret's bar: its two ends.
pub type Bar = [Pos2; 2];

/// What the canvas knows about the caret in one frame.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    /// Where the caret belongs, in content space.
    pub target: Bar,
    /// Fingerprint of what puts the caret there: the document's revision and the selection.
    pub cause: u64,
    /// Fingerprint of how the document is shown: zoom, page arrangement, which document.
    pub view: u64,
    /// The caret is drawn this frame (the canvas has focus and nothing is selected).
    pub shown: bool,
    /// The part of content space on screen.
    pub viewport: Rect,
    /// egui's clock, in seconds.
    pub now: f64,
    /// egui's frame counter: a caret that skipped a frame doesn't glide from its stale place.
    pub frame_nr: u64,
}

#[derive(Clone, Copy, Debug)]
struct Glide {
    from: Bar,
    to: Bar,
    started: f64,
}

/// The caret's motion between frames. Time-based, so it looks the same at any frame rate.
#[derive(Clone, Debug)]
pub struct CaretGlide {
    secs: f64,
    /// Where the caret was drawn last frame (content space); `None` when it wasn't drawn.
    drawn: Option<Bar>,
    glide: Option<Glide>,
    cause: u64,
    view: u64,
    frame_nr: u64,
}

impl Default for CaretGlide {
    fn default() -> Self {
        CaretGlide::with_secs(glide_secs())
    }
}

impl CaretGlide {
    /// A caret whose glides take `secs` seconds (0 or less: it always jumps).
    pub fn with_secs(secs: f64) -> Self {
        let secs = if secs.is_finite() { secs.clamp(0.0, MAX_GLIDE_SECS) } else { GLIDE_SECS };
        CaretGlide { secs, drawn: None, glide: None, cause: 0, view: 0, frame_nr: 0 }
    }

    /// Advance to this frame; returns where to draw the caret, in content space.
    pub fn frame(&mut self, f: Frame) -> Bar {
        let prev = self.drawn.filter(|_| f.frame_nr.wrapping_sub(self.frame_nr) <= 1);
        let edited = f.cause != self.cause;
        let same_view = f.view == self.view;
        self.cause = f.cause;
        self.view = f.view;
        self.frame_nr = f.frame_nr;
        let sane = is_finite(f.target) && f.now.is_finite();
        let prev = match prev {
            Some(prev) if f.shown && sane && same_view && self.secs > 0.0 => prev,
            _ => {
                self.glide = None;
                self.drawn = (f.shown && sane).then_some(f.target);
                return f.target;
            }
        };
        // A new destination: set off from where the caret is drawn now (mid-glide or at rest).
        let heading = self.glide.map_or(prev, |g| g.to);
        if edited && distance(heading, f.target) >= MIN_MOVE {
            let seen = f.viewport.expand(OFFSCREEN_SLACK);
            let on_screen = |b: Bar| seen.intersects(Rect::from_two_pos(b[0], b[1]));
            self.glide = (on_screen(prev) && on_screen(f.target)).then_some(Glide { from: prev, to: f.target, started: f.now });
        }
        // The text shifted under a glide (pixel snapping while scrolling): carry the glide along.
        if let Some(g) = &mut self.glide
            && g.to != f.target
        {
            let d = f.target[0] - g.to[0];
            g.from = [g.from[0] + d, g.from[1] + d];
            g.to = f.target;
        }
        let bar = match self.glide {
            Some(g) => {
                let t = (f.now - g.started) / self.secs;
                if (0.0..1.0).contains(&t) {
                    lerp(g.from, g.to, ease_out(t as f32))
                } else {
                    // Arrived (or the clock went backwards).
                    self.glide = None;
                    f.target
                }
            }
            None => f.target,
        };
        self.drawn = Some(bar);
        bar
    }

    /// The caret is on its way somewhere: keep repainting, and don't blink it.
    pub fn is_gliding(&self) -> bool {
        self.glide.is_some()
    }

    /// Where the caret was drawn last frame (content space), if it was drawn.
    pub fn drawn(&self) -> Option<Bar> {
        self.drawn
    }

    /// Forget the caret's place: it appears wherever it belongs next frame.
    pub fn reset(&mut self) {
        self.glide = None;
        self.drawn = None;
    }
}

/// Decelerating curve: quick off the mark, soft landing.
fn ease_out(t: f32) -> f32 {
    let r = 1.0 - t.clamp(0.0, 1.0);
    1.0 - r * r * r
}

fn lerp(a: Bar, b: Bar, t: f32) -> Bar {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

/// How far the bar's farthest end moves.
fn distance(a: Bar, b: Bar) -> f32 {
    (b[0] - a[0]).length().max((b[1] - a[1]).length())
}

fn is_finite(b: Bar) -> bool {
    b[0].x.is_finite() && b[0].y.is_finite() && b[1].x.is_finite() && b[1].y.is_finite()
}

/// The glide's duration: [`GLIDE_SECS`], or `WORDCRAFT_CARET_GLIDE_MS` when set (tests ignore it).
fn glide_secs() -> f64 {
    if cfg!(test) {
        return GLIDE_SECS;
    }
    static SECS: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    *SECS.get_or_init(|| parse_ms(std::env::var("WORDCRAFT_CARET_GLIDE_MS").ok().as_deref()))
}

fn parse_ms(value: Option<&str>) -> f64 {
    match value.and_then(|s| s.trim().parse::<f64>().ok()) {
        Some(ms) if ms.is_finite() => (ms / 1000.0).clamp(0.0, MAX_GLIDE_SECS),
        _ => GLIDE_SECS,
    }
}

fn state_id() -> egui::Id {
    egui::Id::new("aniefork.caret_glide")
}

/// What puts the caret where it is: the document's revision and the caret's position in it.
fn cause_key(app: &WordApp) -> u64 {
    let s = &app.session;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (s.rev(), &s.sel.focus).hash(&mut h);
    if let Some(m) = &s.math {
        format!("{:?}", m.pos).hash(&mut h);
    }
    h.finish()
}

/// How the document is shown: which document, the zoom and where the pages sit.
fn view_key(app: &WordApp, geo: &Geometry) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (app.session.document_id(), geo.scale.to_bits(), geo.cols, geo.size.x.to_bits()).hash(&mut h);
    geo.rects.first().map(|r| (r.min.x.to_bits(), r.min.y.to_bits())).hash(&mut h);
    h.finish()
}

/// The canvas's hook: `target` is where the caret belongs on screen this frame and `shown`
/// whether it is drawn at all. Returns the bar to draw and whether it is mid-glide (a gliding
/// caret stays lit). `content_origin` is the scrolled content's top-left on screen.
pub fn glide(app: &WordApp, ui: &Ui, content_origin: Pos2, geo: &Geometry, target: Bar, shown: bool) -> (Bar, bool) {
    let off = content_origin.to_vec2();
    // egui's own switch for "no animations".
    let animate = ui.style().animation_time > 0.0;
    let frame = Frame {
        target: [target[0] - off, target[1] - off],
        cause: cause_key(app),
        view: view_key(app, geo),
        shown: shown && animate,
        viewport: ui.clip_rect().translate(-off),
        now: ui.input(|i| i.time),
        frame_nr: ui.ctx().cumulative_frame_nr(),
    };
    let (bar, gliding) = ui.ctx().data_mut(|d| {
        let state = d.get_temp_mut_or_default::<CaretGlide>(state_id());
        let bar = state.frame(frame);
        (bar, state.is_gliding())
    });
    if !gliding {
        // At rest the caller's own pixel-snapped bar is drawn, untouched.
        return (target, false);
    }
    ui.ctx().request_repaint();
    ([bar[0] + off, bar[1] + off], true)
}

/// The caret's glide state this frame (for tests and tools).
pub fn state(ctx: &egui::Context) -> CaretGlide {
    ctx.data(|d| d.get_temp::<CaretGlide>(state_id())).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use egui::{pos2, vec2};
    use serde_json::json;

    use super::*;

    const DT: f64 = 1.0 / 60.0;

    fn bar(x: f32, y: f32) -> Bar {
        [pos2(x, y), pos2(x, y + 16.0)]
    }

    /// A caret driven frame by frame, with the bookkeeping a canvas would do.
    struct Rig {
        glide: CaretGlide,
        f: Frame,
    }

    impl Rig {
        fn new(at: Bar) -> Self {
            let f = Frame {
                target: at,
                cause: 1,
                view: 1,
                shown: true,
                viewport: Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 800.0)),
                now: 100.0,
                frame_nr: 10,
            };
            let mut rig = Rig { glide: CaretGlide::with_secs(GLIDE_SECS), f };
            assert_eq!(rig.glide.frame(rig.f), at, "the first frame draws the caret in place");
            rig
        }

        /// The next frame, `dt` seconds later.
        fn step(&mut self, dt: f64) -> Bar {
            self.f.now += dt;
            self.f.frame_nr += 1;
            self.glide.frame(self.f)
        }

        /// The caret moved because of an edit or a selection change.
        fn move_to(&mut self, to: Bar) {
            self.f.target = to;
            self.f.cause += 1;
        }
    }

    #[test]
    fn a_move_glides_and_lands_exactly() {
        let mut r = Rig::new(bar(100.0, 50.0));
        r.move_to(bar(200.0, 90.0));
        assert_eq!(r.step(DT), bar(100.0, 50.0), "the glide starts from where the caret was");
        assert!(r.glide.is_gliding());
        let mut last = 100.0;
        let mut frames = 0;
        while r.glide.is_gliding() {
            let b = r.step(DT);
            assert!(b[0].x >= last && b[0].x <= 200.0, "moves toward the target without overshooting: {b:?}");
            assert!((b[1].y - b[0].y - 16.0).abs() < 1e-3, "the bar keeps its height: {b:?}");
            last = b[0].x;
            frames += 1;
            assert!(frames < 60);
        }
        assert_eq!(r.glide.drawn(), Some(bar(200.0, 90.0)));
        // 0.12 s at 60 Hz: eight frames, and it decelerates (most of the way in half the time).
        assert!((7..=9).contains(&frames), "{frames} frames");
        assert_eq!(r.step(DT), bar(200.0, 90.0), "then it rests");
    }

    #[test]
    fn the_glide_is_the_same_at_any_frame_rate() {
        let at = |hz: f64| {
            let mut r = Rig::new(bar(0.0, 0.0));
            r.move_to(bar(300.0, 0.0));
            r.step(1.0 / hz);
            let mut b = bar(0.0, 0.0);
            // 0.05 s in: three frames at 60 Hz, twelve at 240 Hz.
            for _ in 0..(hz / 20.0).round() as usize {
                b = r.step(1.0 / hz);
            }
            b[0].x
        };
        let expected = 300.0 * (1.0 - (1.0 - 0.05 / GLIDE_SECS).powi(3)) as f32;
        assert!((at(60.0) - expected).abs() < 0.5, "{} vs {expected}", at(60.0));
        assert!((at(240.0) - expected).abs() < 0.5, "{} vs {expected}", at(240.0));
        assert!(expected > 150.0, "decelerating: past half-way in under half the time");
    }

    #[test]
    fn a_new_move_takes_over_from_where_the_caret_is() {
        let mut r = Rig::new(bar(0.0, 0.0));
        r.move_to(bar(100.0, 0.0));
        r.step(DT);
        let mid = r.step(DT * 2.0);
        assert!(mid[0].x > 0.0 && mid[0].x < 100.0);
        // Typing on: the caret is asked to go further before it arrived.
        r.move_to(bar(110.0, 0.0));
        assert_eq!(r.step(DT), mid, "no jump when the destination changes");
        let next = r.step(DT);
        assert!(next[0].x > mid[0].x && next[0].x < 110.0);
        for _ in 0..10 {
            r.step(DT);
        }
        assert_eq!(r.glide.drawn(), Some(bar(110.0, 0.0)));
        assert!(!r.glide.is_gliding());
        // Something else changed the document but left the caret alone: nothing to do.
        r.f.cause += 1;
        assert_eq!(r.step(DT), bar(110.0, 0.0));
        assert!(!r.glide.is_gliding());
    }

    #[test]
    fn view_changes_and_scrolling_never_glide() {
        // Zoom or a resized window: the caret stays glued to its text.
        let mut r = Rig::new(bar(100.0, 100.0));
        r.f.target = bar(180.0, 150.0);
        r.f.view += 1;
        assert_eq!(r.step(DT), bar(180.0, 150.0));
        assert!(!r.glide.is_gliding());
        // …even when the same command also moved the caret.
        r.move_to(bar(40.0, 40.0));
        r.f.view += 1;
        assert_eq!(r.step(DT), bar(40.0, 40.0));
        assert!(!r.glide.is_gliding());
        // The target shifting with nothing edited (pixel snapping while scrolling) is followed.
        r.f.target = bar(40.4, 40.0);
        assert_eq!(r.step(DT), bar(40.4, 40.0));
        assert!(!r.glide.is_gliding());
        // The same shift under a glide moves the whole glide; it doesn't restart it.
        r.move_to(bar(140.0, 40.0));
        r.step(DT);
        let before = r.step(DT);
        r.f.target = bar(140.0, 40.5);
        let after = r.step(DT);
        assert!(after[0].x > before[0].x && (after[0].y - 40.5).abs() < 1e-3, "{after:?}");
        for _ in 0..10 {
            r.step(DT);
        }
        assert_eq!(r.glide.drawn(), Some(bar(140.0, 40.5)));
    }

    #[test]
    fn a_hidden_stale_or_off_screen_caret_appears_in_place() {
        // Hidden (a selection, or the canvas lost focus), then back somewhere else.
        let mut r = Rig::new(bar(100.0, 100.0));
        r.f.shown = false;
        r.move_to(bar(300.0, 100.0));
        r.step(DT);
        assert_eq!(r.glide.drawn(), None);
        r.f.shown = true;
        r.move_to(bar(500.0, 100.0));
        assert_eq!(r.step(DT), bar(500.0, 100.0));
        assert!(!r.glide.is_gliding());
        // The canvas skipped frames (Backstage open): no glide from the stale place.
        r.f.frame_nr += 30;
        r.move_to(bar(200.0, 100.0));
        assert_eq!(r.step(DT), bar(200.0, 100.0));
        assert!(!r.glide.is_gliding());
        // A jump to somewhere off screen (Ctrl+End), and back from there.
        r.move_to(bar(200.0, 5000.0));
        assert_eq!(r.step(DT), bar(200.0, 5000.0));
        assert!(!r.glide.is_gliding());
        r.move_to(bar(200.0, 100.0));
        assert_eq!(r.step(DT), bar(200.0, 100.0));
        assert!(!r.glide.is_gliding());
        // Just past the edge (the next line while the view scrolls to it) still glides.
        r.move_to(bar(200.0, 810.0));
        r.step(DT);
        assert!(r.glide.is_gliding());
    }

    #[test]
    fn turned_off_or_hostile_numbers_never_glide_or_panic() {
        let mut off = Rig::new(bar(0.0, 0.0));
        off.glide = CaretGlide::with_secs(0.0);
        off.step(DT);
        off.move_to(bar(90.0, 0.0));
        assert_eq!(off.step(DT), bar(90.0, 0.0));
        assert!(!off.glide.is_gliding());

        assert_eq!(CaretGlide::with_secs(f64::NAN).secs, GLIDE_SECS);
        assert_eq!(CaretGlide::with_secs(-3.0).secs, 0.0);
        assert_eq!(CaretGlide::with_secs(f64::INFINITY).secs, GLIDE_SECS);
        assert_eq!(parse_ms(None), GLIDE_SECS);
        assert_eq!(parse_ms(Some("junk")), GLIDE_SECS);
        assert_eq!(parse_ms(Some("NaN")), GLIDE_SECS);
        assert_eq!(parse_ms(Some(" 0 ")), 0.0);
        assert_eq!(parse_ms(Some("-5")), 0.0);
        assert_eq!(parse_ms(Some("250")), 0.25);
        assert_eq!(parse_ms(Some("1e300")), MAX_GLIDE_SECS);

        let mut r = Rig::new(bar(10.0, 10.0));
        let junk = [pos2(f32::NAN, f32::INFINITY), pos2(f32::NEG_INFINITY, 0.0)];
        r.move_to(junk);
        r.step(DT);
        assert!(!r.glide.is_gliding() && r.glide.drawn().is_none());
        r.move_to(bar(50.0, 10.0));
        assert_eq!(r.step(DT), bar(50.0, 10.0), "back from junk: in place");
        // The clock stops, jumps back or goes bad mid-glide.
        r.move_to(bar(150.0, 10.0));
        r.step(DT);
        assert!(is_finite(r.step(0.0)));
        assert_eq!(r.step(-50.0), bar(150.0, 10.0));
        r.move_to(bar(250.0, 10.0));
        r.step(DT);
        assert_eq!(r.step(f64::NAN), bar(250.0, 10.0));
        r.f.now = 200.0;
        r.move_to(bar(350.0, 10.0));
        r.step(DT);
        assert_eq!(r.step(f64::INFINITY), bar(350.0, 10.0));
        // Coordinates at the edge of f32.
        let mut r = Rig::new(bar(0.0, 0.0));
        r.f.viewport = Rect::EVERYTHING;
        r.move_to(bar(f32::MAX, 0.0));
        r.step(DT);
        for _ in 0..12 {
            r.step(DT);
        }
        r.glide.reset();
        assert!(r.glide.drawn().is_none() && !r.glide.is_gliding());
    }

    /// The whole path: a caret command in the real app glides the drawn caret across the canvas.
    #[test]
    fn the_canvas_caret_glides_after_a_command() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Default::default());
        a.run("text.insert", json!({"text": "Some text here"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 900.0)).with_step_dt(DT as f32).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        // The test harness turns egui's animations off, and the glide follows that switch.
        h.step();
        assert!(state(&h.ctx).drawn().is_none(), "no animations: the glide stands down");
        h.ctx.all_styles_mut(|s| s.animation_time = 0.1);
        for _ in 0..30 {
            h.step();
        }
        let end = state(&h.ctx).drawn().expect("the focused canvas draws a caret");
        assert!(!state(&h.ctx).is_gliding());
        h.state_mut().run("caret.docStart", json!({})).unwrap();
        h.step();
        h.step();
        let mid = state(&h.ctx);
        assert!(mid.is_gliding(), "the caret is on its way");
        let x = mid.drawn().unwrap()[0].x;
        for _ in 0..20 {
            h.step();
        }
        let start = state(&h.ctx).drawn().unwrap();
        assert!(!state(&h.ctx).is_gliding(), "and comes to rest");
        assert!(start[0].x + 20.0 < end[0].x, "the caret moved to the start of the line: {start:?} from {end:?}");
        assert!(x > start[0].x && x < end[0].x, "half-way it was drawn in between: {x}");
        // Zooming moves the caret on screen but doesn't glide it.
        h.state_mut().run("view.zoom", json!({"value": 150})).unwrap();
        for _ in 0..3 {
            h.step();
            assert!(!state(&h.ctx).is_gliding());
        }
        assert_ne!(state(&h.ctx).drawn().unwrap(), start);
    }
}
