//! Draws a frame: sky, bodies, orbits, swarms, rings, glows and markers.

use super::gl::{self, Program, NOISE};
use crate::astro;
use crate::belts;
use crate::camera::Camera;
use crate::catalog::{Kind, Motion, Shade, BODIES};
use crate::mesh;
use crate::sim::Sim;
use glam::{DVec3, Mat3, Vec3};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{HtmlImageElement, WebGl2RenderingContext as GL, WebGlTexture, WebGlVertexArrayObject};

const NEAR: f32 = 1e-5;
const FAR: f32 = 1e6;

struct Mesh {
    vao: WebGlVertexArrayObject,
    count: i32,
}

struct OrbitLine {
    vao: WebGlVertexArrayObject,
    count: i32,
    around: usize,
}

pub struct Scene<'a> {
    pub cam: &'a Camera,
    pub sim: &'a Sim,
    pub width: f64,
    pub height: f64,
    pub dpr: f64,
    pub selected: Option<usize>,
    pub show_orbits: bool,
    pub seconds: f64,
}

pub struct Renderer {
    gl: GL,
    planet: Program,
    halo: Program,
    ring: Program,
    sky: Program,
    stars: Program,
    belt: Program,
    line: Program,
    marker: Program,
    glow: Program,
    shell: Program,
    sphere: Mesh,
    sphere_lo: Mesh,
    annulus: Mesh,
    empty: WebGlVertexArrayObject,
    star_points: Mesh,
    inner: Mesh,
    outer: Mesh,
    oort: Mesh,
    termination_shock: Mesh,
    heliopause: Mesh,
    orbits: Vec<Option<OrbitLine>>,
    orbits_days: f64,
    markers_vao: WebGlVertexArrayObject,
    markers_buf: web_sys::WebGlBuffer,
    marker_data: Vec<f32>,
    children: Vec<Vec<usize>>,
    textures: HashMap<&'static str, WebGlTexture>,
    white: WebGlTexture,
    black: WebGlTexture,
    pending: Rc<RefCell<Vec<(&'static str, HtmlImageElement)>>>,
    anisotropy: f32,
}

fn indexed(gl: &GL, verts: &[f32], idx: &[u16], layout: &[(u32, i32)]) -> Mesh {
    Mesh { vao: gl::vertex_array(gl, verts, layout, Some(idx)), count: idx.len() as i32 }
}

fn points(gl: &GL, data: &[f32], layout: &[(u32, i32)]) -> Mesh {
    let stride: i32 = layout.iter().map(|&(_, n)| n).sum();
    Mesh { vao: gl::vertex_array(gl, data, layout, None), count: data.len() as i32 / stride }
}

/// Every texture the catalogue asks for, plus the Milky Way.
pub fn texture_names() -> Vec<&'static str> {
    let mut names = vec!["stars_milky_way"];
    for b in BODIES {
        let l = &b.look;
        for n in [l.tex, l.night, l.clouds, l.rings.and_then(|r| r.tex)].into_iter().flatten() {
            if !names.contains(&n) {
                names.push(n);
            }
        }
    }
    names
}

