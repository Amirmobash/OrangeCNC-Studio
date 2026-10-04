use std::collections::BTreeMap;

use orangecnc_domain::{
    ArcDefinition, CanonicalCommand, DistanceMode, MotionMode, Plane, Units, Vec3,
};

use crate::{lex_line, ModalState, Token};

#[derive(Debug, Clone, Default)]
pub struct ParsedLine {
    pub commands: Vec<CanonicalCommand>,
    pub words: BTreeMap<char, Vec<f64>>,
}

#[derive(Debug, Default)]
pub struct Parser {
    state: ModalState,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> &ModalState {
        &self.state
    }

    pub fn reset(&mut self) {
        self.state = ModalState::default();
    }

    pub fn parse_program(&mut self, source: &str) -> Result<Vec<ParsedLine>, String> {
        source
            .lines()
            .enumerate()
            .map(|(index, line)| {
                self.parse_line(line)
                    .map_err(|error| format!("Zeile {}: {error}", index + 1))
            })
            .collect()
    }

    pub fn parse_line(&mut self, source: &str) -> Result<ParsedLine, String> {
        let tokens = lex_line(source).map_err(|error| error.to_string())?;
        let mut words: BTreeMap<char, Vec<f64>> = BTreeMap::new();
        let mut comments = Vec::new();

        for token in tokens {
            match token {
                Token::Word(word) => words.entry(word.address).or_default().push(word.value),
                Token::Comment(comment) => comments.push(comment),
            }
        }

        self.apply_g_codes(&words)?;
        self.apply_scalar_words(&words)?;

        let mut commands = Vec::new();
        if words.contains_key(&'T') {
            if let Some(tool) = self.state.selected_tool {
                commands.push(CanonicalCommand::SelectTool(tool));
            }
        }

        if should_emit_motion(&words, self.state.motion) {
            commands.push(self.build_move(&words)?);
        }

        self.apply_m_codes(&words, &mut commands)?;
        commands.extend(comments.into_iter().map(CanonicalCommand::Comment));

        if commands.is_empty() {
            commands.push(CanonicalCommand::NoOp);
        }

        Ok(ParsedLine { commands, words })
    }

    fn apply_g_codes(&mut self, words: &BTreeMap<char, Vec<f64>>) -> Result<(), String> {
        for &g_code in words.get(&'G').into_iter().flatten() {
            match integer_code(g_code)? {
                0 => self.state.motion = MotionMode::Rapid,
                1 => self.state.motion = MotionMode::Linear,
                2 => self.state.motion = MotionMode::ArcClockwise,
                3 => self.state.motion = MotionMode::ArcCounterClockwise,
                17 => self.state.plane = Plane::Xy,
                18 => self.state.plane = Plane::Xz,
                19 => self.state.plane = Plane::Yz,
                20 => self.state.units = Units::Inch,
                21 => self.state.units = Units::Millimeter,
                90 => self.state.distance = DistanceMode::Absolute,
                91 => self.state.distance = DistanceMode::Incremental,
                _ => {}
            }
        }
        Ok(())
    }

    fn apply_scalar_words(&mut self, words: &BTreeMap<char, Vec<f64>>) -> Result<(), String> {
        if let Some(&feed) = last(words, 'F') {
            if feed < 0.0 {
                return Err(format!("Negativer Vorschub ist ungültig: {feed}"));
            }
            self.state.feed_mm_min = Some(self.state.units.to_mm(feed));
        }

        if let Some(&rpm) = last(words, 'S') {
            if rpm < 0.0 {
                return Err(format!("Negative Spindeldrehzahl ist ungültig: {rpm}"));
            }
            self.state.spindle_rpm = Some(rpm);
        }

        if let Some(&tool) = last(words, 'T') {
            self.state.selected_tool = Some(tool_number(tool)?);
        }

        Ok(())
    }

    fn build_move(&mut self, words: &BTreeMap<char, Vec<f64>>) -> Result<CanonicalCommand, String> {
        let start = self.state.position_mm;
        let end = Vec3::new(
            self.coordinate(words, 'X', start.x),
            self.coordinate(words, 'Y', start.y),
            self.coordinate(words, 'Z', start.z),
        );

        let arc = if self.state.motion.is_arc() {
            if let Some(&radius) = last(words, 'R') {
                Some(ArcDefinition::Radius(self.state.units.to_mm(radius)))
            } else {
                Some(ArcDefinition::CenterOffset {
                    i: self.state.units.to_mm(last(words, 'I').copied().unwrap_or(0.0)),
                    j: self.state.units.to_mm(last(words, 'J').copied().unwrap_or(0.0)),
                    k: self.state.units.to_mm(last(words, 'K').copied().unwrap_or(0.0)),
                })
            }
        } else {
            None
        };

        self.state.position_mm = end;
        Ok(CanonicalCommand::Move {
            mode: self.state.motion,
            plane: self.state.plane,
            start,
            end,
            arc,
            feed_mm_min: self.state.feed_mm_min,
        })
    }

