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
        source.lines().map(|line| self.parse_line(line)).collect()
    }

    pub fn parse_line(&mut self, source: &str) -> Result<ParsedLine, String> {
        let tokens = lex_line(source).map_err(|e| e.to_string())?;
        let mut words: BTreeMap<char, Vec<f64>> = BTreeMap::new();
        let mut comments = Vec::new();

        for token in tokens {
            match token {
                Token::Word(word) => words.entry(word.address).or_default().push(word.value),
                Token::Comment(comment) => comments.push(comment),
            }
        }

        self.apply_g_codes(&words)?;
        self.apply_scalar_words(&words);

        let mut commands = Vec::new();
        if words.contains_key(&'T') {
            if let Some(tool) = self.state.selected_tool {
                commands.push(CanonicalCommand::SelectTool(tool));
            }
        }

        if has_motion_coordinates(&words) {
            commands.push(self.build_move(&words)?);
        }

        self.apply_m_codes(&words, &mut commands);
        commands.extend(comments.into_iter().map(CanonicalCommand::Comment));

        if commands.is_empty() {
            commands.push(CanonicalCommand::NoOp);
        }

        Ok(ParsedLine { commands, words })
    }

    fn apply_g_codes(&mut self, words: &BTreeMap<char, Vec<f64>>) -> Result<(), String> {
        for &g in words.get(&'G').into_iter().flatten() {
            let code = rounded_code(g)?;
            match code {
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

    fn apply_scalar_words(&mut self, words: &BTreeMap<char, Vec<f64>>) {
        if let Some(&feed) = last(words, 'F') {
            self.state.feed_mm_min = Some(self.state.units.to_mm(feed));
        }
        if let Some(&rpm) = last(words, 'S') {
            self.state.spindle_rpm = Some(rpm);
        }
        if let Some(&tool) = last(words, 'T') {
            if tool >= 0.0 {
                self.state.selected_tool = Some(tool.round() as u32);
            }
        }
    }

    fn build_move(&mut self, words: &BTreeMap<char, Vec<f64>>) -> Result<CanonicalCommand, String> {
        let start = self.state.position_mm;
        let end = Vec3::new(
            self.coordinate(words, 'X', start.x),
            self.coordinate(words, 'Y', start.y),
            self.coordinate(words, 'Z', start.z),
        );

        let arc = if matches!(self.state.motion, MotionMode::ArcClockwise | MotionMode::ArcCounterClockwise) {
            if let Some(&r) = last(words, 'R') {
                Some(ArcDefinition::Radius(self.state.units.to_mm(r)))
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
        let Some(&raw) = last(words, address) else { return current };
        let mm = self.state.units.to_mm(raw);
        match self.state.distance {
            DistanceMode::Absolute => mm,
            DistanceMode::Incremental => current + mm,
        }
    }

    fn apply_m_codes(&self, words: &BTreeMap<char, Vec<f64>>, out: &mut Vec<CanonicalCommand>) {
        for &m in words.get(&'M').into_iter().flatten() {
            let Ok(code) = rounded_code(m) else { continue };
            match code {
                3 => out.push(CanonicalCommand::Spindle {
                    enabled: true,
                    clockwise: true,
                    rpm: self.state.spindle_rpm,
                }),
                4 => out.push(CanonicalCommand::Spindle {
                    enabled: true,
                    clockwise: false,
                    rpm: self.state.spindle_rpm,
                }),
                5 => out.push(CanonicalCommand::Spindle {
                    enabled: false,
                    clockwise: true,
                    rpm: self.state.spindle_rpm,
                }),
                6 => out.push(CanonicalCommand::ToolChange),
                2 | 30 => out.push(CanonicalCommand::ProgramEnd),
                _ => {}
            }
        }
    }
}

fn rounded_code(value: f64) -> Result<i32, String> {
    let rounded = value.round();
    if (value - rounded).abs() > 1e-9 {
        return Err(format!("Nicht ganzzahliger G/M-Code wird noch nicht unterstützt: {value}"));
    }
    Ok(rounded as i32)
}

fn last(map: &BTreeMap<char, Vec<f64>>, key: char) -> Option<&f64> {
    map.get(&key).and_then(|values| values.last())
}

fn has_motion_coordinates(words: &BTreeMap<char, Vec<f64>>) -> bool {
    ['X', 'Y', 'Z'].into_iter().any(|key| words.contains_key(&key))
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
}