impl Renderer {
    pub fn new(gl: GL) -> Result<Renderer, String> {
        let planet_vs = [NOISE, include_str!("shaders/planet.vert")];
        let p = |name, vs: &[&str], fs: &[&str]| Program::new(&gl, name, vs, fs);
        let planet = p("planet", &planet_vs, &[NOISE, include_str!("shaders/planet.frag")])?;
        let halo = p("halo", &planet_vs, &[include_str!("shaders/halo.frag")])?;
        let ring = p("ring", &[include_str!("shaders/ring.vert")], &[include_str!("shaders/ring.frag")])?;
        let sky = p("sky", &[include_str!("shaders/sky.vert")], &[include_str!("shaders/sky.frag")])?;
        let stars = p("stars", &[include_str!("shaders/stars.vert")], &[include_str!("shaders/stars.frag")])?;
        let belt = p("belt", &[include_str!("shaders/belt.vert")], &[include_str!("shaders/points.frag")])?;
        let line = p("line", &[include_str!("shaders/line.vert")], &[include_str!("shaders/line.frag")])?;
        let marker = p("marker", &[include_str!("shaders/marker.vert")], &[include_str!("shaders/marker.frag")])?;
        let glow = p("glow", &[include_str!("shaders/glow.vert")], &[include_str!("shaders/glow.frag")])?;
        let shell = p("shell", &[include_str!("shaders/shell.vert")], &[include_str!("shaders/shell.frag")])?;

        let (sv, si) = mesh::sphere(128, 64);
        let sphere = indexed(&gl, &sv, &si, &[(0, 3), (1, 2)]);
        let (sv, si) = mesh::sphere(64, 32);
        let sphere_lo = indexed(&gl, &sv, &si, &[(0, 3), (1, 2)]);
        let ann = mesh::annulus(256);
        let annulus = Mesh { vao: gl::vertex_array(&gl, &ann, &[(0, 3)], None), count: ann.len() as i32 / 3 };
        let empty = gl.create_vertex_array().ok_or("vao")?;
        let star_points = points(&gl, &mesh::stars(9_000), &[(0, 3), (1, 1), (2, 3)]);
        let belt_layout = [(0, 4), (1, 2), (2, 3)];
        let inner = points(&gl, &belts::inner_belts(), &belt_layout);
        let outer = points(&gl, &belts::outer_belts(), &belt_layout);
        let oort = points(&gl, &belts::oort_cloud(), &belt_layout);
        let (hv, hi) = mesh::heliosphere(90.0, 0.9, 96, 48);
        let termination_shock = indexed(&gl, &hv, &hi, &[(0, 3), (1, 3)]);
        let (hv, hi) = mesh::heliosphere(121.0, 1.6, 96, 48);
        let heliopause = indexed(&gl, &hv, &hi, &[(0, 3), (1, 3)]);

        let markers_vao = gl.create_vertex_array().ok_or("vao")?;
        gl.bind_vertex_array(Some(&markers_vao));
        let markers_buf = gl::array_buffer(&gl, &[0.0; 8], GL::DYNAMIC_DRAW);
        for (loc, n, off) in [(0u32, 3, 0), (1, 4, 12), (2, 1, 28)] {
            gl.enable_vertex_attrib_array(loc);
            gl.vertex_attrib_pointer_with_i32(loc, n, GL::FLOAT, false, 32, off);
        }
        gl.bind_vertex_array(None);

        let mut children = vec![Vec::new(); BODIES.len()];
        for (i, b) in BODIES.iter().enumerate() {
            if matches!(b.motion, Motion::Moon { .. } | Motion::Lunar) {
                if let Some(p) = b.parent.and_then(crate::catalog::index_of) {
                    children[p].push(i);
                }
            }
        }
        for c in &mut children {
            c.sort_by(|&a, &b| BODIES[b].radius_km.total_cmp(&BODIES[a].radius_km));
            c.truncate(4);
        }

        let anisotropy = if gl.get_extension("EXT_texture_filter_anisotropic").ok().flatten().is_some() { 8.0 } else { 1.0 };
        let white = gl::solid_texture(&gl, [255, 255, 255, 255]);
        let black = gl::solid_texture(&gl, [0, 0, 0, 255]);
        Ok(Renderer {
            planet,
            halo,
            ring,
            sky,
            stars,
            belt,
            line,
            marker,
            glow,
            shell,
            sphere,
            sphere_lo,
            annulus,
            empty,
            star_points,
            inner,
            outer,
            oort,
            termination_shock,
            heliopause,
            orbits: Vec::new(),
            orbits_days: f64::NAN,
            markers_vao,
            markers_buf,
            marker_data: Vec::new(),
            children,
            textures: HashMap::new(),
            white,
            black,
            pending: Rc::new(RefCell::new(Vec::new())),
            anisotropy,
            gl,
        })
    }

