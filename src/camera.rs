//! The point of view: where you are and which way you face.

use glam::{DMat3, DQuat, DVec2, DVec3, Mat4, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub pos: DVec3,
    pub rot: DQuat,
    /// Vertical field of view, radians.
    pub fov: f64,
    /// Lens shift in normalised device coordinates: moves the picture on
    /// screen (to keep an object clear of a panel) without turning the view.
    pub shift: DVec2,
}

impl Camera {
    pub fn forward(&self) -> DVec3 {
        self.rot * DVec3::NEG_Z
    }

    pub fn up(&self) -> DVec3 {
        self.rot * DVec3::Y
    }

    pub fn right(&self) -> DVec3 {
        self.rot * DVec3::X
    }

    /// View matrix without translation: everything is drawn relative to the
    /// camera, so large coordinates never reach the GPU.
    pub fn view_rotation(&self) -> Mat4 {
        Mat4::from_quat(self.rot.inverse().as_quat())
    }

    /// Screen position (CSS pixels) and depth for a point relative to the
    /// camera, or None if it is behind.
    pub fn project(&self, rel: DVec3, width: f64, height: f64) -> Option<(f64, f64, f64)> {
        let v = self.rot.inverse() * rel;
        if v.z > -1e-9 {
            return None;
        }
        let depth = -v.z;
        let f = 1.0 / (self.fov / 2.0).tan();
        let x = v.x * f / (width / height) / depth + self.shift.x;
        let y = v.y * f / depth + self.shift.y;
        Some(((x + 1.0) * 0.5 * width, (1.0 - y) * 0.5 * height, depth))
    }

    /// The projection matrix, lens shift included.
    pub fn projection(&self, aspect: f64, near: f32, far: f32) -> Mat4 {
        Mat4::from_translation(Vec3::new(self.shift.x as f32, self.shift.y as f32, 0.0))
            * Mat4::perspective_rh_gl(self.fov as f32, aspect as f32, near, far)
    }

    /// Pixels per display unit at a given depth.
    pub fn pixels_per_unit(&self, depth: f64, height: f64) -> f64 {
        height / (2.0 * (self.fov / 2.0).tan() * depth)
    }
}

/// A rotation that looks along `forward` with `up` roughly up.
pub fn look_rotation(forward: DVec3, up: DVec3) -> DQuat {
    let f = forward.normalize();
    let mut r = f.cross(up);
    if r.length_squared() < 1e-10 {
        r = f.cross(DVec3::X);
        if r.length_squared() < 1e-10 {
            r = f.cross(DVec3::Z);
        }
    }
    let r = r.normalize();
    let u = r.cross(f);
    DQuat::from_mat3(&DMat3::from_cols(r, u, -f)).normalize()
}

pub fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_rotation_faces_forward() {
        let q = look_rotation(DVec3::new(1.0, 0.2, 0.3), DVec3::Y);
        let cam = Camera { pos: DVec3::ZERO, rot: q, fov: 1.0, shift: DVec2::ZERO };
        assert!((cam.forward() - DVec3::new(1.0, 0.2, 0.3).normalize()).length() < 1e-9);
        let (x, y, _) = cam.project(DVec3::new(1.0, 0.2, 0.3) * 5.0, 800.0, 600.0).unwrap();
        assert!((x - 400.0).abs() < 1e-6 && (y - 300.0).abs() < 1e-6);
        assert!(cam.up().y > 0.0);
    }
}
