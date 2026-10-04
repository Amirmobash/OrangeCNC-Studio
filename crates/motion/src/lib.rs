use std::f64::consts::{PI, TAU};

use orangecnc_domain::{ArcDefinition, CanonicalCommand, MotionMode, Plane, Vec3};

const POSITION_EPSILON_MM: f64 = 1e-12;
const ARC_RADIUS_MATCH_TOLERANCE_MM: f64 = 0.05;

#[derive(Debug, Clone)]
pub struct PlannerSettings {
    pub chord_tolerance_mm: f64,
    pub max_segment_mm: f64,
}

impl Default for PlannerSettings {
    fn default() -> Self {
        Self {
            chord_tolerance_mm: 0.02,
            max_segment_mm: 1.0,
        }
    }
}

impl PlannerSettings {
    fn validate(&self) -> Result<(), String> {
        if !self.chord_tolerance_mm.is_finite() || self.chord_tolerance_mm <= 0.0 {
            return Err("Bogentoleranz muss endlich und größer als 0 sein".to_owned());
        }
        if !self.max_segment_mm.is_finite() || self.max_segment_mm <= 0.0 {
            return Err("Maximale Segmentlänge muss endlich und größer als 0 sein".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    Rapid,
    Feed,
    Arc,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub start: Vec3,
    pub end: Vec3,
    pub kind: SegmentKind,
    pub feed_mm_min: Option<f64>,
}

impl Segment {
    pub fn length(&self) -> f64 {
        self.start.distance(self.end)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Toolpath {
    pub segments: Vec<Segment>,
}

impl Toolpath {
    pub fn total_length_mm(&self) -> f64 {
        self.segments.iter().map(Segment::length).sum()
    }

    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        let first = self.segments.first()?;
        let mut min = first.start;
        let mut max = first.start;

        for point in self.segments.iter().flat_map(|segment| [segment.start, segment.end]) {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            min.z = min.z.min(point.z);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
            max.z = max.z.max(point.z);
        }

        Some((min, max))
    }
}

pub struct MotionPlanner {
    settings: PlannerSettings,
}

impl MotionPlanner {
    pub fn new(settings: PlannerSettings) -> Self {
        Self { settings }
    }

    pub fn build(&self, commands: &[CanonicalCommand]) -> Result<Toolpath, String> {
        self.settings.validate()?;

        let mut path = Toolpath::default();
        for command in commands {
            let CanonicalCommand::Move {
                mode,
                plane,
                start,
                end,
                arc,
                feed_mm_min,
            } = command
            else {
                continue;
            };

            match mode {
                MotionMode::Rapid => {
                    self.linear(*start, *end, SegmentKind::Rapid, *feed_mm_min, &mut path)?
                }
                MotionMode::Linear => {
                    self.linear(*start, *end, SegmentKind::Feed, *feed_mm_min, &mut path)?
                }
                MotionMode::ArcClockwise | MotionMode::ArcCounterClockwise => {
                    let definition = arc
                        .as_ref()
                        .copied()
                        .ok_or_else(|| "Bogenbewegung ohne Definition".to_owned())?;
                    self.arc(
                        *start,
                        *end,
                        *plane,
                        definition,
                        *mode == MotionMode::ArcClockwise,
                        *feed_mm_min,
                        &mut path,
                    )?;
                }
            }
        }

        Ok(path)
    }

    fn linear(
        &self,
        start: Vec3,
        end: Vec3,
        kind: SegmentKind,
        feed: Option<f64>,
        out: &mut Toolpath,
    ) -> Result<(), String> {
        if !start.is_finite() || !end.is_finite() {
            return Err("Linearbewegung enthält nicht-endliche Koordinaten".to_owned());
        }

        let distance = start.distance(end);
        if distance <= POSITION_EPSILON_MM {
            return Ok(());
        }

        let segment_count = (distance / self.settings.max_segment_mm).ceil().max(1.0) as usize;
        let mut previous = start;

        for index in 1..=segment_count {
            let next = if index == segment_count {
                end
            } else {
                start.lerp(end, index as f64 / segment_count as f64)
            };
            out.segments.push(Segment {
                start: previous,
                end: next,
                kind,
                feed_mm_min: feed,
            });
            previous = next;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn arc(
        &self,
        start: Vec3,
        end: Vec3,
        plane: Plane,
        definition: ArcDefinition,
        clockwise: bool,
        feed: Option<f64>,
        out: &mut Toolpath,
    ) -> Result<(), String> {
        if !start.is_finite() || !end.is_finite() {
            return Err("Bogenbewegung enthält nicht-endliche Koordinaten".to_owned());
        }

        let frame = PlaneFrame::from_plane(plane);
        let projected_start = frame.project(start);
        let projected_end = frame.project(end);
        let center = match definition {
            ArcDefinition::CenterOffset { i, j, k } => {
                frame.center_from_offsets(projected_start, i, j, k)
            }
            ArcDefinition::Radius(radius) => {
                center_from_radius(projected_start, projected_end, radius, clockwise)?
            }
        };

        let radius_start = distance_2d(projected_start.u, projected_start.v, center.0, center.1);
        let radius_end = distance_2d(projected_end.u, projected_end.v, center.0, center.1);

        if !radius_start.is_finite() || !radius_end.is_finite() {
            return Err("Bogenradius ist nicht endlich".to_owned());
        }
        if radius_start < 1e-9
            || (radius_start - radius_end).abs() > ARC_RADIUS_MATCH_TOLERANCE_MM
        {
            return Err(format!(
                "Inkonsistenter Bogenradius: {radius_start:.4} / {radius_end:.4} mm"
            ));
        }

        let start_angle = (projected_start.v - center.1).atan2(projected_start.u - center.0);
        let end_angle = (projected_end.v - center.1).atan2(projected_end.u - center.0);
        let sweep = directed_sweep(start_angle, end_angle, clockwise);
        let segment_count = segment_count(radius_start, sweep, &self.settings);
        let mut previous = start;

        for index in 1..=segment_count {
            let t = index as f64 / segment_count as f64;
            let next = if index == segment_count {
                // Preserve the canonical endpoint exactly; trigonometric reconstruction can differ by
                // a few ulps and those errors otherwise accumulate across consecutive moves.
                end
            } else {
                let angle = start_angle + sweep * t;
                frame.unproject(Projected {
                    u: center.0 + radius_start * angle.cos(),
                    v: center.1 + radius_start * angle.sin(),
                    w: projected_start.w + (projected_end.w - projected_start.w) * t,
                })
            };

            out.segments.push(Segment {
                start: previous,
                end: next,
                kind: SegmentKind::Arc,
                feed_mm_min: feed,
            });
            previous = next;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Projected {
    u: f64,
    v: f64,
    w: f64,
}

#[derive(Clone, Copy)]
struct PlaneFrame {
    plane: Plane,
}

impl PlaneFrame {
    fn from_plane(plane: Plane) -> Self {
        Self { plane }
    }

    fn project(self, point: Vec3) -> Projected {
        match self.plane {
            Plane::Xy => Projected {
                u: point.x,
                v: point.y,
                w: point.z,
            },
            Plane::Xz => Projected {
                u: point.x,
                v: point.z,
                w: point.y,
            },
            Plane::Yz => Projected {
                u: point.y,
                v: point.z,
                w: point.x,
            },
        }
    }

    fn unproject(self, point: Projected) -> Vec3 {
        match self.plane {
            Plane::Xy => Vec3::new(point.u, point.v, point.w),
            Plane::Xz => Vec3::new(point.u, point.w, point.v),
            Plane::Yz => Vec3::new(point.w, point.u, point.v),
        }
    }

    fn center_from_offsets(
        self,
        start: Projected,
        i: f64,
        j: f64,
        k: f64,
    ) -> (f64, f64) {
        match self.plane {
            Plane::Xy => (start.u + i, start.v + j),
            Plane::Xz => (start.u + i, start.v + k),
            Plane::Yz => (start.u + j, start.v + k),
        }
    }
}

fn distance_2d(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
}

fn directed_sweep(start: f64, end: f64, clockwise: bool) -> f64 {
    let mut sweep = end - start;
    if clockwise {
        if sweep >= 0.0 {
            sweep -= TAU;
        }
    } else if sweep <= 0.0 {
        sweep += TAU;
    }
    sweep
}

fn segment_count(radius: f64, sweep: f64, settings: &PlannerSettings) -> usize {
    let tolerance = settings.chord_tolerance_mm.min(radius).max(1e-5);
    let angle_by_tolerance = if tolerance < radius {
        2.0 * (1.0 - tolerance / radius).acos()
    } else {
        PI
    };

    let by_tolerance = (sweep.abs() / angle_by_tolerance.max(1e-6)).ceil();
    let by_length = (radius * sweep.abs() / settings.max_segment_mm).ceil();
    by_tolerance.max(by_length).max(2.0) as usize
}

fn center_from_radius(
    start: Projected,
    end: Projected,
    signed_radius: f64,
    clockwise: bool,
) -> Result<(f64, f64), String> {
    if !signed_radius.is_finite() || signed_radius.abs() < 1e-12 {
        return Err("R-Bogen benötigt einen endlichen Radius ungleich 0".to_owned());
    }

    let dx = end.u - start.u;
    let dy = end.v - start.v;
    let chord = (dx * dx + dy * dy).sqrt();
    let radius = signed_radius.abs();

    if chord < 1e-12 {
        return Err("R-Bogen mit identischem Start/Ende ist mehrdeutig".to_owned());
    }
    if chord > 2.0 * radius + 1e-9 {
        return Err("R-Bogen: Radius ist kleiner als halbe Sehne".to_owned());
    }

    let midpoint = ((start.u + end.u) / 2.0, (start.v + end.v) / 2.0);
    let center_offset = (radius * radius - (chord * chord) / 4.0).max(0.0).sqrt();
    let normal_x = -dy / chord;
    let normal_y = dx / chord;
    let first = (
        midpoint.0 + normal_x * center_offset,
        midpoint.1 + normal_y * center_offset,
    );
    let second = (
        midpoint.0 - normal_x * center_offset,
        midpoint.1 - normal_y * center_offset,
    );

    let sweep_for = |center: (f64, f64)| {
        let start_angle = (start.v - center.1).atan2(start.u - center.0);
        let end_angle = (end.v - center.1).atan2(end.u - center.0);
        directed_sweep(start_angle, end_angle, clockwise).abs()
    };

    let first_sweep = sweep_for(first);
    let second_sweep = sweep_for(second);
    let want_major_arc = signed_radius < 0.0;

    Ok(if want_major_arc {
        if first_sweep >= second_sweep {
            first
        } else {
            second
        }
    } else if first_sweep <= second_sweep {
        first
    } else {
        second
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radius_arc_creates_segments_and_ends_exactly_at_target() {
        let planner = MotionPlanner::new(PlannerSettings::default());
        let target = Vec3::new(10.0, 0.0, 0.0);
        let commands = [CanonicalCommand::Move {
            mode: MotionMode::ArcCounterClockwise,
            plane: Plane::Xy,
            start: Vec3::ZERO,
            end: target,
            arc: Some(ArcDefinition::Radius(5.0)),
            feed_mm_min: Some(500.0),
        }];

        let path = planner.build(&commands).unwrap();
        assert!(path.segments.len() > 5);
        assert_eq!(path.segments.last().unwrap().end, target);
    }

    #[test]
    fn full_circle_ijk_arc_closes_at_start() {
        let planner = MotionPlanner::new(PlannerSettings::default());
        let start = Vec3::new(10.0, 0.0, 0.0);
        let commands = [CanonicalCommand::Move {
            mode: MotionMode::ArcCounterClockwise,
            plane: Plane::Xy,
            start,
            end: start,
            arc: Some(ArcDefinition::CenterOffset {
                i: -10.0,
                j: 0.0,
                k: 0.0,
            }),
            feed_mm_min: Some(500.0),
        }];

        let path = planner.build(&commands).unwrap();
        assert!(path.segments.len() > 20);
        assert_eq!(path.segments.last().unwrap().end, start);
        assert!((path.total_length_mm() - 2.0 * PI * 10.0).abs() < 0.1);
    }

    #[test]
    fn zero_length_linear_move_does_not_create_fake_segment() {
        let planner = MotionPlanner::new(PlannerSettings::default());
        let commands = [CanonicalCommand::Move {
            mode: MotionMode::Linear,
            plane: Plane::Xy,
            start: Vec3::ZERO,
            end: Vec3::ZERO,
            arc: None,
            feed_mm_min: Some(500.0),
        }];

        assert!(planner.build(&commands).unwrap().segments.is_empty());
    }
}