    /// Starts downloading the texture maps; each appears once it arrives.
    pub fn load_textures(&self, url_for: impl Fn(&str) -> String) {
        for name in texture_names() {
            let Ok(img) = HtmlImageElement::new() else { continue };
            let pending = self.pending.clone();
            let loaded = img.clone();
            let onload = Closure::once(move || pending.borrow_mut().push((name, loaded)));
            img.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            img.set_src(&url_for(name));
        }
    }

    fn upload_textures(&mut self) {
        let ready: Vec<_> = self.pending.borrow_mut().drain(..).collect();
        for (name, img) in ready {
            let wrap = if name.contains("ring") { GL::CLAMP_TO_EDGE } else { GL::REPEAT };
            let tex = gl::image_texture(&self.gl, &img, wrap, self.anisotropy);
            self.textures.insert(name, tex);
        }
    }

    fn tex(&self, name: Option<&str>, fallback: &WebGlTexture) -> (bool, WebGlTexture) {
        match name.and_then(|n| self.textures.get(n)) {
            Some(t) => (true, t.clone()),
            None => (false, fallback.clone()),
        }
    }

    fn build_orbits(&mut self, sim: &Sim) {
        let gl = &self.gl;
        self.orbits = (0..BODIES.len())
            .map(|i| {
                let b = &BODIES[i];
                let (pts, around): (Vec<DVec3>, usize) = match b.motion {
                    Motion::Orbit(_) => {
                        let n = if b.kind == Kind::Planet { 512 } else { 1024 };
                        let path = sim.kepler[i]?.path(sim.days, n);
                        (path.into_iter().map(astro::to_display).collect(), 0)
                    }
                    Motion::Moon { .. } | Motion::Lunar => {
                        let p = sim.parent[i]?;
                        let s = &sim.states[i];
                        let d = (s.pos - sim.states[p].pos).length();
                        let n = s.orbit_normal;
                        let e1 = n.any_orthonormal_vector();
                        let e2 = n.cross(e1);
                        let pts = (0..256)
                            .map(|k| {
                                let t = std::f64::consts::TAU * k as f64 / 256.0;
                                (e1 * t.cos() + e2 * t.sin()) * d
                            })
                            .collect();
                        (pts, p)
                    }
                    _ => return None,
                };
                let flat: Vec<f32> = pts.iter().flat_map(|p| [p.x as f32, p.y as f32, p.z as f32]).collect();
                Some(OrbitLine { vao: gl::vertex_array(gl, &flat, &[(0, 3)], None), count: pts.len() as i32, around })
            })
            .collect();
        self.orbits_days = sim.days;
    }

