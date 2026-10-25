//! The game: input, the three ways of moving (free flight, circling an
//! object, autopilot), the journal and the readouts.

use super::render::{Renderer, Scene};
use super::ui::Ui;
use crate::astro;
use crate::format::{fmt_au, fmt_duration, fmt_num};
use crate::camera::{look_rotation, smoothstep, Camera};
use crate::catalog::{Kind, Motion, BODIES};
use crate::sim::{Sim, FIRST_DAY, LAST_DAY};
use glam::{DQuat, DVec3};
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;
use web_sys::Storage;

/// Time warp steps: simulated seconds per real second, and how to say it.
const WARPS: [(f64, &str); 8] = [
    (1.0, "Real time"),
    (60.0, "1 minute per second"),
    (3_600.0, "1 hour per second"),
    (21_600.0, "6 hours per second"),
    (86_400.0, "1 day per second"),
    (604_800.0, "1 week per second"),
    (2_629_800.0, "1 month per second"),
    (31_557_600.0, "1 year per second"),
];
const DEFAULT_WARP: usize = 2;
const JOURNAL_KEY: &str = "tss-journal-v1";
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
/// Where the flight starts, and the Home view: above the inner Solar System.
const OVERVIEW: DVec3 = DVec3::new(-38.0, 62.0, 118.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Select(usize),
    Fly(usize),
    Next,
    Prev,
    Close,
    Pause,
    Faster,
    Slower,
    Now,
    Orbits,
    Labels,
    Journal,
    Help,
    Fullscreen,
    Tour,
    Explore,
    CloseOverlay,
    Home,
}

impl Action {
    pub fn parse(s: &str) -> Option<Action> {
        let index = |rest: &str| rest.parse::<usize>().ok().filter(|&i| i < BODIES.len());
        Some(match s {
            "next" => Action::Next,
            "prev" => Action::Prev,
            "close" => Action::Close,
            "pause" => Action::Pause,
            "faster" => Action::Faster,
            "slower" => Action::Slower,
            "now" => Action::Now,
            "orbits" => Action::Orbits,
            "labels" => Action::Labels,
            "journal" => Action::Journal,
            "help" => Action::Help,
            "fullscreen" => Action::Fullscreen,
            "tour" => Action::Tour,
            "explore" => Action::Explore,
            "close-overlay" => Action::CloseOverlay,
            "home" => Action::Home,
            _ => {
                if let Some(rest) = s.strip_prefix("select:") {
                    Action::Select(index(rest)?)
                } else if let Some(rest) = s.strip_prefix("fly:") {
                    Action::Fly(index(rest)?)
                } else {
                    return None;
                }
            }
        })
    }
}

pub enum Input {
    Action(Action),
    Down { id: i32, x: f64, y: f64 },
    Move { id: i32, x: f64, y: f64 },
    Up { id: i32, x: f64, y: f64 },
    Wheel { dy: f64 },
    DoubleClick { x: f64, y: f64 },
    Key { code: String, down: bool },
    Blur,
}

pub type Queue = Rc<RefCell<VecDeque<Input>>>;

#[derive(Clone, Copy)]
struct Travel {
    /// The body the camera ends up circling (the Sun, for regions).
    target: usize,
    /// What the trip is to (shown in the panel on arrival).
    subject: usize,
    start: DVec3,
    end: DVec3,
    from_rot: DQuat,
    t: f64,
    duration: f64,
}

#[derive(Clone, Copy)]
enum Mode {
    Free,
    Orbit { target: usize, offset: DVec3, subject: usize },
    Travel(Travel),
}

pub struct App {
    sim: Sim,
    cam: Camera,
    mode: Mode,
    renderer: Renderer,
    pub ui: Ui,
    queue: Queue,
    days: f64,
    warp: usize,
    paused: bool,
    keys: HashSet<String>,
    pointers: Vec<(i32, f64, f64)>,
    press: Option<(f64, f64, f64)>,
    dragged: f64,
    pinch: Option<f64>,
    selected: Option<usize>,
    throttle: f64,
    show_orbits: bool,
    show_labels: bool,
    visited: Vec<bool>,
    tour: Option<usize>,
    last_ms: Option<f64>,
    hud_ms: f64,
    reduced_motion: bool,
    storage: Option<Storage>,
    width: f64,
    height: f64,
    dpr: f64,
    now_ms: f64,
}

