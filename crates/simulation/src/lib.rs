use orangecnc_domain::Vec3;
use orangecnc_machine::{AxisLimits, MachineProfile};
use orangecnc_motion::{Segment, SegmentKind, Toolpath};

#[derive(Debug, Clone, Default)]
pub struct SimulationReport {
    pub final_position: Vec3,
    pub total_length_mm: f64,
    pub rapid_length_mm: f64,
    pub feed_length_mm: f64,
    pub estimated_motion_seconds: f64,
    pub segment_count: usize,
    pub warnings: Vec<String>,
}

pub struct Simulator {
    profile: MachineProfile,
}

impl Simulator {
    pub fn new(profile: MachineProfile) -> Self {
        Self { profile }
    }

    pub fn run(&self, path: &Toolpath) -> SimulationReport {
        let mut report = SimulationReport {
            segment_count: path.segments.len(),
            ..SimulationReport::default()
        };

        for segment in &path.segments {
            let length = segment.length();
            report.total_length_mm += length;

            match segment.kind {
                SegmentKind::Rapid => {
                    report.rapid_length_mm += length;
                    report.estimated_motion_seconds += self.rapid_duration_seconds(segment);
                }
                SegmentKind::Feed | SegmentKind::Arc => {
                    report.feed_length_mm += length;
                    if let Some(feed) = segment.feed_mm_min.filter(|feed| *feed > 0.0) {
                        report.estimated_motion_seconds += length / feed * 60.0;
                        self.validate_feed(segment, feed, &mut report.warnings);
                    }
                }
            }

            report
                .warnings
                .extend(self.profile.validate_position(segment.end));
            report.final_position = segment.end;
        }

        report.warnings.sort();
        report.warnings.dedup();
        report
    }

    fn rapid_duration_seconds(&self, segment: &Segment) -> f64 {
        let delta = segment.end - segment.start;
        let x = axis_duration_minutes(delta.x, &self.profile.x);
        let y = axis_duration_minutes(delta.y, &self.profile.y);
        let z = axis_duration_minutes(delta.z, &self.profile.z);
        x.max(y).max(z) * 60.0
    }

    fn validate_feed(&self, segment: &Segment, feed_mm_min: f64, warnings: &mut Vec<String>) {
        let length = segment.length();
        if length <= f64::EPSILON {
            return;
        }

        let delta = segment.end - segment.start;
        check_axis_feed("X", delta.x, length, feed_mm_min, &self.profile.x, warnings);
        check_axis_feed("Y", delta.y, length, feed_mm_min, &self.profile.y, warnings);
        check_axis_feed("Z", delta.z, length, feed_mm_min, &self.profile.z, warnings);
    }
}

fn axis_duration_minutes(delta_mm: f64, limits: &AxisLimits) -> f64 {
    if delta_mm.abs() <= f64::EPSILON {
        return 0.0;
    }
    if !limits.max_feed_mm_min.is_finite() || limits.max_feed_mm_min <= 0.0 {
        return 0.0;
    }
    delta_mm.abs() / limits.max_feed_mm_min
}

fn check_axis_feed(
    axis: &str,
    delta_mm: f64,
    path_length_mm: f64,
    path_feed_mm_min: f64,
    limits: &AxisLimits,
    warnings: &mut Vec<String>,
) {
    let axis_feed = path_feed_mm_min * delta_mm.abs() / path_length_mm;
    if axis_feed > limits.max_feed_mm_min + 1e-9 {
        warnings.push(format!(
            "{axis}-Vorschub überschritten: {axis_feed:.1} mm/min > {:.1} mm/min",
            limits.max_feed_mm_min
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orangecnc_motion::{Segment, SegmentKind, Toolpath};

    #[test]
    fn rapid_estimate_uses_machine_axis_limit() {
        let profile = MachineProfile::default();
        let simulator = Simulator::new(profile);
        let path = Toolpath {
            segments: vec![Segment {
                start: Vec3::ZERO,
                end: Vec3::new(0.0, 0.0, -50.0),
                kind: SegmentKind::Rapid,
                feed_mm_min: None,
            }],
        };

        let report = simulator.run(&path);
        assert!((report.estimated_motion_seconds - 1.0).abs() < 1e-9);
    }
}