    pub fn draw(&mut self, s: &Scene) {
        self.upload_textures();
        if !((s.sim.days - self.orbits_days).abs() < 120.0) {
            self.build_orbits(s.sim);
        }
        let gl = self.gl.clone();
        let gl = &gl;
        let (w, h) = ((s.width * s.dpr).round() as i32, (s.height * s.dpr).round() as i32);
        gl.viewport(0, 0, w, h);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.depth_mask(true);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let cam = s.cam;
        let proj = cam.projection(s.width / s.height, NEAR, FAR);
        let view = cam.view_rotation();
        let vp = proj * view;
        let logf = 1.0 / (FAR + 1.0).log2();
        let rel = |p: DVec3| (p - cam.pos).as_vec3();
        let px_per_unit = cam.pixels_per_unit(1.0, s.height); // CSS px at distance 1
        let sun_rel = rel(DVec3::ZERO);
        let cam_au = Sim::au_from_sun(cam.pos);

        // --- the sky ---------------------------------------------------------
        gl.disable(GL::DEPTH_TEST);
        gl.depth_mask(false);
        gl.disable(GL::CULL_FACE);
        gl.disable(GL::BLEND);
        if let Some(t) = self.textures.get("stars_milky_way") {
            let to_gal = |v: DVec3| {
                astro::equatorial_to_galactic(astro::ecliptic_to_equatorial(astro::render_to_ecl(v))).as_vec3()
            };
            self.sky.bind(gl);
            self.sky.m4(gl, "u_invViewProj", vp.inverse());
            self.sky.m3(gl, "u_toGalactic", Mat3::from_cols(to_gal(DVec3::X), to_gal(DVec3::Y), to_gal(DVec3::Z)));
            self.sky.f1(gl, "u_intensity", 0.3);
            gl.active_texture(GL::TEXTURE0);
            gl.bind_texture(GL::TEXTURE_2D, Some(t));
            self.sky.i1(gl, "u_tex", 0);
            gl.bind_vertex_array(Some(&self.empty));
            gl.draw_arrays(GL::TRIANGLES, 0, 3);
        }
        gl.enable(GL::BLEND);
        gl.blend_func(GL::ONE, GL::ONE);
        self.stars.bind(gl);
        self.stars.m4(gl, "u_viewProj", vp);
        self.stars.f1(gl, "u_pointScale", s.dpr as f32);
        gl.bind_vertex_array(Some(&self.star_points.vao));
        gl.draw_arrays(GL::POINTS, 0, self.star_points.count);

        // --- solid bodies ------------------------------------------------------
        gl.enable(GL::DEPTH_TEST);
        gl.depth_func(GL::LEQUAL);
        gl.depth_mask(true);
        gl.disable(GL::BLEND);
        gl.enable(GL::CULL_FACE);
        gl.cull_face(GL::BACK);
        let pr = &self.planet;
        pr.bind(gl);
        pr.m4(gl, "u_viewProj", vp);
        pr.f1(gl, "u_logF", logf);
        pr.v3(gl, "u_sun", sun_rel);
        pr.f1(gl, "u_sunRadius", astro::SUN_DISPLAY_RADIUS as f32);
        for (unit, name) in ["u_tex0", "u_tex1", "u_tex2", "u_ringTex"].into_iter().enumerate() {
            pr.i1(gl, name, unit as i32);
        }
        gl.bind_vertex_array(Some(&self.sphere.vao));
        let forward = cam.forward();
        let mut ghosts = Vec::new();
        for (i, b) in BODIES.iter().enumerate() {
            let st = &s.sim.states[i];
            if !st.exists || st.radius <= 0.0 || b.look.shade == Shade::None {
                continue;
            }
            let c = st.pos - cam.pos;
            let depth = c.dot(forward);
            if depth < -st.radius * 2.5 || st.radius * px_per_unit / c.length().max(1e-9) < 0.4 {
                continue;
            }
            if b.look.shade == Shade::Ghost {
                ghosts.push(i);
                continue;
            }
            self.draw_body(s, i, rel(st.pos), sun_rel, &rel);
        }

        // --- orbits --------------------------------------------------------
        gl.disable(GL::CULL_FACE);
        gl.enable(GL::BLEND);
        gl.depth_mask(false);
        if s.show_orbits {
            gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
            self.line.bind(gl);
            self.line.m4(gl, "u_viewProj", vp);
            self.line.f1(gl, "u_logF", logf);
            for (i, line) in self.orbits.iter().enumerate() {
                let Some(line) = line else { continue };
                let b = &BODIES[i];
                let around = s.sim.states[line.around].pos;
                let mut color = match b.kind {
                    Kind::Planet => [0.42, 0.58, 0.85, 0.4],
                    Kind::Moon => [0.6, 0.62, 0.72, 0.3],
                    Kind::Comet => [0.45, 0.85, 1.0, 0.22],
                    _ => [0.8, 0.62, 0.45, 0.17],
                };
                if b.kind == Kind::Moon {
                    let near = s.sim.view_distance(line.around) * 10.0;
                    let d = (cam.pos - around).length();
                    color[3] *= (1.0 - ((d - near) / (near * 2.0)).clamp(0.0, 1.0)) as f32;
                    if color[3] <= 0.01 {
                        continue;
                    }
                }
                if s.selected == Some(i) {
                    color = [0.75, 0.9, 0.75, 0.85];
                }
                self.line.v3(gl, "u_origin", rel(around));
                self.line.f4(gl, "u_color", color);
                gl.bind_vertex_array(Some(&line.vao));
                gl.draw_arrays(GL::LINE_LOOP, 0, line.count);
            }
        }

        // --- asteroid belt, Kuiper belt, Oort cloud -----------------------------
        gl.blend_func(GL::ONE, GL::ONE);
        self.belt.bind(gl);
        self.belt.m4(gl, "u_viewProj", vp);
        self.belt.f1(gl, "u_logF", logf);
        self.belt.v3(gl, "u_cam", cam.pos.as_vec3());
        self.belt.f1(gl, "u_days", s.sim.days as f32);
        self.belt.f1(gl, "u_scale", astro::DISTANCE_SCALE as f32);
        self.belt.f1(gl, "u_pxPerUnit", (px_per_unit * s.dpr) as f32);
        let smooth = |a: f64, b: f64, x: f64| crate::camera::smoothstep((x - a) / (b - a));
        for (mesh, size, alpha) in [
            (&self.inner, 0.05, 0.85 * (1.0 - smooth(400.0, 3000.0, cam_au))),
            (&self.outer, 0.5, 0.85 * (1.0 - smooth(5_000.0, 40_000.0, cam_au))),
            (&self.oort, 40.0, 0.9 * smooth(300.0, 2_500.0, cam_au)),
        ] {
            if alpha > 0.005 {
                self.belt.f1(gl, "u_size", size);
                self.belt.f1(gl, "u_alpha", alpha as f32);
                gl.bind_vertex_array(Some(&mesh.vao));
                gl.draw_arrays(GL::POINTS, 0, mesh.count);
            }
        }

        // --- rings ---------------------------------------------------------
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        self.ring.bind(gl);
        self.ring.m4(gl, "u_viewProj", vp);
        self.ring.f1(gl, "u_logF", logf);
        self.ring.v3(gl, "u_sun", sun_rel);
        self.ring.i1(gl, "u_ringTex", 0);
        gl.bind_vertex_array(Some(&self.annulus.vao));
        for (i, b) in BODIES.iter().enumerate() {
            let Some(rings) = b.look.rings else { continue };
            let st = &s.sim.states[i];
            let c = st.pos - cam.pos;
            let r_eq = st.radius * b.look.shape[0] as f64;
            if r_eq * rings.outer as f64 * px_per_unit / c.length() < 1.0 {
                continue;
            }
            let (has, tex) = self.tex(rings.tex, &self.white);
            gl.active_texture(GL::TEXTURE0);
            gl.bind_texture(GL::TEXTURE_2D, Some(&tex));
            self.ring.i1(gl, "u_hasTex", has as i32);
            self.ring.v3(gl, "u_center", rel(st.pos));
            self.ring.m3(gl, "u_basis", basis(st));
            self.ring.f1(gl, "u_radius", r_eq as f32);
            self.ring.f2(gl, "u_radii", rings.inner, rings.outer);
            self.ring.rgb(gl, "u_color", rings.color);
            self.ring.f1(gl, "u_opacity", rings.opacity);
            self.ring.f1(gl, "u_planetR", r_eq as f32);
            self.ring.v3(gl, "u_normal", st.frame.y.as_vec3());
            gl.draw_arrays(GL::TRIANGLE_STRIP, 0, self.annulus.count);
        }

        // --- atmospheres seen edge-on ------------------------------------------
        gl.blend_func(GL::ONE, GL::ONE);
        gl.enable(GL::CULL_FACE);
        gl.cull_face(GL::FRONT);
        self.halo.bind(gl);
        self.halo.m4(gl, "u_viewProj", vp);
        self.halo.f1(gl, "u_logF", logf);
        self.halo.v3(gl, "u_sun", sun_rel);
        self.halo.f1(gl, "u_bumpy", 0.0);
        self.halo.v3(gl, "u_offset", Vec3::ZERO);
        gl.bind_vertex_array(Some(&self.sphere_lo.vao));
        for (i, b) in BODIES.iter().enumerate() {
            let atmo = b.look.atmosphere;
            let st = &s.sim.states[i];
            if atmo[3] <= 0.0 || !st.exists {
                continue;
            }
            let c = st.pos - cam.pos;
            if st.radius * px_per_unit / c.length() < 3.0 || c.dot(forward) < -st.radius * 2.0 {
                continue;
            }
            let thick = st.radius * (0.04 + 0.08 * (atmo[3] as f64).min(1.2));
            let shell = (st.radius + thick) as f32;
            self.halo.v3(gl, "u_center", rel(st.pos));
            self.halo.m3(gl, "u_basis", basis(st));
            self.halo.v3(gl, "u_scale", Vec3::splat(shell));
            self.halo.f1(gl, "u_radius", st.radius as f32);
            self.halo.f1(gl, "u_thick", thick as f32);
            self.halo.f4(gl, "u_atmo", atmo);
            gl.draw_elements_with_i32(GL::TRIANGLES, self.sphere_lo.count, GL::UNSIGNED_SHORT, 0);
        }

        // --- hypothetical worlds ------------------------------------------------
        gl.cull_face(GL::BACK);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        if !ghosts.is_empty() {
            self.planet.bind(gl);
            gl.bind_vertex_array(Some(&self.sphere.vao));
            for &i in &ghosts {
                let st = &s.sim.states[i];
                self.draw_body(s, i, rel(st.pos), sun_rel, &rel);
            }
        }
        gl.disable(GL::CULL_FACE);

        // --- the heliosphere --------------------------------------------------
        let shell_alpha = smooth(12.0, 70.0, cam_au) * (1.0 - smooth(8_000.0, 60_000.0, cam_au));
        if shell_alpha > 0.005 {
            gl.blend_func(GL::ONE, GL::ONE);
            self.shell.bind(gl);
            self.shell.m4(gl, "u_viewProj", vp);
            self.shell.f1(gl, "u_logF", logf);
            self.shell.v3(gl, "u_origin", sun_rel);
            for (mesh, color, a) in [
                (&self.termination_shock, [0.3, 0.55, 1.0], 0.3),
                (&self.heliopause, [1.0, 0.62, 0.32], 0.45),
            ] {
                self.shell.rgb(gl, "u_color", color);
                self.shell.f1(gl, "u_alpha", (a * shell_alpha) as f32);
                gl.bind_vertex_array(Some(&mesh.vao));
                gl.draw_elements_with_i32(GL::TRIANGLES, mesh.count, GL::UNSIGNED_SHORT, 0);
            }
        }

        // --- glows: the Sun, comets ------------------------------------------
        gl.blend_func(GL::ONE, GL::ONE);
        self.glow.bind(gl);
        self.glow.m4(gl, "u_proj", proj);
        self.glow.f1(gl, "u_logF", logf);
        gl.bind_vertex_array(Some(&self.empty));
        let to_view = |v: Vec3| (view * v.extend(0.0)).truncate();
        for (i, b) in BODIES.iter().enumerate() {
            if b.kind != Kind::Comet {
                continue;
            }
            let st = &s.sim.states[i];
            let r = st.au.length();
            let activity = ((5.0 - r) / 4.4).clamp(0.0, 1.0).powf(1.5);
            if activity <= 0.0 {
                continue;
            }
            let away = st.au.normalize();
            let tip = astro::to_display(st.au + away * (0.35 * activity / r.max(0.5).sqrt()));
            let tail = tip - st.pos;
            let len = tail.length().max(1e-6);
            let center_view = to_view(rel(st.pos + tail * 0.5));
            let axis_x = to_view((tail * 0.5).as_vec3());
            let axis_y = axis_x.cross(center_view).normalize_or_zero() * (len * 0.16) as f32;
            self.glow.v3(gl, "u_centerView", center_view);
            self.glow.v3(gl, "u_axisX", axis_x);
            self.glow.v3(gl, "u_axisY", axis_y);
            self.glow.i1(gl, "u_mode", 1);
            self.glow.rgb(gl, "u_color", [0.55, 0.8, 1.0]);
            self.glow.f1(gl, "u_strength", activity as f32 * 0.9);
            gl.draw_arrays(GL::TRIANGLE_STRIP, 0, 4);
            let coma = (len * 0.12) as f32;
            self.glow.v3(gl, "u_centerView", to_view(rel(st.pos)));
            self.glow.v3(gl, "u_axisX", Vec3::new(coma, 0.0, 0.0));
            self.glow.v3(gl, "u_axisY", Vec3::new(0.0, coma, 0.0));
            self.glow.i1(gl, "u_mode", 2);
            self.glow.rgb(gl, "u_color", [0.75, 0.95, 1.0]);
            gl.draw_arrays(GL::TRIANGLE_STRIP, 0, 4);
        }
        let sun_dist = cam.pos.length().max(1e-6);
        let glare = (astro::SUN_DISPLAY_RADIUS * 7.0).max(70.0 / (px_per_unit / sun_dist)) as f32;
        self.glow.v3(gl, "u_centerView", to_view(sun_rel));
        self.glow.v3(gl, "u_axisX", Vec3::new(glare, 0.0, 0.0));
        self.glow.v3(gl, "u_axisY", Vec3::new(0.0, glare, 0.0));
        self.glow.i1(gl, "u_mode", 0);
        self.glow.rgb(gl, "u_color", [1.0, 0.82, 0.55]);
        self.glow.f1(gl, "u_strength", 1.0);
        gl.draw_arrays(GL::TRIANGLE_STRIP, 0, 4);

        // --- markers ---------------------------------------------------------
        self.marker_data.clear();
        for (i, b) in BODIES.iter().enumerate() {
            let st = &s.sim.states[i];
            if !st.exists || b.is_region() {
                continue;
            }
            let c = st.pos - cam.pos;
            let dist = c.length();
            if c.dot(forward) <= 0.0 {
                continue;
            }
            let px = st.radius * px_per_unit / dist;
            let toward = (c - c / dist * (st.radius * 1.05)).as_vec3();
            let (size, shape) = match b.kind {
                Kind::Spacecraft if px < 8.0 => (11.0, 1.0),
                Kind::Hypothetical => ((px * 2.4).max(18.0), 2.0),
                _ if px < 2.0 => (
                    match b.kind {
                        Kind::Star => 8.0,
                        Kind::Planet => 5.0,
                        Kind::DwarfPlanet => 4.0,
                        _ => 3.0,
                    },
                    0.0,
                ),
                _ => (0.0, 0.0),
            };
            if size > 0.0 {
                let d = b.look.dot;
                self.marker_data.extend([toward.x, toward.y, toward.z, d[0], d[1], d[2], (size * s.dpr) as f32, shape]);
            }
            if s.selected == Some(i) && px < 60.0 {
                let size = ((px * 2.6 + 14.0).max(26.0) * s.dpr).min(120.0 * s.dpr) as f32;
                self.marker_data.extend([toward.x, toward.y, toward.z, 0.75, 0.92, 0.75, size, 3.0]);
            }
        }
        if !self.marker_data.is_empty() {
            gl.blend_func(GL::ONE, GL::ONE_MINUS_SRC_ALPHA);
            self.marker.bind(gl);
            self.marker.m4(gl, "u_viewProj", vp);
            self.marker.f1(gl, "u_logF", logf);
            gl.bind_vertex_array(Some(&self.markers_vao));
            gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.markers_buf));
            gl.buffer_data_with_array_buffer_view(
                GL::ARRAY_BUFFER,
                &js_sys::Float32Array::from(self.marker_data.as_slice()),
                GL::DYNAMIC_DRAW,
            );
            gl.draw_arrays(GL::POINTS, 0, (self.marker_data.len() / 8) as i32);
        }
        gl.bind_vertex_array(None);
        gl.depth_mask(true);
    }

    fn draw_body(&self, s: &Scene, i: usize, center: Vec3, sun_rel: Vec3, rel: &dyn Fn(DVec3) -> Vec3) {
        let gl = &self.gl;
        let pr = &self.planet;
        let b = &BODIES[i];
        let st = &s.sim.states[i];
        let look = &b.look;
        let shade = match look.shade {
            Shade::Sun => 0,
            Shade::Rock => 1,
            Shade::Gas => 2,
            Shade::Earth => 3,
            _ => 4,
        };
        let (has0, t0) = self.tex(look.tex, &self.white);
        let (_, t1) = self.tex(look.night, &self.black);
        let (_, t2) = self.tex(look.clouds, &self.black);
        for (unit, t) in [t0, t1, t2].iter().enumerate() {
            gl.active_texture(GL::TEXTURE0 + unit as u32);
            gl.bind_texture(GL::TEXTURE_2D, Some(t));
        }
        pr.i1(gl, "u_shade", shade);
        pr.i1(gl, "u_feature", look.feature);
        pr.i1(gl, "u_hasTex", has0 as i32);
        pr.rgb(gl, "u_color", look.color);
        pr.rgb(gl, "u_color2", look.color2);
        pr.f1(gl, "u_craters", look.craters);
        pr.f1(gl, "u_bumpy", look.bumpy);
        pr.f1(gl, "u_seed", i as f32 * 7.13);
        pr.f4(gl, "u_atmo", look.atmosphere);
        let time = if shade == 0 { (s.seconds % 10_000.0) as f32 } else { s.sim.days as f32 };
        pr.f1(gl, "u_time", time);
        pr.v3(gl, "u_center", center);
        pr.m3(gl, "u_basis", basis(st));

        // Shadow casters: a planet's largest moons; a moon's planet.
        let mut occ = [0f32; 16];
        let mut n = 0;
        let casters: Vec<usize> = match s.sim.parent[i] {
            Some(p) if matches!(b.motion, Motion::Moon { .. } | Motion::Lunar) => vec![p],
            _ => self.children[i].clone(),
        };
        for c in casters.into_iter().take(4) {
            let cs = &s.sim.states[c];
            let r = rel(cs.pos);
            occ[n * 4..n * 4 + 4].copy_from_slice(&[r.x, r.y, r.z, cs.radius as f32]);
            n += 1;
        }
        pr.f4v(gl, "u_occ", &occ);
        pr.i1(gl, "u_occN", n as i32);

        if let Some(rings) = look.rings {
            let r_eq = (st.radius * look.shape[0] as f64) as f32;
            let (has, t) = self.tex(rings.tex, &self.white);
            gl.active_texture(GL::TEXTURE3);
            gl.bind_texture(GL::TEXTURE_2D, Some(&t));
            pr.f4(gl, "u_ring", [rings.inner * r_eq, rings.outer * r_eq, rings.opacity, has as i32 as f32]);
            pr.v3(gl, "u_ringNormal", st.frame.y.as_vec3());
        } else {
            pr.f4(gl, "u_ring", [0.0; 4]);
        }
        let _ = sun_rel;

        let r = st.radius as f32;
        let shape = Vec3::from(look.shape) * r;
        if look.lobes.is_empty() {
            pr.v3(gl, "u_scale", shape);
            pr.v3(gl, "u_offset", Vec3::ZERO);
            gl.draw_elements_with_i32(GL::TRIANGLES, self.sphere.count, GL::UNSIGNED_SHORT, 0);
        } else {
            for lobe in look.lobes {
                pr.v3(gl, "u_scale", Vec3::from(lobe.scale) * r);
                pr.v3(gl, "u_offset", Vec3::from(lobe.offset) * r);
                gl.draw_elements_with_i32(GL::TRIANGLES, self.sphere.count, GL::UNSIGNED_SHORT, 0);
            }
        }
    }
}

fn basis(st: &crate::sim::State) -> Mat3 {
    Mat3::from_cols(st.frame.x.as_vec3(), st.frame.y.as_vec3(), st.frame.z.as_vec3())
}
