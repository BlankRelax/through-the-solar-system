//! Thin helpers over WebGL2: programs with cached uniform locations, buffers,
//! vertex arrays and textures.

use glam::{Mat3, Mat4, Vec3};
use std::cell::RefCell;
use std::collections::HashMap;
use web_sys::{
    HtmlImageElement, WebGl2RenderingContext as GL, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture,
    WebGlUniformLocation, WebGlVertexArrayObject,
};

const HEADER: &str = "#version 300 es\nprecision highp float;\nprecision highp int;\n";
pub const NOISE: &str = include_str!("shaders/noise.glsl");

pub struct Program {
    prog: WebGlProgram,
    locs: RefCell<HashMap<&'static str, Option<WebGlUniformLocation>>>,
}

fn compile(gl: &GL, kind: u32, parts: &[&str], name: &str) -> Result<WebGlShader, String> {
    let shader = gl.create_shader(kind).ok_or("cannot create shader")?;
    let mut src = String::from(HEADER);
    for p in parts {
        src.push_str(p);
        src.push('\n');
    }
    gl.shader_source(&shader, &src);
    gl.compile_shader(&shader);
    if gl.get_shader_parameter(&shader, GL::COMPILE_STATUS).as_bool().unwrap_or(false) {
        Ok(shader)
    } else {
        Err(format!("{name}: {}", gl.get_shader_info_log(&shader).unwrap_or_default()))
    }
}

impl Program {
    pub fn new(gl: &GL, name: &str, vs: &[&str], fs: &[&str]) -> Result<Program, String> {
        let v = compile(gl, GL::VERTEX_SHADER, vs, name)?;
        let f = compile(gl, GL::FRAGMENT_SHADER, fs, name)?;
        let prog = gl.create_program().ok_or("cannot create program")?;
        gl.attach_shader(&prog, &v);
        gl.attach_shader(&prog, &f);
        gl.link_program(&prog);
        if !gl.get_program_parameter(&prog, GL::LINK_STATUS).as_bool().unwrap_or(false) {
            return Err(format!("{name}: {}", gl.get_program_info_log(&prog).unwrap_or_default()));
        }
        gl.delete_shader(Some(&v));
        gl.delete_shader(Some(&f));
        Ok(Program { prog, locs: RefCell::new(HashMap::new()) })
    }

    pub fn bind(&self, gl: &GL) {
        gl.use_program(Some(&self.prog));
    }

    fn loc(&self, gl: &GL, name: &'static str) -> Option<WebGlUniformLocation> {
        self.locs
            .borrow_mut()
            .entry(name)
            .or_insert_with(|| gl.get_uniform_location(&self.prog, name))
            .clone()
    }

    pub fn f1(&self, gl: &GL, name: &'static str, v: f32) {
        gl.uniform1f(self.loc(gl, name).as_ref(), v);
    }

    pub fn f2(&self, gl: &GL, name: &'static str, x: f32, y: f32) {
        gl.uniform2f(self.loc(gl, name).as_ref(), x, y);
    }

    pub fn v3(&self, gl: &GL, name: &'static str, v: Vec3) {
        gl.uniform3f(self.loc(gl, name).as_ref(), v.x, v.y, v.z);
    }

    pub fn rgb(&self, gl: &GL, name: &'static str, c: [f32; 3]) {
        gl.uniform3f(self.loc(gl, name).as_ref(), c[0], c[1], c[2]);
    }

    pub fn f4(&self, gl: &GL, name: &'static str, v: [f32; 4]) {
        gl.uniform4f(self.loc(gl, name).as_ref(), v[0], v[1], v[2], v[3]);
    }

    pub fn f4v(&self, gl: &GL, name: &'static str, v: &[f32]) {
        gl.uniform4fv_with_f32_array(self.loc(gl, name).as_ref(), v);
    }

    pub fn i1(&self, gl: &GL, name: &'static str, v: i32) {
        gl.uniform1i(self.loc(gl, name).as_ref(), v);
    }

    pub fn m3(&self, gl: &GL, name: &'static str, m: Mat3) {
        gl.uniform_matrix3fv_with_f32_array(self.loc(gl, name).as_ref(), false, &m.to_cols_array());
    }

    pub fn m4(&self, gl: &GL, name: &'static str, m: Mat4) {
        gl.uniform_matrix4fv_with_f32_array(self.loc(gl, name).as_ref(), false, &m.to_cols_array());
    }
}

pub fn array_buffer(gl: &GL, data: &[f32], usage: u32) -> WebGlBuffer {
    let buf = gl.create_buffer().expect("buffer");
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buf));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &js_sys::Float32Array::from(data), usage);
    buf
}

/// A vertex array over one interleaved float buffer. `layout` lists
/// (location, component count) in order.
pub fn vertex_array(gl: &GL, data: &[f32], layout: &[(u32, i32)], indices: Option<&[u16]>) -> WebGlVertexArrayObject {
    let vao = gl.create_vertex_array().expect("vao");
    gl.bind_vertex_array(Some(&vao));
    array_buffer(gl, data, GL::STATIC_DRAW);
    let stride: i32 = layout.iter().map(|&(_, n)| n).sum::<i32>() * 4;
    let mut offset = 0;
    for &(loc, n) in layout {
        gl.enable_vertex_attrib_array(loc);
        gl.vertex_attrib_pointer_with_i32(loc, n, GL::FLOAT, false, stride, offset);
        offset += n * 4;
    }
    if let Some(idx) = indices {
        let ib = gl.create_buffer().expect("index buffer");
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ib));
        gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &js_sys::Uint16Array::from(idx), GL::STATIC_DRAW);
    }
    gl.bind_vertex_array(None);
    vao
}

pub fn solid_texture(gl: &GL, rgba: [u8; 4]) -> WebGlTexture {
    let tex = gl.create_texture().expect("texture");
    gl.bind_texture(GL::TEXTURE_2D, Some(&tex));
    let _ = gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        GL::TEXTURE_2D,
        0,
        GL::RGBA as i32,
        1,
        1,
        0,
        GL::RGBA,
        GL::UNSIGNED_BYTE,
        Some(&rgba),
    );
    tex
}

pub fn image_texture(gl: &GL, img: &HtmlImageElement, wrap_s: u32, anisotropy: f32) -> WebGlTexture {
    let tex = gl.create_texture().expect("texture");
    gl.bind_texture(GL::TEXTURE_2D, Some(&tex));
    let _ = gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
        GL::TEXTURE_2D,
        0,
        GL::RGBA as i32,
        GL::RGBA,
        GL::UNSIGNED_BYTE,
        img,
    );
    gl.generate_mipmap(GL::TEXTURE_2D);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR_MIPMAP_LINEAR as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, wrap_s as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
    if anisotropy > 1.0 {
        // EXT_texture_filter_anisotropic: TEXTURE_MAX_ANISOTROPY_EXT
        gl.tex_parameterf(GL::TEXTURE_2D, 0x84FE, anisotropy);
    }
    tex
}
