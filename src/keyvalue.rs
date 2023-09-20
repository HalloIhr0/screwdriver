use std::{fs, io, iter::Peekable, path::Path, str::Chars};

use crate::gameinfo::Gameinfo;

/// Parser for the KeyValues format
/// https://developer.valvesoftware.com/wiki/KeyValues
/// Conditionl statements don't work (for now)
#[derive(Debug, Clone)]
pub enum KeyValues {
    Value { value: String },
    List { subkeys: Vec<(String, KeyValues)> },
}

#[derive(thiserror::Error, Debug, PartialEq)]
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
    /// Reads a file and parses it into KeyValues
    /// Errors if the file has invalid syntax
    pub fn parse(file: &Path) -> Result<Self, KeyValuesError> {
        let content = fs::read_to_string(file)
            .map_err(|_| KeyValuesError::InvalidFile(file.to_string_lossy().to_string()))?;
        let mut content = content.chars().peekable();
        Self::parse_internal(&mut content, &|x| {
            let include_path = file
                .parent()
                .expect("is safe if we can read the file")
                .join(x);
            Self::parse(&include_path)
        })
    }

    pub fn parse_from_searchpath(
        gameinfo: &Gameinfo,
        path: &str,
        extension: &str,
    ) -> Result<Self, KeyValuesError> {
        let content = gameinfo
            .get_file(path, extension)
            .ok_or(KeyValuesError::InvalidFile(format!(
                "(SearchPath) {}.{}",
                path, extension
            )))?;
        let content = String::from_utf8(content).or(Err(KeyValuesError::InvalidFile(format!(
            "(SearchPath) {}.{}",
            path, extension
        ))))?;
        let mut content = content.chars().peekable();
        Self::parse_internal(&mut content, &|x| {
            let (p, e) = x
                .split_once('.')
                .ok_or(KeyValuesError::InvalidFile(format!(
                    "(SearchPath) {} (inside {}.{})",
                    x, path, extension
                )))?; // Should be safe to assume filenames only contain one dot
            Self::parse_from_searchpath(gameinfo, p, e)
        })
    }

    /// Gets the first value with the specified name
    /// Return None if the name does not exist
    /// Only works on KeyValues::List
    /// Name should always be lowercase
    pub fn get(&self, name: &str) -> Option<&Self> {
        match self {
            KeyValues::Value { .. } => {
                eprintln!("Tried to get element from KeyValues::Value");
                None
            }
            KeyValues::List { subkeys } => {
                for (key_name, value) in subkeys {
                    if key_name == name {
                        return Some(value);
                    }
                }
                None
            }
        }
    }

    /// Gets every value with the specified name
    /// Only works on KeyValues::List
    /// Name should always be lowercase
    pub fn get_all(&self, name: &str) -> Vec<&Self> {
        match self {
            KeyValues::Value { .. } => {
                eprintln!("Tried to get elements from KeyValues::Value");
                vec![]
            }
            KeyValues::List { subkeys } => subkeys
                .iter()
                .filter_map(|(key_name, value)| if key_name == name { Some(value) } else { None })
                .collect(),
        }
    }

    /// Gets every pair of key/value in this subkey
    /// Only works on KeyValues::List
    /// Keys are always lowercase
    pub fn get_all_kv_pairs(&self) -> Vec<(&String, &Self)> {
        match self {
            KeyValues::Value { .. } => {
                eprintln!("Tried to get KVs from KeyValues::Value");
                vec![]
            }
            KeyValues::List { subkeys } => {
                subkeys.iter().map(|(name, value)| (name, value)).collect()
            }
        }
    }

    /// Gets the actual value with the specified name
    /// Only works on KeyValues::Value
    pub fn get_value(&self) -> Option<&String> {
        match self {
            KeyValues::Value { value } => Some(value),
            KeyValues::List { .. } => {
                eprintln!("Tried to get value from KeyValues::List");
                None
            }
        }
    }

    /// Writes KeyValues into a file with the right syntax
    /// Errors if writing fails
    pub fn write(&self, file: &Path) -> Result<(), io::Error> {
        let content = match self {
            KeyValues::Value { .. } => {
                eprintln!("Tried to write KeyValues::Value. Should only be KeyValues::List. Something went wrong!");
                return Err(io::ErrorKind::InvalidData.into());
            }
            KeyValues::List { subkeys } => subkeys
                .iter()
                .map(|(name, value)| value.get_string(name, 0))
                .collect::<Vec<String>>()
                .join("\n"),
        };

        fs::write(file, content)
    }

    pub fn parse_internal(
        content: &mut Peekable<Chars>,
        file_open_handler: &dyn Fn(String) -> Result<Self, KeyValuesError>,
    ) -> Result<Self, KeyValuesError> {
        let mut current = vec![];
        let mut stack: Vec<(String, Vec<(String, KeyValues)>)> = vec![];
        'parse: loop {
            skip_whitespace(content);
            match content.peek() {
                None => {
                    break 'parse;
                }
                Some('}') => {
                    let (name, new_current) = stack.pop().ok_or(KeyValuesError::UnexpectedEnd)?;
                    let part = Self::List { subkeys: current };
                    current = new_current;
                    current.push((name.to_lowercase(), part));
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
            let name = read_text(content)?;
            skip_whitespace(content);
            match content.peek().ok_or(KeyValuesError::UnexpectedEnd)? {
                '{' => {
                    content.next();
                    stack.push((name.to_lowercase(), current));
                    current = vec![];
                }
                '}' => {
                    return Err(KeyValuesError::UnexpectedClosingBrace(name));
                }
                _ => {
                    let value = read_text(content)?;
                    // Handle macros
                    if name.starts_with('#') {
                        match name.as_str() {
                            "#include" | "#base" => {
                                let include_data = file_open_handler(value)?;
                                if let Self::List { mut subkeys } = include_data {
                                    current.append(&mut subkeys);
                                }
                            }
                            _ => return Err(KeyValuesError::UnknownMacro(name)),
                        }
                    } else {
                        current.push((name.to_lowercase(), Self::Value { value }))
                    }
                }
            }
        }
        if !stack.is_empty() {
            return Err(KeyValuesError::UnexpectedEnd);
        }
        Ok(Self::List { subkeys: current })
    }

    fn get_string(&self, name: &str, indent: u16) -> String {
        // You shouldn't have more than 2^16 levels of indentation. This is fine
        let indent_str = String::from(" ").repeat((indent * 4) as usize);
        let name = escape_token(name);
        match self {
            KeyValues::Value { value } => {
                format!("{indent_str}\"{name}\" \"{}\"", escape_token(value))
            }
            KeyValues::List { subkeys } => {
                format!(
                    "{indent_str}\"{name}\"\n{indent_str}{{\n{}\n{indent_str}}}",
                    subkeys
                        .iter()
                        .map(|(name, value)| value.get_string(name, indent + 1))
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            }
        }
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

fn escape_token(token: &str) -> String {
    token
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\"', "\\\"")
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
    #[test]
    fn test_escape_token() {
        assert_eq!(
            escape_token(&"Hello\nWorld\tHello\\World\"Hello".to_string()),
            "Hello\\nWorld\\tHello\\\\World\\\"Hello"
        )
    }
}