fn ease(s: f64) -> f64 {
    if s < 0.5 {
        4.0 * s * s * s
    } else {
        1.0 - (-2.0 * s + 2.0).powi(3) / 2.0
    }
}

impl App {
    pub fn new(renderer: Renderer, ui: Ui, queue: Queue, now_ms: f64, reduced_motion: bool, storage: Option<Storage>) -> App {
        let days = astro::days_from_unix_ms(now_ms).clamp(FIRST_DAY, LAST_DAY);
        let sim = Sim::new(days);
        let cam = Camera { pos: OVERVIEW, rot: look_rotation(-OVERVIEW, DVec3::Y), fov: 55f64.to_radians() };
        let mut visited = vec![false; BODIES.len()];
        if let Some(saved) = storage.as_ref().and_then(|s| s.get_item(JOURNAL_KEY).ok().flatten()) {
            for id in saved.split(',') {
                if let Some(i) = crate::catalog::index_of(id) {
                    visited[i] = true;
                }
            }
        }
        let app = App {
            sim,
            cam,
            mode: Mode::Orbit { target: 0, offset: OVERVIEW, subject: 0 },
            renderer,
            ui,
            queue,
            days,
            warp: DEFAULT_WARP,
            paused: false,
            keys: HashSet::new(),
            pointers: Vec::new(),
            press: None,
            dragged: 0.0,
            pinch: None,
            selected: None,
            throttle: 1.0,
            show_orbits: true,
            show_labels: true,
            visited,
            tour: None,
            last_ms: None,
            hud_ms: 0.0,
            reduced_motion,
            storage,
            width: 1.0,
            height: 1.0,
            dpr: 1.0,
            now_ms,
        };
        app.ui.update_journal(&app.visited);
        app.ui.set_toggles(true, true);
        app
    }

    pub fn load_textures(&self, url_for: impl Fn(&str) -> String) {
        self.renderer.load_textures(url_for);
    }

    pub fn frame(&mut self, now_ms: f64, width: f64, height: f64, dpr: f64) {
        let dt = self.last_ms.map_or(0.0, |t| ((now_ms - t) / 1000.0).clamp(0.0, 0.1));
        self.last_ms = Some(now_ms);
        self.now_ms = now_ms;
        self.width = width.max(1.0);
        self.height = height.max(1.0);
        self.dpr = dpr;

        let inputs: Vec<Input> = self.queue.borrow_mut().drain(..).collect();
        for input in inputs {
            self.handle(input);
        }
        if !self.paused {
            self.days = (self.days + dt * WARPS[self.warp].0 / 86_400.0).clamp(FIRST_DAY, LAST_DAY);
        }
        self.sim.update(self.days);
        self.move_camera(dt);
        self.check_journal();

        self.renderer.draw(&Scene {
            cam: &self.cam,
            sim: &self.sim,
            width: self.width,
            height: self.height,
            dpr,
            selected: self.selected,
            show_orbits: self.show_orbits,
            seconds: now_ms / 1000.0,
        });
        self.place_labels();
        if now_ms - self.hud_ms > 100.0 {
            self.hud_ms = now_ms;
            self.update_hud();
        }
        self.ui.tick(now_ms);
    }

    // --- input ----------------------------------------------------------------

