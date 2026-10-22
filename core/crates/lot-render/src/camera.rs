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
        let (right, up) = self.screen_axes();
        self.target += (-dx * right + dy * up) * self.world_per_point();
    }

    /// Zoomen zum Zeiger. Ein Faktor größer 1 zoomt hinein. `x` und `y` geben
    /// die Lage des Zeigers in Punkten vom Mittelpunkt der Ansicht an, y nach
    /// oben. Der Punkt unter dem Zeiger bleibt in der Zielebene unter dem Zeiger.
    pub fn zoom(&mut self, factor: f32, x: f32, y: f32) {
        if !(factor.is_finite() && factor > 0.0) {
            return;
        }
        let distance = (self.distance / factor).clamp(0.2, 50_000.0);
        let applied = self.distance / distance;
        let (right, up) = self.screen_axes();
        let under_pointer = self.target + (x * right + y * up) * self.world_per_point();
        self.target = under_pointer + (self.target - under_pointer) / applied;
        self.distance = distance;
    }

    /// Rechts und oben auf dem Bildschirm, als Richtungen im Modell.
    fn screen_axes(&self) -> (Vec3, Vec3) {
        let forward = (self.target - self.eye()).normalize();
        let right = forward.cross(Vec3::Z).normalize_or_zero();
        (right, right.cross(forward))
    }

    /// Meter je Punkt in der Ebene durch das Ziel.
    fn world_per_point(&self) -> f32 {
        2.0 * self.distance * (self.fov_y / 2.0).tan() / self.view_size.1.max(1.0)
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
    fn zoom_keeps_point_under_pointer() {
        let mut c = OrbitCamera { view_size: (800.0, 600.0), ..Default::default() };
        // Punkt 200 Punkte rechts und 100 Punkte über der Mitte, in der Zielebene.
        let (right, up) = c.screen_axes();
        let point = c.target + (200.0 * right + 100.0 * up) * c.world_per_point();
        let before = c.view_proj().project_point3(point);
        c.zoom(2.0, 200.0, 100.0);
        let after = c.view_proj().project_point3(point);
        assert!((before - after).length() < 1e-4, "{before} -> {after}");
        assert!((c.distance - 5.0).abs() < 1e-4);
    }

    #[test]
    fn zoom_at_center_keeps_target() {
        let mut c = OrbitCamera::default();
        let target = c.target;
        c.zoom(1.5, 0.0, 0.0);
        assert!((c.target - target).length() < 1e-6);
    }

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