    fn coordinate(&self, words: &BTreeMap<char, Vec<f64>>, address: char, current: f64) -> f64 {
        let Some(&raw) = last(words, address) else {
            return current;
        };

        let millimeters = self.state.units.to_mm(raw);
        match self.state.distance {
            DistanceMode::Absolute => millimeters,
            DistanceMode::Incremental => current + millimeters,
        }
    }

    fn apply_m_codes(
        &self,
        words: &BTreeMap<char, Vec<f64>>,
        commands: &mut Vec<CanonicalCommand>,
    ) -> Result<(), String> {
        for &m_code in words.get(&'M').into_iter().flatten() {
            match integer_code(m_code)? {
                3 => commands.push(CanonicalCommand::Spindle {
                    enabled: true,
                    clockwise: true,
                    rpm: self.state.spindle_rpm,
                }),
                4 => commands.push(CanonicalCommand::Spindle {
                    enabled: true,
                    clockwise: false,
                    rpm: self.state.spindle_rpm,
                }),
                5 => commands.push(CanonicalCommand::Spindle {
                    enabled: false,
                    clockwise: true,
                    rpm: self.state.spindle_rpm,
                }),
                6 => commands.push(CanonicalCommand::ToolChange),
                2 | 30 => commands.push(CanonicalCommand::ProgramEnd),
                _ => {}
            }
        }
        Ok(())
    }
}

fn integer_code(value: f64) -> Result<i32, String> {
    let rounded = value.round();
    if (value - rounded).abs() > 1e-9 {
        return Err(format!(
            "Nicht ganzzahliger G/M-Code wird noch nicht unterstützt: {value}"
        ));
    }
    Ok(rounded as i32)
}

fn tool_number(value: f64) -> Result<u32, String> {
    if value < 0.0 || value > u32::MAX as f64 {
        return Err(format!("Ungültige Werkzeugnummer: {value}"));
    }

    let rounded = value.round();
    if (value - rounded).abs() > 1e-9 {
        return Err(format!("Werkzeugnummer muss ganzzahlig sein: {value}"));
    }

    Ok(rounded as u32)
}

fn last(map: &BTreeMap<char, Vec<f64>>, key: char) -> Option<&f64> {
    map.get(&key).and_then(|values| values.last())
}

fn should_emit_motion(words: &BTreeMap<char, Vec<f64>>, motion: MotionMode) -> bool {
    let has_endpoint = ['X', 'Y', 'Z']
        .into_iter()
        .any(|address| words.contains_key(&address));

    if has_endpoint {
        return true;
    }

    motion.is_arc()
        && ['I', 'J', 'K', 'R']
            .into_iter()
            .any(|address| words.contains_key(&address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compact_absolute_motion() {
        let mut parser = Parser::new();
        parser.parse_line("G21G90G1X10Y20F1200").unwrap();
        assert_eq!(parser.state().position_mm, Vec3::new(10.0, 20.0, 0.0));
        assert_eq!(parser.state().feed_mm_min, Some(1200.0));
    }

    #[test]
    fn converts_inches_to_mm() {
        let mut parser = Parser::new();
        parser.parse_line("G20 G90 G1 X1").unwrap();
        assert!((parser.state().position_mm.x - 25.4).abs() < 1e-9);
    }

    #[test]
    fn incremental_mode_accumulates() {
        let mut parser = Parser::new();
        parser.parse_line("G90 G1 X10").unwrap();
        parser.parse_line("G91 G1 X2.5").unwrap();
        assert_eq!(parser.state().position_mm.x, 12.5);
    }

    #[test]
    fn full_circle_ijk_arc_is_emitted_without_endpoint_words() {
        let mut parser = Parser::new();
        parser.parse_line("G0 X10 Y0").unwrap();
        let parsed = parser.parse_line("G3 I-10 J0 F500").unwrap();

        assert!(matches!(
            parsed.commands.as_slice(),
            [CanonicalCommand::Move {
                mode: MotionMode::ArcCounterClockwise,
                start,
                end,
                ..
            }] if start == end
        ));
    }

    #[test]
    fn fractional_m_code_is_rejected() {
        let mut parser = Parser::new();
        assert!(parser.parse_line("M3.2").is_err());
    }

    #[test]
    fn fractional_tool_number_is_rejected() {
        let mut parser = Parser::new();
        assert!(parser.parse_line("T1.5").is_err());
    }
}