    fn handle(&mut self, input: Input) {
        match input {
            Input::Action(a) => self.act(a),
            Input::Down { id, x, y } => {
                self.pointers.retain(|p| p.0 != id);
                self.pointers.push((id, x, y));
                if self.pointers.len() == 1 {
                    self.press = Some((x, y, self.now_ms));
                    self.dragged = 0.0;
                } else {
                    self.press = None;
                    self.pinch = self.pinch_distance();
                }
            }
            Input::Move { id, x, y } => {
                let Some(k) = self.pointers.iter().position(|p| p.0 == id) else { return };
                let (_, px, py) = self.pointers[k];
                self.pointers[k] = (id, x, y);
                if self.pointers.len() == 1 {
                    let (dx, dy) = (x - px, y - py);
                    self.dragged += dx.abs() + dy.abs();
                    self.drag(dx, dy);
                } else if let (Some(before), Some(now)) = (self.pinch, self.pinch_distance()) {
                    self.zoom((before / now).ln() * 600.0);
                    self.pinch = Some(now);
                }
            }
            Input::Up { id, x, y } => {
                self.pointers.retain(|p| p.0 != id);
                if self.pointers.len() < 2 {
                    self.pinch = None;
                }
                if let Some((_, _, t)) = self.press.take() {
                    if self.dragged < 6.0 && self.now_ms - t < 500.0 {
                        if let Some(i) = self.pick(x, y) {
                            self.act(Action::Select(i));
                        }
                    }
                }
            }
            Input::Wheel { dy } => self.zoom(dy),
            Input::DoubleClick { x, y } => {
                if let Some(i) = self.pick(x, y) {
                    self.act(Action::Fly(i));
                }
            }
            Input::Key { code, down } => {
                if down {
                    let fresh = self.keys.insert(code.clone());
                    if fresh {
                        self.key_press(&code);
                    }
                } else {
                    self.keys.remove(&code);
                }
            }
            Input::Blur => {
                self.keys.clear();
                self.pointers.clear();
                self.press = None;
                self.pinch = None;
            }
        }
    }

    fn pinch_distance(&self) -> Option<f64> {
        if self.pointers.len() < 2 {
            return None;
        }
        let (_, ax, ay) = self.pointers[0];
        let (_, bx, by) = self.pointers[1];
        Some(((ax - bx).powi(2) + (ay - by).powi(2)).sqrt().max(1.0))
    }

    fn key_press(&mut self, code: &str) {
        let action = match code {
            "Space" => Action::Pause,
            "Comma" => Action::Slower,
            "Period" => Action::Faster,
            "KeyT" => Action::Now,
            "KeyO" => Action::Orbits,
            "KeyL" => Action::Labels,
            "KeyJ" => Action::Journal,
            "KeyH" | "Slash" => Action::Help,
            "BracketRight" | "KeyN" => Action::Next,
            "BracketLeft" | "KeyP" => Action::Prev,
            "Home" | "Digit0" => Action::Home,
            "KeyG" | "Enter" | "NumpadEnter" => match self.selected {
                Some(i) => Action::Fly(i),
                None => return,
            },
            "Escape" => {
                if self.ui.help_open() || self.ui.journal_open() {
                    Action::CloseOverlay
                } else if self.ui.intro_open() {
                    Action::Explore
                } else if self.ui.panel_open() {
                    Action::Close
                } else {
                    self.go_free();
                    return;
                }
            }
            _ => return,
        };
        self.act(action);
    }

    fn act(&mut self, action: Action) {
        match action {
            Action::Select(i) => {
                self.selected = Some(i);
                self.tour = Some(i);
                let parent = self.sim.parent[i].map(|p| &BODIES[p]);
                self.ui.show_panel(i, parent, self.visited[i]);
                self.update_live();
            }
            Action::Fly(i) => {
                self.ui.hide_intro();
                self.ui.close_overlays();
                self.act(Action::Select(i));
                self.start_travel(i);
            }
            Action::Next | Action::Prev => {
                let n = BODIES.len();
                let next = match (self.tour, action) {
                    (None, _) => 0,
                    (Some(t), Action::Next) => (t + 1) % n,
                    (Some(t), _) => (t + n - 1) % n,
                };
                self.act(Action::Fly(next));
            }
            Action::Close => {
                self.selected = None;
                self.ui.hide_panel();
                self.ui.root.focus().ok();
            }
            Action::Pause => {
                self.paused = !self.paused;
                self.ui.set_paused(self.paused);
            }
            Action::Faster | Action::Slower => {
                if self.paused {
                    self.paused = false;
                    self.ui.set_paused(false);
                }
                self.warp = if action == Action::Faster {
                    (self.warp + 1).min(WARPS.len() - 1)
                } else {
                    self.warp.saturating_sub(1)
                };
            }
            Action::Now => {
                self.days = astro::days_from_unix_ms(js_sys::Date::now()).clamp(FIRST_DAY, LAST_DAY);
                self.warp = DEFAULT_WARP;
                self.ui.toast("Back to the present moment.", self.now_ms);
            }
            Action::Orbits => {
                self.show_orbits = !self.show_orbits;
                self.ui.set_toggles(self.show_orbits, self.show_labels);
            }
            Action::Labels => {
                self.show_labels = !self.show_labels;
                self.ui.set_toggles(self.show_orbits, self.show_labels);
            }
            Action::Journal => {
                let open = !self.ui.journal_open();
                self.ui.show_journal(open, &self.visited);
            }
            Action::Help => self.ui.show_help(!self.ui.help_open()),
            Action::CloseOverlay => self.ui.close_overlays(),
            Action::Fullscreen => {
                let doc = web_sys::window().and_then(|w| w.document());
                if let Some(doc) = doc {
                    if doc.fullscreen_element().is_some() {
                        doc.exit_fullscreen();
                    } else {
                        self.ui.root.request_fullscreen().ok();
                    }
                }
            }
            Action::Tour => {
                self.ui.hide_intro();
                self.ui.root.focus().ok();
                self.act(Action::Fly(0));
            }
            Action::Explore => {
                self.ui.hide_intro();
                self.ui.root.focus().ok();
                self.ui.toast("Free flight: drag to look, W A S D to move, scroll to change speed.", self.now_ms);
                self.go_free();
            }
            Action::Home => {
                self.ui.hide_intro();
                self.travel_to(0, 0, OVERVIEW);
            }
        }
    }

