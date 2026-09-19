use orangecnc_domain::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisLimits {
    pub min_mm: f64,
    pub max_mm: f64,
    pub max_feed_mm_min: f64,
    pub max_accel_mm_s2: f64,
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
            x: AxisLimits { min_mm: 0.0, max_mm: 500.0, max_feed_mm_min: 8_000.0, max_accel_mm_s2: 500.0 },
            y: AxisLimits { min_mm: 0.0, max_mm: 500.0, max_feed_mm_min: 8_000.0, max_accel_mm_s2: 500.0 },
            z: AxisLimits { min_mm: -150.0, max_mm: 20.0, max_feed_mm_min: 3_000.0, max_accel_mm_s2: 300.0 },
        }
    }
}

impl MachineProfile {
    pub fn validate_position(&self, p: Vec3) -> Vec<String> {
        let mut warnings=Vec::new();
        check("X",p.x,&self.x,&mut warnings);
        check("Y",p.y,&self.y,&mut warnings);
        check("Z",p.z,&self.z,&mut warnings);
        warnings
    }
}

fn check(name:&str,value:f64,limit:&AxisLimits,warnings:&mut Vec<String>) {
    if value < limit.min_mm || value > limit.max_mm {
        warnings.push(format!("{name}-Grenze verletzt: {value:.3} mm ({} .. {} mm)",limit.min_mm,limit.max_mm));
    }
}
