use std::{fs, iter::Peekable, path::Path, str::Chars};
use thiserror::Error;

/// Parser for the KeyValues format
/// https://developer.valvesoftware.com/wiki/KeyValues
/// Conditionl statements don't work (for now)
#[derive(Debug)]
pub enum KeyValues {
    Value { value: String },
    List { subkeys: Vec<(String, KeyValues)> },
}

#[derive(Error, Debug, PartialEq)]
pub enum KeyValuesError {
    #[error("invalid escape sequence \\{0}")]
    InvalidEscape(char),
    #[error("unexpected end")]
    UnexpectedEnd,
    #[error("unexpected closing brace after {0}")]
    UnexpectedClosingBrace(String),
    #[error("unknown macro \"{0}\"")]
    UnknownMacro(String),
    #[error("invalid file \"{0}\"")]
    InvalidFile(String),
}

impl KeyValues {
    pub fn parse(file: &Path) -> Result<Self, KeyValuesError> {
        let content = fs::read_to_string(file)
            .map_err(|_| KeyValuesError::InvalidFile(file.to_string_lossy().to_string()))?;
        let mut content = content.chars().peekable();
        let mut current = vec![];
        let mut stack = vec![];
        'parse: loop {
            skip_whitespace(&mut content);
            match content.peek() {
                None => {
                    break 'parse;
                }
                Some('}') => {
                    let (name, new_current) = stack.pop().ok_or(KeyValuesError::UnexpectedEnd)?;
                    let part = Self::List { subkeys: current };
                    current = new_current;
                    current.push((name, part));
                    content.next();
                    continue 'parse;
                }
                Some('/') => {
                    content.next();
                    match content.peek() {
                        Some('*') => {
                            let mut last = content.next();
                            loop {
                                match (last, content.next()) {
                                    (Some('*'), Some('/')) => {
                                        continue 'parse;
                                    }
                                    (_, None) => {
                                        break 'parse;
                                    }
                                    (_, x) => {
                                        last = x;
                                    }
                                }
                            }
                        }
                        _ => {
                            loop {
                                // This should only trigger for 2 slashes, but there is a bug in the original parser which only looks for one
                                match content.next() {
                                    None => {
                                        break 'parse;
                                    }
                                    Some('\n') => {
                                        continue 'parse;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            let name = read_text(&mut content)?;
            skip_whitespace(&mut content);
            match content.peek().ok_or(KeyValuesError::UnexpectedEnd)? {
                '{' => {
                    content.next();
                    stack.push((name, current));
                    current = vec![];
                }
                '}' => {
                    return Err(KeyValuesError::UnexpectedClosingBrace(name));
                }
                _ => {
                    let value = read_text(&mut content)?;
                    // Handle macros
                    if name.starts_with('#') {
                        match name.as_str() {
                            "#include" | "#base" => {
                                let include_path = file
                                    .parent()
                                    .expect("is safe if we can read the file")
                                    .join(value);
                                let include_data = Self::parse(&include_path)?;
                                if let Self::List { mut subkeys } = include_data {
                                    current.append(&mut subkeys);
                                }
                            }
                            _ => return Err(KeyValuesError::UnknownMacro(name)),
                        }
                    } else {
                        current.push((name, Self::Value { value }))
                    }
                }
            }
        }
        if !stack.is_empty() {
            return Err(KeyValuesError::UnexpectedEnd);
        }
        Ok(Self::List { subkeys: current })
    }
}

fn skip_whitespace(content: &mut Peekable<Chars>) {
    loop {
        match content.peek() {
            Some(' ' | '\r' | '\n' | '\t') => {
                content.next();
            }
            _ => return,
        }
    }
    // the above code does the same as this, just actually changing the iter
    // code.skip_while(|c| *c == ' ' || *c == '\r' || *c == '\n' || *c == '\t');
}

fn read_text(content: &mut Peekable<Chars>) -> Result<String, KeyValuesError> {
    let mut text = String::new();
    let mut require_quote = false;
    match content.next().ok_or(KeyValuesError::UnexpectedEnd)? {
        '"' => {
            require_quote = true;
        }
        c => text.push(c),
    }
    loop {
        // Ugly code
        match content.peek() {
            None => {
                if require_quote {
                    return Err(KeyValuesError::UnexpectedEnd);
                }
                break;
            }
            Some('\\') => {
                content.next(); // To acctually advance the peek
                match content.next().ok_or(KeyValuesError::UnexpectedEnd)? {
                    'n' => text.push('\n'),
                    't' => text.push('\t'),
                    '\\' => text.push('\\'),
                    '"' => text.push('"'),
                    c => return Err(KeyValuesError::InvalidEscape(c)),
                };
            }
            Some('"') => {
                if require_quote {
                    content.next();
                }
                break;
            }
            Some(' ' | '\r' | '\n' | '\t' | '{' | '}') => {
                if !require_quote {
                    break;
                }
                text.push(content.next().expect("has been checked by peek before"));
            }
            Some(c) => {
                text.push(*c);
                content.next();
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