    fn go_free(&mut self) {
        self.mode = Mode::Free;
    }

    /// The body under a screen point: the nearest one whose disc (or, for
    /// tiny ones, a 14 px target) contains the point.
    fn pick(&self, x: f64, y: f64) -> Option<usize> {
        let mut best: Option<(usize, f64)> = None;
        for (i, b) in BODIES.iter().enumerate() {
            let st = &self.sim.states[i];
            if !st.exists || b.is_region() {
                continue;
            }
            let Some((sx, sy, depth)) = self.cam.project(st.pos - self.cam.pos, self.width, self.height) else { continue };
            let r = (st.radius * self.cam.pixels_per_unit(depth, self.height)).max(14.0);
            let d = ((sx - x).powi(2) + (sy - y).powi(2)).sqrt();
            if d < r && best.is_none_or(|(_, bd)| d / r < bd) {
                best = Some((i, d / r));
            }
        }
        best.map(|(i, _)| i)
    }

    // --- moving -----------------------------------------------------------------

    fn drag(&mut self, dx: f64, dy: f64) {
        let k = self.cam.fov / self.height;
        match &mut self.mode {
            Mode::Free => {
                let q = DQuat::from_axis_angle(self.cam.up(), dx * k) * DQuat::from_axis_angle(self.cam.right(), dy * k);
                self.cam.rot = (q * self.cam.rot).normalize();
            }
            Mode::Orbit { offset, .. } => {
                let k = k * 1.6;
                let q = DQuat::from_axis_angle(self.cam.up(), -dx * k) * DQuat::from_axis_angle(self.cam.right(), -dy * k);
                *offset = q * *offset;
                self.cam.rot = (q * self.cam.rot).normalize();
            }
            Mode::Travel(_) => {}
        }
    }

    fn zoom(&mut self, dy: f64) {
        match &mut self.mode {
            Mode::Orbit { target, offset, .. } => {
                let st = &self.sim.states[*target];
                let min = (st.radius * 1.12).max(0.01);
                let len = (offset.length() * (dy * 0.0012).exp()).clamp(min, 60_000.0);
                *offset = offset.normalize() * len;
            }
            Mode::Free => {
                self.throttle = (self.throttle * (-dy * 0.0012).exp()).clamp(0.02, 50.0);
                self.ui.toast(&format!("Speed × {}", fmt_num(self.throttle, if self.throttle < 1.0 { 2 } else { 1 })), self.now_ms);
            }
            Mode::Travel(_) => {}
        }
    }

    fn start_travel(&mut self, subject: usize) {
        let (target, end) = self.sim.vantage(subject);
        self.travel_to(subject, target, end);
    }

    fn travel_to(&mut self, subject: usize, target: usize, end: DVec3) {
        let tp = self.sim.states[target].pos;
        let mut start = self.cam.pos - tp;
        if start.length() < 1e-9 {
            start = end * 2.0;
        }
        let ratio = (start.length() / end.length()).max(end.length() / start.length());
        let turn = start.normalize().angle_between(end.normalize());
        let duration = if self.reduced_motion { 0.0 } else { (1.4 + 0.7 * ratio.ln() + 0.5 * turn).clamp(1.4, 8.0) };
        self.mode = Mode::Travel(Travel { target, subject, start, end, from_rot: self.cam.rot, t: 0.0, duration });
    }

