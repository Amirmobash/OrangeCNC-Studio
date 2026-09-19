use orangecnc_domain::{DistanceMode, MotionMode, Plane, Units, Vec3};

#[derive(Debug, Clone)]
pub struct ModalState {
    pub motion: MotionMode,
    pub plane: Plane,
    pub units: Units,
    pub distance: DistanceMode,
    pub position_mm: Vec3,
    pub feed_mm_min: Option<f64>,
    pub spindle_rpm: Option<f64>,
    pub selected_tool: Option<u32>,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            motion: MotionMode::Rapid,
            plane: Plane::Xy,
            units: Units::Millimeter,
            distance: DistanceMode::Absolute,
            position_mm: Vec3::ZERO,
            feed_mm_min: None,
            spindle_rpm: None,
            selected_tool: None,
        }
    }
}
