use std::{iter::Peekable, str::Chars};
use thiserror::Error;

/// Parser for the KeyValues format
/// https://developer.valvesoftware.com/wiki/KeyValues
#[derive(Debug)]
pub enum KeyValues {
    Root {
        subkeys: Vec<KeyValues>,
    },
    KeyValue {
        key: String,
        value: String,
    },
    List {
        name: String,
        subkeys: Vec<KeyValues>,
    },
}

#[derive(Error, Debug, PartialEq)]
pub enum KeyValuesError {
    #[error("invalid escape sequence \\{0}")]
    InvalidEscape(char),
    #[error("unexpected end")]
    UnexpectedEnd,
    #[error("unexpected closing brace after {0}")]
    UnexpectedClosingBrace(String),
}

impl KeyValues {
    pub fn parse(code: &str) -> Result<Self, KeyValuesError> {
        let mut code = code.chars().peekable();
        let mut current = vec![];
        let mut stack = vec![];
        loop {
            skip_whitespace(&mut code);
            match code.peek() {
                None => {
                    break;
                }
                Some('}') => {
                    let (name, new_current) = stack.pop().ok_or(KeyValuesError::UnexpectedEnd)?;
                    let part = Self::List { name: name, subkeys: current };
                    current = new_current;
                    current.push(part);
                    code.next();
                    continue;
                }
                _ => {}
            }
            let name = read_text(&mut code)?;
            skip_whitespace(&mut code);
            match code.peek().ok_or(KeyValuesError::UnexpectedEnd)? {
                '{' => {
                    code.next();
                    stack.push((name, current));
                    current = vec![];
                }
                '}' => {
                    return Err(KeyValuesError::UnexpectedClosingBrace(name));
                }
                _ => {
                    let value = read_text(&mut code)?;
                    current.push(Self::KeyValue {
                        key: name,
                        value: value,
                    })
                }
            }
        }
        if !stack.is_empty() {
            return Err(KeyValuesError::UnexpectedEnd);
        }
        Ok(Self::Root { subkeys: current })
    }
}

fn skip_whitespace(code: &mut Peekable<Chars>) {
    loop {
        match code.peek() {
            Some(' ' | '\r' | '\n' | '\t') => {
                code.next();
            }
            _ => return,
        }
    }
    // the above code does the same as this, just actually changing the iter
    // code.skip_while(|c| *c == ' ' || *c == '\r' || *c == '\n' || *c == '\t');
}

fn read_text(code: &mut Peekable<Chars>) -> Result<String, KeyValuesError> {
    let mut text = String::new();
    let mut require_quote = false;
    match code.next().ok_or(KeyValuesError::UnexpectedEnd)? {
        '"' => {
            require_quote = true;
        }
        c => text.push(c),
    }
    loop {
        // Ugly code
        match code.peek() {
            None => {
                if require_quote {
                    return Err(KeyValuesError::UnexpectedEnd);
                }
                break;
            }
            Some('\\') => {
                code.next(); // To acctually advance the peek
                match code.next().ok_or(KeyValuesError::UnexpectedEnd)? {
                    'n' => text.push('\n'),
                    't' => text.push('\t'),
                    '\\' => text.push('\\'),
                    '"' => text.push('"'),
                    c => return Err(KeyValuesError::InvalidEscape(c)),
                };
            }
            Some('"') => {
                if require_quote {
                    code.next();
                }
                break;
            }
            Some(' ' | '\r' | '\n' | '\t' | '{' | '}') => {
                if !require_quote {
                    break;
                }
                text.push(code.next().expect("has been checked by peek before"));
            }
            Some(c) => {
                text.push(*c);
                code.next();
            }
        }
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_text_unquoted() {
        {
            let mut iter = "Hello World".chars().peekable();
            assert_eq!(read_text(&mut iter), Ok(String::from("Hello")));
            assert_eq!(iter.collect::<String>(), String::from(" World"));
        }
        {
            let mut iter = "Hello{World".chars().peekable();
            assert_eq!(read_text(&mut iter), Ok(String::from("Hello")));
            assert_eq!(iter.collect::<String>(), String::from("{World"));
        }
        {
            let mut iter = "Hello\"World".chars().peekable();
            assert_eq!(read_text(&mut iter), Ok(String::from("Hello")));
            assert_eq!(iter.collect::<String>(), String::from("\"World"));
        }
    }
    #[test]
    fn test_read_text_quoted() {
        {
            let mut iter = "\"Hello World\"".chars().peekable();
            assert_eq!(read_text(&mut iter), Ok(String::from("Hello World")));
            assert_eq!(iter.collect::<String>(), String::from(""));
        }
        {
            let mut iter = "\"Hello{World\"".chars().peekable();
            assert_eq!(read_text(&mut iter), Ok(String::from("Hello{World")));
            assert_eq!(iter.collect::<String>(), String::from(""));
        }
    }
    #[test]
    fn test_read_text_escape() {
        {
            let mut iter = "Hello\\nbeatiful World".chars().peekable();
            assert_eq!(read_text(&mut iter), Ok(String::from("Hello\nbeatiful")));
            assert_eq!(iter.collect::<String>(), String::from(" World"));
        }
        {
            let mut iter = "Hello\\xbeatiful World".chars().peekable();
            assert_eq!(
                read_text(&mut iter),
                Err(KeyValuesError::InvalidEscape('x'))
            );
        }
        {
            let mut iter = "\"Hello\\nbeatiful World\"".chars().peekable();
            assert_eq!(
                read_text(&mut iter),
                Ok(String::from("Hello\nbeatiful World"))
            );
            assert_eq!(iter.collect::<String>(), String::from(""));
        }
        {
            let mut iter = "\"Hello\\xbeatiful World\"".chars().peekable();
            assert_eq!(
                read_text(&mut iter),
                Err(KeyValuesError::InvalidEscape('x'))
            );
        }
    }
    #[test]
    fn test_read_text_unexpected_end() {
        {
            let mut iter = "".chars().peekable();
            assert_eq!(read_text(&mut iter), Err(KeyValuesError::UnexpectedEnd));
        }
        {
            let mut iter = "Hello\\".chars().peekable();
            assert_eq!(read_text(&mut iter), Err(KeyValuesError::UnexpectedEnd));
        }
        {
            let mut iter = "\"Hello World".chars().peekable();
            assert_eq!(read_text(&mut iter), Err(KeyValuesError::UnexpectedEnd));
        }
    }
}