    fn move_camera(&mut self, dt: f64) {
        let held = |app: &App, codes: &[&str]| codes.iter().any(|c| app.keys.contains(*c));
        let mut thrust = DVec3::ZERO;
        for (codes, dir) in [
            (&["KeyW"][..], DVec3::NEG_Z),
            (&["KeyS"][..], DVec3::Z),
            (&["KeyA"][..], DVec3::NEG_X),
            (&["KeyD"][..], DVec3::X),
            (&["KeyR"][..], DVec3::Y),
            (&["KeyF"][..], DVec3::NEG_Y),
        ] {
            if held(self, codes) {
                thrust += dir;
            }
        }
        if thrust != DVec3::ZERO && !matches!(self.mode, Mode::Free) {
            self.mode = Mode::Free;
        }
        let turn_x = (held(self, &["ArrowLeft"]) as i32 - held(self, &["ArrowRight"]) as i32) as f64;
        let turn_y = (held(self, &["ArrowUp"]) as i32 - held(self, &["ArrowDown"]) as i32) as f64;
        let roll = (held(self, &["KeyQ"]) as i32 - held(self, &["KeyE"]) as i32) as f64;

        match self.mode {
            Mode::Free => {
                let rate = 1.1 * dt;
                let q = DQuat::from_axis_angle(self.cam.up(), turn_x * rate)
                    * DQuat::from_axis_angle(self.cam.right(), turn_y * rate)
                    * DQuat::from_axis_angle(self.cam.forward(), -roll * rate);
                self.cam.rot = (q * self.cam.rot).normalize();
                if thrust != DVec3::ZERO {
                    let boost = if held(self, &["ShiftLeft", "ShiftRight"]) { 5.0 } else { 1.0 };
                    let speed = self.throttle * boost * self.nearest_surface().0.max(0.004) * 0.9;
                    self.cam.pos += (self.cam.rot * thrust.normalize()) * speed * dt;
                }
            }
            Mode::Orbit { target, offset, subject } => {
                if turn_x != 0.0 || turn_y != 0.0 {
                    let a = 1.2 * dt;
                    let q = DQuat::from_axis_angle(self.cam.up(), -turn_x * a) * DQuat::from_axis_angle(self.cam.right(), turn_y * a);
                    self.mode = Mode::Orbit { target, offset: q * offset, subject };
                    self.cam.rot = (q * self.cam.rot).normalize();
                }
                if let Mode::Orbit { target, offset, .. } = self.mode {
                    self.cam.pos = self.sim.states[target].pos + offset;
                }
            }
            Mode::Travel(mut t) => {
                t.t += dt;
                let s = if t.duration > 0.0 { (t.t / t.duration).min(1.0) } else { 1.0 };
                let e = ease(s);
                let d = (t.start.length().ln() * (1.0 - e) + t.end.length().ln() * e).exp();
                let arc = DQuat::from_rotation_arc(t.start.normalize(), t.end.normalize());
                let dir = DQuat::IDENTITY.slerp(arc, e) * t.start.normalize();
                let tp = self.sim.states[t.target].pos;
                self.cam.pos = tp + dir * d;
                self.keep_outside_bodies(Some(t.target));
                let look = look_rotation(tp - self.cam.pos, self.cam.up());
                self.cam.rot = t.from_rot.slerp(look, smoothstep(s / 0.35)).normalize();
                if s >= 1.0 {
                    self.cam.pos = tp + t.end;
                    self.cam.rot = look_rotation(-t.end, self.cam.up());
                    self.mode = Mode::Orbit { target: t.target, offset: t.end, subject: t.subject };
                    self.arrive(t.subject);
                } else {
                    self.mode = Mode::Travel(t);
                }
                return;
            }
        }
        self.keep_outside_bodies(None);
    }

