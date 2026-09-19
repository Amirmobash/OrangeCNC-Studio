use orangecnc_domain::Vec3;
use orangecnc_machine::MachineProfile;
use orangecnc_motion::{SegmentKind, Toolpath};

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
    pub fn new(profile: MachineProfile) -> Self { Self { profile } }

    pub fn run(&self, path:&Toolpath) -> SimulationReport {
        let mut report=SimulationReport::default();
        report.segment_count=path.segments.len();
        for segment in &path.segments {
            let len=segment.length();
            report.total_length_mm += len;
            match segment.kind {
                SegmentKind::Rapid => {
                    report.rapid_length_mm += len;
                    report.estimated_motion_seconds += len / 6_000.0 * 60.0;
                }
                SegmentKind::Feed | SegmentKind::Arc => {
                    report.feed_length_mm += len;
                    if let Some(feed)=segment.feed_mm_min.filter(|f| *f>0.0) {
                        report.estimated_motion_seconds += len/feed*60.0;
                    }
                }
            }
            report.warnings.extend(self.profile.validate_position(segment.end));
            report.final_position=segment.end;
        }
        report.warnings.sort(); report.warnings.dedup();
        report
    }
}
