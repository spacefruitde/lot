use glam::camera::rh::{proj, view};
use glam::{Mat4, Vec3};

/// Kamera, die um einen Zielpunkt kreist. z zeigt nach oben.
#[derive(Debug, Clone, PartialEq)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    /// Drehung um die z-Achse, im Bogenmaß.
    pub yaw: f32,
    /// Neigung über der Horizontalen, im Bogenmaß.
    pub pitch: f32,
    /// Vertikaler Öffnungswinkel, im Bogenmaß.
    pub fov_y: f32,
    /// Größe der Ansicht in Punkten (nicht Pixeln), für Verschieben unter dem Finger.
    pub view_size: (f32, f32),
}

/// Drehung je Punkt Fingerbewegung, im Bogenmaß.
const ORBIT_PER_POINT: f32 = 0.006;
const MAX_PITCH: f32 = 89.0 * std::f32::consts::PI / 180.0;

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 10.0,
            yaw: (-60.0f32).to_radians(),
            pitch: 25.0f32.to_radians(),
            fov_y: 40.0f32.to_radians(),
            view_size: (800.0, 600.0),
        }
    }
}

impl OrbitCamera {
    pub fn eye(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        self.target + self.distance * Vec3::new(cp * cy, cp * sy, sp)
    }

    pub fn view(&self) -> Mat4 {
        view::look_at_mat4(self.eye(), self.target, Vec3::Z)
    }

    pub fn projection(&self) -> Mat4 {
        let aspect = self.view_size.0 / self.view_size.1.max(1.0);
        let near = (self.distance * 0.01).max(0.01);
        let far = self.distance * 100.0 + 100.0;
        // WebGPU: Tiefe 0 bis 1, y nach oben, wie die DirectX-Konvention.
        proj::directx::perspective(self.fov_y, aspect, near, far)
    }

    pub fn view_proj(&self) -> Mat4 {
        self.projection() * self.view()
    }

    /// Drehen. Finger nach rechts dreht das Modell nach rechts, nach oben kippt es nach hinten.
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx * ORBIT_PER_POINT;
        self.pitch = (self.pitch + dy * ORBIT_PER_POINT).clamp(-MAX_PITCH, MAX_PITCH);
    }

    /// Verschieben. Der Punkt unter dem Finger bleibt in der Zielebene unter dem Finger.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let per_point = 2.0 * self.distance * (self.fov_y / 2.0).tan() / self.view_size.1.max(1.0);
        let forward = (self.target - self.eye()).normalize();
        let right = forward.cross(Vec3::Z).normalize_or_zero();
        let up = right.cross(forward);
        self.target += (-dx * right + dy * up) * per_point;
    }

    /// Faktor größer 1 zoomt hinein.
    pub fn zoom(&mut self, factor: f32) {
        if factor.is_finite() && factor > 0.0 {
            self.distance = (self.distance / factor).clamp(0.2, 50_000.0);
        }
    }

    /// Stellt die Kamera so ein, dass eine Kugel um `center` mit `radius` ganz sichtbar ist.
    pub fn fit(&mut self, center: Vec3, radius: f32) {
        let aspect = self.view_size.0 / self.view_size.1.max(1.0);
        let half = (self.fov_y / 2.0).min((aspect * (self.fov_y / 2.0).tan()).atan());
        self.target = center;
        self.distance = (radius.max(0.1) / half.sin()) * 1.1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_is_clamped() {
        let mut c = OrbitCamera::default();
        c.orbit(0.0, 1.0e6);
        assert!((c.pitch - MAX_PITCH).abs() < 1e-6);
    }

    #[test]
    fn fit_shows_whole_sphere() {
        let mut c = OrbitCamera::default();
        c.fit(Vec3::new(2.5, 0.0, 1.3), 3.0);
        assert_eq!(c.target, Vec3::new(2.5, 0.0, 1.3));
        assert!(c.distance > 3.0);
    }

    #[test]
    fn pan_keeps_distance() {
        let mut c = OrbitCamera::default();
        let d = c.distance;
        c.pan(100.0, -50.0);
        assert!((c.eye() - c.target).length() - d < 1e-4);
    }
}
