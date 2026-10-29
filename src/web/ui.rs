//! The HTML around the canvas: labels on objects, the readout, the controls,
//! the information panel, the journal and the help card.
//!
//! The fixed parts are a template written here; everything that comes from
//! the catalogue is set with `textContent`, never parsed as HTML.

use crate::catalog::{Body, Kind, BODIES};
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlCanvasElement, HtmlElement};

const TEMPLATE: &str = r#"
<canvas class="tss-canvas" role="img" aria-label="A 3D view of the Solar System. Use the object labels, the panel and the controls below to explore it."></canvas>
<div class="tss-labels"></div>
<div class="tss-hud">
  <p class="tss-title">Through the Solar System</p>
  <p><span class="tss-date"></span><span class="tss-warp" aria-live="polite"></span></p>
  <p class="tss-where"></p>
  <p class="tss-near"></p>
</div>
<div class="tss-bar" role="toolbar" aria-label="Controls">
  <div class="tss-group">
    <button type="button" data-action="prev" aria-label="Previous stop" title="Previous stop ( [ )">&#8249;</button>
    <button type="button" data-action="next" class="tss-primary" title="Next stop ( ] )">Next stop &#8250;</button>
  </div>
  <div class="tss-group tss-time" role="group" aria-label="Time">
    <button type="button" data-action="slower" aria-label="Slow time down" title="Slower ( , )">&#171;</button>
    <button type="button" data-action="pause" class="tss-pause" aria-label="Pause time" aria-pressed="false" title="Pause (Space)">&#10074;&#10074;</button>
    <button type="button" data-action="faster" aria-label="Speed time up" title="Faster ( . )">&#187;</button>
    <button type="button" data-action="now" title="Back to the present (T)">Now</button>
  </div>
  <div class="tss-group">
    <button type="button" data-action="orbits" aria-pressed="true" title="Orbits (O)">Orbits</button>
    <button type="button" data-action="labels" aria-pressed="true" title="Labels (L)">Labels</button>
    <button type="button" data-action="journal" class="tss-journal-button" title="Journal (J)">Journal</button>
    <button type="button" data-action="help" aria-label="Help, scale and credits" title="Help (H)">?</button>
    <button type="button" data-action="fullscreen" aria-label="Full screen" title="Full screen">&#x26F6;</button>
  </div>
</div>
<aside class="tss-panel" hidden aria-label="About the selected object"><div class="tss-panel-scroll"></div></aside>
<div class="tss-toast" role="status" aria-live="polite"></div>
<div class="tss-overlay tss-intro">
  <div class="tss-card">
    <p class="tss-kicker">An annotated flight</p>
    <h2>Through the Solar System</h2>
    <p>You are a point of view, free to drift from the Sun out to the Oort cloud. Every object is annotated with <strong>verified facts</strong>, each with its source, and <strong>hypotheses</strong>, each labelled with how settled it is.</p>
    <p>The planets are where they really are right now, worked out from NASA/JPL orbital elements, and time runs at one hour per second. Distances and sizes are squeezed so it all fits &mdash; see Help for the true scale.</p>
    <div class="tss-actions">
      <button type="button" data-action="tour" class="tss-primary">Begin the grand tour</button>
      <button type="button" data-action="explore">Fly freely</button>
    </div>
    <p class="tss-small">Drag to look around &middot; W A S D to fly &middot; click any object to read about it &middot; H for help</p>
  </div>
