use std::f64::consts::{PI, TAU};

use orangecnc_domain::{ArcDefinition, CanonicalCommand, MotionMode, Plane, Vec3};

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
        for p in self.segments.iter().flat_map(|s| [s.start, s.end]) {
            min.x = min.x.min(p.x); min.y = min.y.min(p.y); min.z = min.z.min(p.z);
            max.x = max.x.max(p.x); max.y = max.y.max(p.y); max.z = max.z.max(p.z);
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
        let mut path = Toolpath::default();
        for command in commands {
            let CanonicalCommand::Move { mode, plane, start, end, arc, feed_mm_min } = command else {
                continue;
            };
            match mode {
                MotionMode::Rapid => self.linear(*start, *end, SegmentKind::Rapid, *feed_mm_min, &mut path),
                MotionMode::Linear => self.linear(*start, *end, SegmentKind::Feed, *feed_mm_min, &mut path),
                MotionMode::ArcClockwise | MotionMode::ArcCounterClockwise => {
                    let definition = arc.ok_or_else(|| "Bogenbewegung ohne Definition".to_owned())?;
                    self.arc(*start, *end, *plane, definition, *mode == MotionMode::ArcClockwise, *feed_mm_min, &mut path)?;
                }
            }
        }
        Ok(path)
    }

    fn linear(&self, start: Vec3, end: Vec3, kind: SegmentKind, feed: Option<f64>, out: &mut Toolpath) {
        let distance = start.distance(end);
        let count = (distance / self.settings.max_segment_mm.max(0.001)).ceil().max(1.0) as usize;
        let mut previous = start;
        for i in 1..=count {
            let next = start.lerp(end, i as f64 / count as f64);
            out.segments.push(Segment { start: previous, end: next, kind, feed_mm_min: feed });
            previous = next;
        }
    }

    fn arc(&self, start: Vec3, end: Vec3, plane: Plane, definition: ArcDefinition, clockwise: bool,
           feed: Option<f64>, out: &mut Toolpath) -> Result<(), String> {
        let frame = PlaneFrame::from_plane(plane);
        let s = frame.project(start);
        let e = frame.project(end);
        let center = match definition {
            ArcDefinition::CenterOffset { i, j, k } => frame.center_from_offsets(s, i, j, k),
            ArcDefinition::Radius(radius) => center_from_radius(s, e, radius, clockwise)?,
        };

        let radius_start = ((s.u-center.0).powi(2)+(s.v-center.1).powi(2)).sqrt();
        let radius_end = ((e.u-center.0).powi(2)+(e.v-center.1).powi(2)).sqrt();
        if radius_start < 1e-9 || (radius_start-radius_end).abs() > 0.05 {
            return Err(format!("Inkonsistenter Bogenradius: {:.4} / {:.4} mm", radius_start, radius_end));
        }

        let a0 = (s.v-center.1).atan2(s.u-center.0);
        let a1 = (e.v-center.1).atan2(e.u-center.0);
        let sweep = directed_sweep(a0, a1, clockwise);
        let count = segment_count(radius_start, sweep, &self.settings);
        let mut previous = start;
        for i in 1..=count {
            let t = i as f64 / count as f64;
            let angle = a0 + sweep*t;
            let projected = Projected {
                u: center.0 + radius_start*angle.cos(),
                v: center.1 + radius_start*angle.sin(),
                w: s.w + (e.w-s.w)*t,
            };
            let next = frame.unproject(projected);
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
struct Projected { u: f64, v: f64, w: f64 }

#[derive(Clone, Copy)]
struct PlaneFrame { plane: Plane }

impl PlaneFrame {
    fn from_plane(plane: Plane) -> Self { Self { plane } }
    fn project(self, p: Vec3) -> Projected {
        match self.plane {
            Plane::Xy => Projected { u:p.x, v:p.y, w:p.z },
            Plane::Xz => Projected { u:p.x, v:p.z, w:p.y },
            Plane::Yz => Projected { u:p.y, v:p.z, w:p.x },
        }
    }
    fn unproject(self, p: Projected) -> Vec3 {
        match self.plane {
            Plane::Xy => Vec3::new(p.u,p.v,p.w),
            Plane::Xz => Vec3::new(p.u,p.w,p.v),
            Plane::Yz => Vec3::new(p.w,p.u,p.v),
        }
    }
    fn center_from_offsets(self, s: Projected, i:f64, j:f64, k:f64) -> (f64,f64) {
        match self.plane {
            Plane::Xy => (s.u+i, s.v+j),
            Plane::Xz => (s.u+i, s.v+k),
            Plane::Yz => (s.u+j, s.v+k),
        }
    }
}

fn directed_sweep(start: f64, end: f64, clockwise: bool) -> f64 {
    let mut sweep = end-start;
    if clockwise {
        if sweep >= 0.0 { sweep -= TAU; }
    } else if sweep <= 0.0 {
        sweep += TAU;
    }
    sweep
}

fn segment_count(radius: f64, sweep: f64, settings: &PlannerSettings) -> usize {
    let tol = settings.chord_tolerance_mm.clamp(1e-5, radius.max(1e-5));
    let angle_by_tol = if tol < radius {
        2.0*(1.0-tol/radius).acos()
    } else { PI };
    let by_tolerance = (sweep.abs()/angle_by_tol.max(1e-6)).ceil();
    let by_length = (radius*sweep.abs()/settings.max_segment_mm.max(0.001)).ceil();
    by_tolerance.max(by_length).max(2.0) as usize
}

fn center_from_radius(s: Projected, e: Projected, signed_radius: f64, clockwise: bool) -> Result<(f64,f64), String> {
    let dx=e.u-s.u; let dy=e.v-s.v;
    let chord=(dx*dx+dy*dy).sqrt();
    let r=signed_radius.abs();
    if chord < 1e-12 { return Err("R-Bogen mit identischem Start/Ende ist mehrdeutig".into()); }
    if chord > 2.0*r + 1e-9 { return Err("R-Bogen: Radius ist kleiner als halbe Sehne".into()); }
    let mid=((s.u+e.u)/2.0,(s.v+e.v)/2.0);
    let h=(r*r-(chord*chord)/4.0).max(0.0).sqrt();
    let nx=-dy/chord; let ny=dx/chord;
    let c1=(mid.0+nx*h,mid.1+ny*h);
    let c2=(mid.0-nx*h,mid.1-ny*h);
    let want_major=signed_radius<0.0;
    let score=|c:(f64,f64)| {
        let a0=(s.v-c.1).atan2(s.u-c.0); let a1=(e.v-c.1).atan2(e.u-c.0);
        directed_sweep(a0,a1,clockwise).abs()
    };
    let s1=score(c1); let s2=score(c2);
    let pick = if want_major {
        if s1>=s2 { c1 } else { c2 }
    } else if s1<=s2 { c1 } else { c2 };
    Ok(pick)
}

#[cfg(test)]
mod tests {
    use super::*;
    use orangecnc_domain::{ArcDefinition, CanonicalCommand, Plane};

    #[test]
    fn r_arc_creates_segments() {
        let planner=MotionPlanner::new(PlannerSettings::default());
        let commands=[CanonicalCommand::Move {
            mode: MotionMode::ArcCounterClockwise,
            plane: Plane::Xy,
            start: Vec3::new(0.0,0.0,0.0),
            end: Vec3::new(10.0,0.0,0.0),
            arc: Some(ArcDefinition::Radius(5.0)),
            feed_mm_min: Some(500.0),
        }];
        let path=planner.build(&commands).unwrap();
        assert!(path.segments.len()>5);
    }
}
