use serde::{Deserialize, Serialize};

pub const AUTHOR: &str = "Amir Mobasheraghdam";
pub const PRODUCT_NAME: &str = "OrangeCNC Studio";

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn distance(self, other: Self) -> f64 {
        (self - other).norm()
    }

    pub fn norm(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn lerp(self, other: Self, t: f64) -> Self {
        self + (other - self) * t
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl std::ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Plane {
    #[default]
    Xy,
    Xz,
    Yz,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    Inch,
    #[default]
    Millimeter,
}

impl Units {
    pub fn to_mm(self, value: f64) -> f64 {
        match self {
            Self::Millimeter => value,
            Self::Inch => value * 25.4,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMode {
    #[default]
    Absolute,
    Incremental,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionMode {
    #[default]
    Rapid,
    Linear,
    ArcClockwise,
    ArcCounterClockwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ArcDefinition {
    CenterOffset { i: f64, j: f64, k: f64 },
    Radius(f64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanonicalCommand {
    Move {
        mode: MotionMode,
        plane: Plane,
        start: Vec3,
        end: Vec3,
        arc: Option<ArcDefinition>,
        feed_mm_min: Option<f64>,
    },
    Spindle {
        enabled: bool,
        clockwise: bool,
        rpm: Option<f64>,
    },
    SelectTool(u32),
    ToolChange,
    ProgramEnd,
    Comment(String),
    NoOp,
}

#[derive(Debug, thiserror::Error)]
pub enum CncError {
    #[error("Parserfehler: {0}")]
    Parser(String),
    #[error("Geometriefehler: {0}")]
    Geometry(String),
    #[error("Maschinenfehler: {0}")]
    Machine(String),
}