    /// Never fly inside a planet: push the camera back out to just above
    /// the surface.
    fn keep_outside_bodies(&mut self, except: Option<usize>) {
        for (i, st) in self.sim.states.iter().enumerate() {
            if Some(i) == except || st.radius <= 0.0 || !st.exists || BODIES[i].look.shade == crate::catalog::Shade::Ghost {
                continue;
            }
            let shape = BODIES[i].look.shape;
            let r = st.radius * shape[0].max(shape[1]).max(shape[2]) as f64 * 1.06;
            let d = self.cam.pos - st.pos;
            if d.length() < r {
                self.cam.pos = st.pos + d.normalize_or(DVec3::Y) * r;
                if let Mode::Orbit { target, offset, subject } = self.mode {
                    if target == i {
                        self.mode = Mode::Orbit { target, offset: offset.normalize() * r, subject };
                    }
                }
            }
        }
    }

    /// Distance to the nearest surface (display units) and whose it is.
    fn nearest_surface(&self) -> (f64, usize) {
        let mut best = (f64::MAX, 0);
        for (i, st) in self.sim.states.iter().enumerate() {
            if !st.exists || BODIES[i].is_region() {
                continue;
            }
            let d = ((self.cam.pos - st.pos).length() - st.radius).max(0.0);
            if d < best.0 {
                best = (d, i);
            }
        }
        best
    }

    fn arrive(&mut self, subject: usize) {
        let b = &BODIES[subject];
        self.visit(subject);
        if self.selected != Some(subject) {
            self.act(Action::Select(subject));
        }
        let text = match b.kind {
            Kind::Region => format!("Arrived: {}", b.name),
            _ => format!("Arrived at {}", b.name),
        };
        self.ui.toast(&text, self.now_ms);
    }

    // --- the journal ---------------------------------------------------------

    fn visit(&mut self, i: usize) {
        if self.visited[i] {
            return;
        }
        self.visited[i] = true;
        if let Some(storage) = &self.storage {
            let ids: Vec<&str> = BODIES.iter().zip(&self.visited).filter(|(_, &v)| v).map(|(b, _)| b.id).collect();
            storage.set_item(JOURNAL_KEY, &ids.join(",")).ok();
        }
        self.ui.update_journal(&self.visited);
        let n = self.visited.iter().filter(|&&v| v).count();
        let text = if n == BODIES.len() {
            "Journal complete: you have charted the whole Solar System.".to_string()
        } else {
            format!("New journal entry: {} ({n}/{})", BODIES[i].name, BODIES.len())
        };
        self.ui.toast(&text, self.now_ms);
    }

    fn check_journal(&mut self) {
        let cam_au = Sim::au_from_sun(self.cam.pos);
        for i in 0..BODIES.len() {
            if self.visited[i] || !self.sim.states[i].exists {
                continue;
            }
            let near = match BODIES[i].id {
                "asteroid-belt" => (2.1..3.3).contains(&cam_au),
                "kuiper-belt" => (30.0..50.0).contains(&cam_au),
                "heliosphere" => cam_au > 110.0,
                "oort-cloud" => cam_au > 2_000.0,
                _ if BODIES[i].is_region() => false,
                _ => (self.cam.pos - self.sim.states[i].pos).length() < self.sim.view_distance(i) * 2.0,
            };
            if near && !self.ui.intro_open() {
                self.visit(i);
            }
        }
    }

    // --- readouts ---------------------------------------------------------------

