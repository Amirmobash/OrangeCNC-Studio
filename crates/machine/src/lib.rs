use orangecnc_domain::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisLimits {
    pub min_mm: f64,
    pub max_mm: f64,
    pub max_feed_mm_min: f64,
    pub max_accel_mm_s2: f64,
}

impl AxisLimits {
    pub fn contains(&self, value: f64) -> bool {
        value.is_finite() && value >= self.min_mm && value <= self.max_mm
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineProfile {
    pub name: String,
    pub x: AxisLimits,
    pub y: AxisLimits,
    pub z: AxisLimits,
}

impl Default for MachineProfile {
    fn default() -> Self {
        Self {
            name: "OrangeCNC 3-Achs Standard".into(),
            x: AxisLimits {
                min_mm: 0.0,
                max_mm: 500.0,
                max_feed_mm_min: 8_000.0,
                max_accel_mm_s2: 500.0,
            },
            y: AxisLimits {
                min_mm: 0.0,
                max_mm: 500.0,
                max_feed_mm_min: 8_000.0,
                max_accel_mm_s2: 500.0,
            },
            z: AxisLimits {
                min_mm: -150.0,
                max_mm: 20.0,
                max_feed_mm_min: 3_000.0,
                max_accel_mm_s2: 300.0,
            },
        }
    }
}

impl MachineProfile {
    pub fn validate_position(&self, position: Vec3) -> Vec<String> {
        let mut warnings = Vec::new();
        check_axis("X", position.x, &self.x, &mut warnings);
        check_axis("Y", position.y, &self.y, &mut warnings);
        check_axis("Z", position.z, &self.z, &mut warnings);
        warnings
    }
}

fn check_axis(name: &str, value: f64, limits: &AxisLimits, warnings: &mut Vec<String>) {
    if !value.is_finite() {
        warnings.push(format!("{name}-Position ist nicht endlich"));
        return;
    }

    if !limits.contains(value) {
        warnings.push(format!(
            "{name}-Grenze verletzt: {value:.3} mm ({} .. {} mm)",
            limits.min_mm, limits.max_mm
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_reports_out_of_bounds_position() {
        let warnings = MachineProfile::default().validate_position(Vec3::new(600.0, 0.0, 0.0));
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].starts_with("X-Grenze verletzt"));
    }
}
