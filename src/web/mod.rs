//! The browser side: `start` builds the game inside a container element and
//! runs it with requestAnimationFrame.

mod app;
mod gl;
mod render;
mod ui;

use app::{Action, App, Input, Queue};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{AddEventListenerOptions, Element, Event, HtmlElement, KeyboardEvent, PointerEvent, WebGl2RenderingContext, WheelEvent};

fn listen<E: JsCast + 'static>(target: &web_sys::EventTarget, kind: &str, passive: bool, f: impl FnMut(E) + 'static) {
    let mut f = f;
    let closure = Closure::<dyn FnMut(Event)>::new(move |e: Event| f(e.unchecked_into()));
    let opts = AddEventListenerOptions::new();
    opts.set_passive(passive);
    target
        .add_event_listener_with_callback_and_add_event_listener_options(kind, closure.as_ref().unchecked_ref(), &opts)
        .ok();
    closure.forget();
}

/// The `data-action` of the clicked element or its nearest ancestor.
fn action_of(e: &Event) -> Option<String> {
    let target: Element = e.target()?.dyn_into().ok()?;
    target.closest("[data-action]").ok()??.get_attribute("data-action")
}

/// Keys the game uses; their default (scrolling the page) is suppressed.
const GAME_KEYS: [&str; 6] = ["Space", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Slash"];

/// Starts the game inside `container`. `options` may set `assets` (the
/// folder holding the texture maps, default `textures/`) and `textures` (a
/// map from texture name to URL, used by the single-file build).
#[wasm_bindgen]
pub fn start(container: HtmlElement, options: JsValue) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let window = web_sys::window().ok_or("no window")?;
    let doc = window.document().ok_or("no document")?;

    let ui = ui::Ui::new(&doc, container.clone());
    let ctx_opts = js_sys::Object::new();
    js_sys::Reflect::set(&ctx_opts, &"antialias".into(), &true.into())?;
    js_sys::Reflect::set(&ctx_opts, &"powerPreference".into(), &"high-performance".into())?;
    let gl: WebGl2RenderingContext = match ui.canvas.get_context_with_context_options("webgl2", &ctx_opts) {
        Ok(Some(ctx)) => ctx.dyn_into()?,
        _ => {
            container.set_text_content(Some(
                "This game needs WebGL2, which this browser or device does not provide. Try a recent version of Firefox, Chrome, Edge or Safari.",
            ));
            container.class_list().add_1("tss-unsupported").ok();
            return Err("WebGL2 is not available".into());
        }
    };
    let renderer = render::Renderer::new(gl).map_err(|e| JsValue::from_str(&e))?;

    let queue: Queue = Rc::new(RefCell::new(Default::default()));
    let canvas = ui.canvas.clone();
    let root = ui.root.clone();

    // Buttons anywhere in the game carry a data-action.
    let q = queue.clone();
    listen(&root, "click", true, move |e: Event| {
        if let Some(a) = action_of(&e).and_then(|s| Action::parse(&s)) {
            q.borrow_mut().push_back(Input::Action(a));
        }
    });
    // Double-clicking a label flies there.
    let q = queue.clone();
    listen(&root, "dblclick", true, move |e: Event| {
        if let Some(Action::Select(i)) = action_of(&e).and_then(|s| Action::parse(&s)) {
            q.borrow_mut().push_back(Input::Action(Action::Fly(i)));
        }
    });

    let q = queue.clone();
    let focus_root = root.clone();
    listen(&canvas, "pointerdown", true, move |e: PointerEvent| {
        if let Some(t) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) {
            t.set_pointer_capture(e.pointer_id()).ok();
        }
        focus_root.focus().ok();
        q.borrow_mut().push_back(Input::Down { id: e.pointer_id(), x: e.offset_x() as f64, y: e.offset_y() as f64 });
    });
    let q = queue.clone();
    listen(&canvas, "pointermove", true, move |e: PointerEvent| {
        q.borrow_mut().push_back(Input::Move { id: e.pointer_id(), x: e.offset_x() as f64, y: e.offset_y() as f64 });
    });
    for kind in ["pointerup", "pointercancel"] {
        let q = queue.clone();
        listen(&canvas, kind, true, move |e: PointerEvent| {
            q.borrow_mut().push_back(Input::Up { id: e.pointer_id(), x: e.offset_x() as f64, y: e.offset_y() as f64 });
        });
    }
    let q = queue.clone();
    listen(&canvas, "dblclick", true, move |e: web_sys::MouseEvent| {
        q.borrow_mut().push_back(Input::DoubleClick { x: e.offset_x() as f64, y: e.offset_y() as f64 });
    });
    let q = queue.clone();
    listen(&canvas, "wheel", false, move |e: WheelEvent| {
        e.prevent_default();
        let scale = match e.delta_mode() {
            1 => 16.0,
            2 => 400.0,
            _ => 1.0,
        };
        q.borrow_mut().push_back(Input::Wheel { dy: e.delta_y() * scale });
    });
    for (kind, down) in [("keydown", true), ("keyup", false)] {
        let q = queue.clone();
        listen(&root, kind, false, move |e: KeyboardEvent| {
            let code = e.code();
            if e.ctrl_key() || e.meta_key() || e.alt_key() {
                return;
            }
            // Let buttons respond to Enter and Space themselves.
            let on_button = e
                .target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .is_some_and(|t| t.tag_name() == "BUTTON" || t.tag_name() == "A");
            if on_button && (code == "Enter" || code == "Space" || code == "NumpadEnter") {
                return;
            }
            if GAME_KEYS.contains(&code.as_str()) {
                e.prevent_default();
            }
            q.borrow_mut().push_back(Input::Key { code, down });
        });
    }
    let q = queue.clone();
    listen(&window, "blur", true, move |_: Event| q.borrow_mut().push_back(Input::Blur));

    let storage = window.local_storage().ok().flatten();
    let reduced_motion = window
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|m| m.matches());

    let app = App::new(renderer, ui, queue, js_sys::Date::now(), reduced_motion, storage);

    let base = js_sys::Reflect::get(&options, &"assets".into())
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "textures/".to_string());
    let overrides = js_sys::Reflect::get(&options, &"textures".into()).ok().filter(|v| v.is_object());
    app.load_textures(|name| {
        overrides
            .as_ref()
            .and_then(|o| js_sys::Reflect::get(o, &name.into()).ok())
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| format!("{base}{name}.webp"))
    });

    let app = Rc::new(RefCell::new(app));
    let tick: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let next = tick.clone();
    let win = window.clone();
    *tick.borrow_mut() = Some(Closure::new(move |t: f64| {
        let w = root.client_width().max(1) as f64;
        let h = root.client_height().max(1) as f64;
        let dpr = win.device_pixel_ratio().clamp(1.0, 2.0);
        let (pw, ph) = ((w * dpr).round() as u32, (h * dpr).round() as u32);
        if canvas.width() != pw || canvas.height() != ph {
            canvas.set_width(pw);
            canvas.set_height(ph);
        }
        app.borrow_mut().frame(t, w, h, dpr);
        if let Some(cb) = next.borrow().as_ref() {
            win.request_animation_frame(cb.as_ref().unchecked_ref()).ok();
        }
    }));
    if let Some(cb) = tick.borrow().as_ref() {
        window.request_animation_frame(cb.as_ref().unchecked_ref())?;
    }
    Ok(())
}
