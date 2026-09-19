use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Word {
    pub address: char,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(Word),
    Comment(String),
}

#[derive(Debug, Error, PartialEq)]
pub enum LexError {
    #[error("Nach Adresse {0} fehlt ein Wert")]
    MissingValue(char),
    #[error("Ungültige Zahl bei Adresse {0}: {1}")]
    InvalidNumber(char, String),
    #[error("Ungültiges Zeichen: {0}")]
    InvalidCharacter(char),
    #[error("Kommentar wurde nicht geschlossen")]
    UnclosedComment,
}

pub fn lex_line(source: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        let c = chars[index];
        if c.is_whitespace() || c == '%' {
            index += 1;
            continue;
        }

        if c == ';' {
            let comment: String = chars[index + 1..].iter().collect();
            tokens.push(Token::Comment(comment.trim().to_owned()));
            break;
        }

        if c == '(' {
            let start = index + 1;
            index += 1;
            while index < chars.len() && chars[index] != ')' {
                index += 1;
            }
            if index >= chars.len() {
                return Err(LexError::UnclosedComment);
            }
            let comment: String = chars[start..index].iter().collect();
            tokens.push(Token::Comment(comment.trim().to_owned()));
            index += 1;
            continue;
        }

        if !c.is_ascii_alphabetic() {
            return Err(LexError::InvalidCharacter(c));
        }

        let address = c.to_ascii_uppercase();
        index += 1;
        let start = index;
        if index < chars.len() && matches!(chars[index], '+' | '-') {
            index += 1;
        }

        let mut saw_digit = false;
        let mut saw_dot = false;
        while index < chars.len() {
            match chars[index] {
                d if d.is_ascii_digit() => {
                    saw_digit = true;
                    index += 1;
                }
                '.' if !saw_dot => {
                    saw_dot = true;
                    index += 1;
                }
                _ => break,
            }
        }

        if !saw_digit {
            return Err(LexError::MissingValue(address));
        }
        let raw: String = chars[start..index].iter().collect();
        let value = raw
            .parse::<f64>()
            .map_err(|_| LexError::InvalidNumber(address, raw.clone()))?;
        tokens.push(Token::Word(Word { address, value }));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_words_are_lexed() {
        let tokens = lex_line("G1X12.5Y-3.25F1200").unwrap();
        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn semicolon_comment_ends_line() {
        let tokens = lex_line("G1 X10 ; Schlichtgang").unwrap();
        assert!(matches!(tokens.last(), Some(Token::Comment(c)) if c == "Schlichtgang"));
    }
}