    fn place_labels(&mut self) {
        let cam_au = Sim::au_from_sun(self.cam.pos);
        let selected = self.selected;
        let rank = |i: usize| -> i32 {
            if Some(i) == selected {
                return -1;
            }
            match BODIES[i].kind {
                Kind::Star => 0,
                Kind::Planet => 1,
                Kind::DwarfPlanet | Kind::Region | Kind::Hypothetical => 2,
                Kind::Moon => 4,
                _ => 3,
            }
        };
        let mut order: Vec<usize> = (0..BODIES.len()).collect();
        order.sort_by_key(|&i| rank(i));
        let mut placed: Vec<(f64, f64, f64, f64)> = Vec::new();
        let mut shown = vec![false; BODIES.len()];
        for i in order {
            let b = &BODIES[i];
            let st = &self.sim.states[i];
            let is_selected = Some(i) == selected;
            // Far-flung things are labelled once you are far enough out to care.
            let relevant = is_selected
                || match b.motion {
                    Motion::Region { at: (_, _, au), .. } => cam_au > au * 0.2,
                    Motion::Probe { .. } => cam_au > 8.0,
                    Motion::Fixed { .. } => cam_au > 15.0,
                    _ => true,
                };
            let mut at = None;
            if (self.show_labels || is_selected) && st.exists && relevant {
                let c = st.pos - self.cam.pos;
                if let Some((x, y, depth)) = self.cam.project(c, self.width, self.height) {
                    let ppu = self.cam.pixels_per_unit(depth, self.height);
                    let r = (st.radius * ppu).min(self.height);
                    let mut visible = r < self.height * 0.9;
                    // A moon is labelled once it separates from its planet,
                    // and only alongside the planet's own label.
                    if b.kind == Kind::Moon && !is_selected {
                        if let Some(p) = self.sim.parent[i] {
                            let sep = (st.pos - self.sim.states[p].pos).length() * ppu;
                            visible &= sep > 28.0 && shown[p];
                        }
                    }
                    let w = b.name.chars().count() as f64 * 7.2 + 16.0;
                    let h = 22.0;
                    let candidates = [
                        (x, y + r + 6.0),
                        (x, y - r - 6.0 - h),
                        (x + r + 8.0 + w / 2.0, y - h / 2.0),
                        (x - r - 8.0 - w / 2.0, y - h / 2.0),
                    ];
                    if visible {
                        for (lx, ly) in candidates {
                            let rect = (lx - w / 2.0, ly, lx + w / 2.0, ly + h);
                            let on_screen = rect.0 > -w * 0.4 && rect.2 < self.width + w * 0.4 && rect.1 > 0.0 && rect.3 < self.height;
                            let clear = !placed.iter().any(|p| rect.0 < p.2 && rect.2 > p.0 && rect.1 < p.3 && rect.3 > p.1);
                            if on_screen && (clear || is_selected) {
                                placed.push(rect);
                                at = Some((lx, ly));
                                break;
                            }
                        }
                    }
                }
            }
            shown[i] = at.is_some();
            self.ui.place_label(i, at, is_selected);
        }
    }

    fn update_hud(&mut self) {
        let (y, m, d, hh, mm) = astro::civil_from_days(self.days);
        let date = format!("{d} {} {y}, {hh:02}:{mm:02} UTC", MONTHS[(m - 1) as usize]);
        let warp = if self.paused { "Time paused".to_string() } else { WARPS[self.warp].1.to_string() };
        let r = Sim::au_from_sun(self.cam.pos);
        let where_ = format!("{} from the Sun · sunlight here is {} old", fmt_au(r), fmt_duration(r * astro::LIGHT_SECONDS_PER_AU));
        let near = match self.mode {
            Mode::Orbit { subject, .. } | Mode::Travel(Travel { subject, .. }) => {
                let verb = if matches!(self.mode, Mode::Travel(_)) { "Flying to" } else { "Visiting" };
                format!("{verb}: {}", BODIES[subject].name)
            }
            Mode::Free => format!("Free flight · nearest: {}", BODIES[self.nearest_surface().1].name),
        };
        self.ui.set_hud(date, warp, where_, near);
        self.update_live();
    }

    fn update_live(&mut self) {
        let Some(i) = self.selected else { return };
        let b = &BODIES[i];
        let earth = self.sim.states[crate::catalog::index_of("earth").unwrap_or(3)].au;
        let au = self.sim.states[i].au;
        let text = match (b.id, &b.motion) {
            (_, Motion::Region { .. }) => String::new(),
            ("earth", _) => format!("Right now: {} from the Sun.", fmt_au(au.length())),
            ("sun", _) => format!("Right now: {} from Earth; its light takes {} to arrive.", fmt_au(earth.length()), fmt_duration(earth.length() * astro::LIGHT_SECONDS_PER_AU)),
            ("planet-nine", _) => "Its true position, if it exists, is unknown.".to_string(),
            (_, Motion::Probe { launched, .. }) if !self.sim.states[i].exists => format!("Not launched yet on this date (launch {}).", launched.floor()),
            _ => {
                let de = (au - earth).length();
                let what = if b.kind == Kind::Spacecraft { "radio signals take" } else { "light takes" };
                format!(
                    "Right now: {} from the Sun · {} from Earth · {what} {} to reach Earth.",
                    fmt_au(au.length()),
                    fmt_au(de),
                    fmt_duration(de * astro::LIGHT_SECONDS_PER_AU)
                )
            }
        };
        self.ui.set_live(text);
    }
}