</div>
<div class="tss-overlay tss-help" hidden>
  <div class="tss-card tss-card--wide" role="dialog" aria-label="Help">
    <h2>How to fly</h2>
    <dl class="tss-keys">
      <div><dt>Drag</dt><dd>Look around, or circle the object you are visiting</dd></div>
      <div><dt>Scroll / pinch</dt><dd>Zoom in and out, or change speed in free flight</dd></div>
      <div><dt>W A S D</dt><dd>Fly forward, back, left and right</dd></div>
      <div><dt>R / F</dt><dd>Up / down</dd></div>
      <div><dt>Q / E</dt><dd>Roll</dd></div>
      <div><dt>Arrow keys</dt><dd>Turn (or circle the object you are visiting)</dd></div>
      <div><dt>Shift</dt><dd>Fly five times faster</dd></div>
      <div><dt>Click</dt><dd>Select an object and read about it</dd></div>
      <div><dt>Double-click, G, Enter</dt><dd>Fly to the selected object</dd></div>
      <div><dt>[ and ]</dt><dd>Previous and next stop on the grand tour</dd></div>
      <div><dt>Space</dt><dd>Pause or resume time</dd></div>
      <div><dt>, and .</dt><dd>Slow time down or speed it up</dd></div>
      <div><dt>T</dt><dd>Back to the present moment</dd></div>
      <div><dt>O, L, J, H</dt><dd>Orbits, labels, journal, this help</dd></div>
      <div><dt>Esc</dt><dd>Close a panel, or stop and fly freely</dd></div>
    </dl>
    <h2>About the scale</h2>
    <p>Space is almost entirely empty. If the Sun were a beach ball 70 cm wide, Earth would be a peppercorn 75 m away, Neptune would be 2.3 km away, Voyager 1 about 13 km away &mdash; and the nearest star another beach ball some 20,000 km away.</p>
    <p>So that everything fits, this view squeezes distances from the Sun (drawn in proportion to the square root of the real distance) and enlarges sizes. <strong>Directions are true</strong>: each planet, dwarf planet and comet sits in its real direction from the Sun for the date shown, computed from NASA/JPL orbital elements, and Earth and the Moon turn to show the right side to the Sun. Moon positions other than our Moon's, and spacecraft positions, are approximate. Every number in the panels is the real value.</p>
    <h2>Reading the annotations</h2>
    <ul class="tss-legend">
      <li><span class="tss-badge tss-badge--fact">Verified fact</span> Established by measurement or observation; the source is named.</li>
      <li><span class="tss-badge tss-badge--leading">Leading hypothesis</span> The explanation most researchers favour, not yet proven.</li>
      <li><span class="tss-badge tss-badge--supported">Supported by evidence</span> Backed by direct evidence, but not beyond doubt.</li>
      <li><span class="tss-badge tss-badge--debated">Debated</span> Serious competing explanations.</li>
      <li><span class="tss-badge tss-badge--open">Open question</span> No accepted answer yet.</li>
      <li><span class="tss-badge tss-badge--disputed">Disputed</span> A claim later work has challenged.</li>
      <li><span class="tss-badge tss-badge--speculative">Speculative</span> An idea with little or no direct evidence.</li>
    </ul>
    <h2>Credits</h2>
    <p class="tss-small">Planet, Sun, Moon, ring and Milky Way maps: <a href="https://www.solarsystemscope.com/textures/" target="_blank" rel="noopener">Solar System Scope</a> (CC BY 4.0), based on NASA mission data. Orbits: NASA/JPL approximate planetary elements (E. M. Standish) and published elements for smaller bodies. Rotation: IAU Working Group on Cartographic Coordinates and Rotational Elements. Other surfaces are drawn procedurally as approximations. Written in Rust, drawn with WebGL2.</p>
    <div class="tss-actions"><button type="button" data-action="close-overlay" class="tss-primary">Back to the flight</button></div>
  </div>
</div>
<div class="tss-overlay tss-journal" hidden>
  <div class="tss-card tss-card--wide" role="dialog" aria-label="Journal">
    <h2>Journal</h2>
    <p class="tss-journal-count"></p>
    <ul class="tss-journal-list"></ul>
    <div class="tss-actions"><button type="button" data-action="close-overlay" class="tss-primary">Back to the flight</button></div>
  </div>
</div>
"#;

pub struct Ui {
    doc: Document,
    pub root: HtmlElement,
    pub canvas: HtmlCanvasElement,
    labels: Vec<Option<HtmlElement>>,
    label_shown: Vec<Option<(i32, i32)>>,
    hud: HtmlElement,
    bar: HtmlElement,
    date: HtmlElement,
    where_: HtmlElement,
    near: HtmlElement,
    warp: HtmlElement,
    pause: HtmlElement,
    orbits: HtmlElement,
    labels_button: HtmlElement,
    journal_button: HtmlElement,
    panel: HtmlElement,
    panel_scroll: HtmlElement,
    live: Option<HtmlElement>,
    toast: HtmlElement,
    toast_until: f64,
    intro: HtmlElement,
    help: HtmlElement,
    journal: HtmlElement,
    journal_count: HtmlElement,
    journal_items: Vec<HtmlElement>,
    texts: [String; 5],
}

fn find(root: &HtmlElement, sel: &str) -> HtmlElement {
    root.query_selector(sel).ok().flatten().and_then(|e| e.dyn_into().ok()).unwrap_or_else(|| panic!("missing {sel}"))
}

fn set_text(el: &HtmlElement, last: &mut String, text: String) {
    if *last != text {
        el.set_text_content(Some(&text));
        *last = text;
    }
}

impl Ui {
    pub fn new(doc: &Document, root: HtmlElement) -> Ui {
        root.class_list().add_1("tss").ok();
        root.set_tab_index(0);
        root.set_inner_html(TEMPLATE);
        let canvas: HtmlCanvasElement = find(&root, ".tss-canvas").unchecked_into();
        let layer = find(&root, ".tss-labels");
        let labels = BODIES
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let el: HtmlElement = doc.create_element("button").ok()?.dyn_into().ok()?;
                el.set_class_name(&format!("tss-label tss-label--{}", b.kind.css()));
                el.set_attribute("type", "button").ok();
                el.set_attribute("data-action", &format!("select:{i}")).ok();
                el.set_attribute("aria-label", &format!("{} ({})", b.name, b.kind.label())).ok();
                el.set_text_content(Some(b.name));
                el.set_hidden(true);
                layer.append_child(&el).ok();
                Some(el)
            })
            .collect();
        let journal_list = find(&root, ".tss-journal-list");
        let journal_items = BODIES
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let li = doc.create_element("li").unwrap();
                let button: HtmlElement = doc.create_element("button").unwrap().unchecked_into();
                button.set_attribute("type", "button").ok();
                button.set_attribute("data-action", &format!("fly:{i}")).ok();
                let mark = doc.create_element("span").unwrap();
                mark.set_class_name("tss-check");
                mark.set_attribute("aria-hidden", "true").ok();
                let name = doc.create_element("span").unwrap();
                name.set_text_content(Some(b.name));
                let kind = doc.create_element("span").unwrap();
                kind.set_class_name("tss-kind-small");
                kind.set_text_content(Some(b.kind.label()));
                button.append_child(&mark).ok();
                button.append_child(&name).ok();
                button.append_child(&kind).ok();
                li.append_child(&button).ok();
                journal_list.append_child(&li).ok();
                button
            })
            .collect();
        Ui {
            doc: doc.clone(),
            canvas,
            labels,
            label_shown: vec![None; BODIES.len()],
            hud: find(&root, ".tss-hud"),
            bar: find(&root, ".tss-bar"),
            date: find(&root, ".tss-date"),
            where_: find(&root, ".tss-where"),
            near: find(&root, ".tss-near"),
            warp: find(&root, ".tss-warp"),
            pause: find(&root, ".tss-pause"),
            orbits: find(&root, "[data-action=orbits]"),
            labels_button: find(&root, "[data-action=labels]"),
            journal_button: find(&root, ".tss-journal-button"),
            panel: find(&root, ".tss-panel"),
            panel_scroll: find(&root, ".tss-panel-scroll"),
            live: None,
            toast: find(&root, ".tss-toast"),
            toast_until: 0.0,
            intro: find(&root, ".tss-intro"),
            help: find(&root, ".tss-help"),
            journal: find(&root, ".tss-journal"),
            journal_count: find(&root, ".tss-journal-count"),
            journal_items,
            texts: Default::default(),
            root,
        }
    }

    // --- readout --------------------------------------------------------------

    pub fn set_hud(&mut self, date: String, warp: String, where_: String, near: String) {
        let [d, w, wh, n, _] = &mut self.texts;
        set_text(&self.date, d, date);
        set_text(&self.warp, w, warp);
        set_text(&self.where_, wh, where_);
        set_text(&self.near, n, near);
    }

    pub fn set_paused(&self, paused: bool) {
        self.pause.set_attribute("aria-pressed", if paused { "true" } else { "false" }).ok();
        self.pause.set_attribute("aria-label", if paused { "Resume time" } else { "Pause time" }).ok();
        self.pause.set_inner_html(if paused { "&#9654;" } else { "&#10074;&#10074;" });
    }

    pub fn set_toggles(&self, orbits: bool, labels: bool) {
        self.orbits.set_attribute("aria-pressed", if orbits { "true" } else { "false" }).ok();
        self.labels_button.set_attribute("aria-pressed", if labels { "true" } else { "false" }).ok();
    }

    // --- labels ---------------------------------------------------------------

    /// The open panel's rectangle (left, top, right, bottom) relative to
    /// the game's corner.
    pub fn panel_rect(&self) -> Option<(f64, f64, f64, f64)> {
        if self.panel.hidden() {
            return None;
        }
        let o = self.root.get_bounding_client_rect();
        let r = self.panel.get_bounding_client_rect();
        Some((r.left() - o.left(), r.top() - o.top(), r.right() - o.left(), r.bottom() - o.top()))
    }

    pub fn hud_bottom(&self) -> f64 {
        self.hud.get_bounding_client_rect().bottom() - self.root.get_bounding_client_rect().top()
    }

    /// Screen areas labels should keep clear of (left, top, right, bottom,
    /// in CSS pixels from the game's corner): the readout, the control bar,
    /// an open panel and the toast.
    pub fn obstacles(&self) -> Vec<(f64, f64, f64, f64)> {
        let origin = self.root.get_bounding_client_rect();
        [&self.hud, &self.bar, &self.panel, &self.toast]
            .into_iter()
            .filter(|e| !e.hidden() && (!e.class_list().contains("tss-toast") || e.class_list().contains("is-on")))
            .map(|e| e.get_bounding_client_rect())
            .filter(|r| r.width() > 0.0 && r.height() > 0.0)
            .map(|r| (r.left() - origin.left(), r.top() - origin.top(), r.right() - origin.left(), r.bottom() - origin.top()))
            .collect()
    }

    /// Show the label of body `i` at (x, y) CSS pixels, or hide it.
    pub fn place_label(&mut self, i: usize, at: Option<(f64, f64)>, selected: bool) {
        let Some(el) = &self.labels[i] else { return };
        let at = at.map(|(x, y)| (x.round() as i32, y.round() as i32));
        if self.label_shown[i] == at {
            return;
        }
        match at {
            Some((x, y)) => {
                el.style().set_property("transform", &format!("translate({x}px, {y}px)")).ok();
                if self.label_shown[i].is_none() {
                    el.set_hidden(false);
                }
            }
            None => el.set_hidden(true),
        }
        el.class_list().toggle_with_force("is-selected", selected).ok();
        self.label_shown[i] = at;
    }

    pub fn mark_selected_label(&self, selected: Option<usize>) {
        for (i, el) in self.labels.iter().enumerate() {
            if let Some(el) = el {
                el.class_list().toggle_with_force("is-selected", Some(i) == selected).ok();
            }
        }
    }

    // --- the information panel ------------------------------------------------

    pub fn show_panel(&mut self, i: usize, parent: Option<&Body>, visited: bool) {
        let b = &BODIES[i];
        let doc = &self.doc;
        let el = |tag: &str, class: &str, text: Option<&str>| -> HtmlElement {
            let e: HtmlElement = doc.create_element(tag).unwrap().unchecked_into();
            if !class.is_empty() {
                e.set_class_name(class);
            }
            if let Some(t) = text {
                e.set_text_content(Some(t));
            }
            e
        };
        let add = |to: &Element, child: &Element| {
            to.append_child(child).ok();
        };
        let scroll = &self.panel_scroll;
        scroll.set_text_content(None);

        let head = el("header", "tss-panel-head", None);
        let kind_line = match (b.kind, parent) {
            (Kind::Moon, Some(p)) => format!("Moon of {}", p.name.trim_start_matches("The ")),
            (Kind::Star, _) => "Our star".to_string(),
            _ => b.kind.label().to_string(),
        };
        let kind_line = if visited { format!("{kind_line} · in your journal") } else { kind_line };
        add(&head, &el("p", "tss-kind", Some(&kind_line)));
        let h = el("h2", "", Some(b.name));
        h.set_id("tss-panel-title");
        add(&head, &h);
        let close = el("button", "tss-close", Some("×"));
        close.set_attribute("type", "button").ok();
        close.set_attribute("data-action", "close").ok();
        close.set_attribute("aria-label", "Close panel").ok();
        add(&head, &close);
        add(scroll, &head);
        add(scroll, &el("p", "tss-tagline", Some(b.tagline)));

        let actions = el("div", "tss-actions", None);
        for (label, action, class) in [
            ("Fly here", format!("fly:{i}"), "tss-primary"),
            ("‹ Previous", "prev".to_string(), ""),
            ("Next ›", "next".to_string(), ""),
        ] {
            let button = el("button", class, Some(label));
            button.set_attribute("type", "button").ok();
            button.set_attribute("data-action", &action).ok();
            add(&actions, &button);
        }
        add(scroll, &actions);

        let live = el("p", "tss-live", None);
        add(scroll, &live);
        self.live = Some(live);
        self.texts[4].clear();

        let stats = el("dl", "tss-stats", None);
        for (k, v) in b.stats {
            let row = el("div", "", None);
            add(&row, &el("dt", "", Some(k)));
            add(&row, &el("dd", "", Some(v)));
            add(&stats, &row);
        }
        add(scroll, &stats);

        add(scroll, &el("h3", "", Some("Verified facts")));
        let facts = el("ul", "tss-facts", None);
        for f in b.facts {
            let li = el("li", "", None);
            add(&li, &el("p", "", Some(f.text)));
            add(&li, &el("p", "tss-source", Some(&format!("Source: {}", f.source))));
            add(&facts, &li);
        }
        add(scroll, &facts);

        add(scroll, &el("h3", "", Some("Hypotheses and open questions")));
        let ideas = el("ul", "tss-ideas", None);
        for idea in b.ideas {
            let li = el("li", "", None);
            add(&li, &el("span", &format!("tss-badge tss-badge--{}", idea.status.css()), Some(idea.status.label())));
            add(&li, &el("p", "", Some(idea.text)));
            add(&ideas, &li);
        }
        add(scroll, &ideas);

        add(scroll, &el("h3", "", Some(if b.kind == Kind::Spacecraft { "Notes" } else { "Exploration" })));
        let missions = el("ul", "tss-missions", None);
        for m in b.missions {
            add(&missions, &el("li", "", Some(m)));
        }
        add(scroll, &missions);

        self.panel.set_hidden(false);
        self.panel.set_attribute("aria-labelledby", "tss-panel-title").ok();
        self.panel_scroll.set_scroll_top(0);
        self.mark_selected_label(Some(i));
    }

    pub fn set_live(&mut self, text: String) {
        if let Some(live) = &self.live {
            live.set_hidden(text.is_empty());
            set_text(live, &mut self.texts[4], text);
        }
    }

    pub fn hide_panel(&mut self) {
        self.panel.set_hidden(true);
        self.live = None;
        self.mark_selected_label(None);
    }

    pub fn panel_open(&self) -> bool {
        !self.panel.hidden()
    }

    // --- toasts and overlays ----------------------------------------------------

    pub fn toast(&mut self, text: &str, now: f64) {
        self.toast.set_text_content(Some(text));
        self.toast.class_list().add_1("is-on").ok();
        self.toast_until = now + 3800.0;
    }

    pub fn tick(&mut self, now: f64) {
        if self.toast_until > 0.0 && now > self.toast_until {
            self.toast.class_list().remove_1("is-on").ok();
            self.toast_until = 0.0;
        }
    }

    pub fn hide_intro(&self) {
        self.intro.set_hidden(true);
    }

    pub fn intro_open(&self) -> bool {
        !self.intro.hidden()
    }

    pub fn show_help(&self, on: bool) {
        self.help.set_hidden(!on);
        if on {
            self.journal.set_hidden(true);
            if let Ok(Some(b)) = self.help.query_selector("[data-action=close-overlay]") {
                b.unchecked_into::<HtmlElement>().focus().ok();
            }
        }
    }

    pub fn help_open(&self) -> bool {
        !self.help.hidden()
    }

    pub fn show_journal(&self, on: bool, visited: &[bool]) {
        self.journal.set_hidden(!on);
        if on {
            self.help.set_hidden(true);
            self.update_journal(visited);
        }
    }

    pub fn journal_open(&self) -> bool {
        !self.journal.hidden()
    }

    pub fn update_journal(&self, visited: &[bool]) {
        let n = visited.iter().filter(|&&v| v).count();
        self.journal_button.set_text_content(Some(&format!("Journal {n}/{}", visited.len())));
        self.journal_count.set_text_content(Some(&format!(
            "{n} of {} charted. Fly close to an object, or arrive at a region, to add it.",
            visited.len()
        )));
        for (item, &v) in self.journal_items.iter().zip(visited) {
            item.class_list().toggle_with_force("is-visited", v).ok();
            if let Ok(Some(mark)) = item.query_selector(".tss-check") {
                mark.set_text_content(Some(if v { "✓" } else { "○" }));
            }
        }
    }

    pub fn close_overlays(&self) {
        self.help.set_hidden(true);
        self.journal.set_hidden(true);
        self.root.focus().ok();
    }
}
