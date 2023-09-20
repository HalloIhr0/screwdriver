use std::{collections::HashMap, fs, iter::Peekable, path::Path, str::Chars};

pub struct FGD {
    entity_defs: HashMap<String, EntityDefinition>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum FGDerror {
    #[error("Unexpected end")]
    UnexpectedEnd,
    #[error("Invalid FGD file \"{0}\"")]
    InvalidFile(String),
    #[error("{0}")]
    InvalidSyntax(String),
    #[error("Unknown class \"{0}\"")]
    UnknownClass(String),
    #[error("Unknown type \"{0}\"")]
    UnknownType(String),
}

#[derive(Debug)]
struct EntityDefinition {
    entity_type: EntityType,
    properties: Vec<()>,
    description: Option<String>,
    keyvalues: Vec<EntityKeyvalue>,
    inputs: Vec<EntityInputOutput>,
    outputs: Vec<EntityInputOutput>,
}

#[derive(Debug)]
enum EntityType {
    PointClass,
}

#[derive(Debug)]
struct EntityInputOutput {
    name: String,
    value_type: InputOutputType,
    description: Option<String>,
}

#[derive(Debug)]
enum InputOutputType {
    Void,
    String,
    Integer,
    Float,
    Boolean,
}

#[derive(Debug)]
struct EntityKeyvalue {
    name: String,
    value_type: KeyvalueType,
    dispname: Option<String>,
    default: Option<String>,
    description: Option<String>,
}

#[derive(Debug)]
enum KeyvalueType {
    String,
    Integer,
    Float,
    Boolean,
    Choices(Vec<(String, String)>),
    Flags(HashMap<u8, (String, bool)>),
}

impl FGD {
    pub fn parse(file: &Path) -> Result<Self, FGDerror> {
        let file_content = fs::read_to_string(file)
            .map_err(|_| FGDerror::InvalidFile(file.to_string_lossy().to_string()))?;
        let mut content = String::new();
        for line in file_content.lines() {
            // TODO: should actually iterate through line to detect comments at the end of a line after the code
            let mut iter = line.chars().peekable();
            skip_whitespace(&mut iter);
            if iter.peek() == Some(&'/') {
                iter.next();
                if iter.peek() != Some(&'/') {
                    // Otherwise its a comment
                    content.push('/');
                    content.push_str(&iter.collect::<String>());
                }
            } else {
                content.push_str(&iter.collect::<String>());
            }
            content.push('\n');
        }
        let mut content = content.chars().peekable();
        let mut entity_defs = HashMap::new();
        loop {
            skip_whitespace(&mut content);
            if content.peek().is_none() {
                break;
            }
            let (class, props) = read_name_properties(&mut content)?;
            match class.as_str() {
                "@BaseClass" => {
                    let (classname, definition) =
                        read_entity(&mut content, EntityType::PointClass)?;
                    println!("{} ->\n{:#?}", classname, definition);
                }
                "@PointClass" => {
                    let (classname, definition) =
                        read_entity(&mut content, EntityType::PointClass)?;
                    println!("{} ->\n{:#?}", classname, definition);
                    entity_defs.insert(classname, definition);
                }
                "@NPCClass" => {
                    let (classname, definition) =
                        read_entity(&mut content, EntityType::PointClass)?;
                    println!("{} ->\n{:#?}", classname, definition);
                    entity_defs.insert(classname, definition);
                }
                "@SolidClass" => {
                    let (classname, definition) =
                        read_entity(&mut content, EntityType::PointClass)?;
                    println!("{} ->\n{:#?}", classname, definition);
                    entity_defs.insert(classname, definition);
                }
                "@KeyFrameClass" => {
                    let (classname, definition) =
                        read_entity(&mut content, EntityType::PointClass)?;
                    println!("{} ->\n{:#?}", classname, definition);
                    entity_defs.insert(classname, definition);
                }
                "@MoveClass" => {
                    let (classname, definition) =
                        read_entity(&mut content, EntityType::PointClass)?;
                    println!("{} ->\n{:#?}", classname, definition);
                    entity_defs.insert(classname, definition);
                }
                "@FilterClass" => {
                    let (classname, definition) =
                        read_entity(&mut content, EntityType::PointClass)?;
                    println!("{} ->\n{:#?}", classname, definition);
                    entity_defs.insert(classname, definition);
                }
                "@mapsize" => {}
                _ => return Err(FGDerror::UnknownClass(class)),
            }
        }
        Ok(FGD { entity_defs })
    }
}

// TODO: same as keyvalues::skip_whitespace, maybe combine into the same function
fn skip_whitespace(content: &mut Peekable<Chars>) {
    loop {
        match content.peek() {
            Some(' ' | '\r' | '\n' | '\t') => {
                content.next();
            }
            _ => return,
        }
    }
}

fn read_text(content: &mut Peekable<Chars>) -> String {
    let mut result = String::new();
    let mut require_quote = false;
    if content.peek() == Some(&'"') {
        require_quote = true;
        content.next();
    }
    loop {
        match content.peek() {
            Some('"') => {
                if require_quote {
                    content.next();
                    return result;
                }
                result.push('"');
                content.next();
            }
            None => {
                return result;
            }
            Some(c @ ' ' | c @ '\r' | c @ '\n' | c @ '\t') => {
                if !require_quote {
                    return result;
                }
                result.push(*c);
                content.next();
            }
            Some(c) => {
                result.push(*c);
                content.next();
            }
        }
    }
}

/// if properties dont exist, the second string will be empty
fn read_name_properties(content: &mut Peekable<Chars>) -> Result<(String, String), FGDerror> {
    let mut name = String::new();
    let mut properties = String::new();
    let mut depth: u32 = 0;
    loop {
        match content.peek() {
            None => {
                if depth == 0 {
                    return Ok((name, properties));
                }
                return Err(FGDerror::UnexpectedEnd);
            }
            Some(c @ ' ' | c @ '\r' | c @ '\n' | c @ '\t') => {
                if depth == 0 {
                    return Ok((name, properties));
                }
                properties.push(*c);
            }
            Some('(') => {
                depth += 1;
            }
            Some(')') => {
                depth -= 1;
            }
            Some(c) => {
                if depth == 0 {
                    name.push(*c);
                } else {
                    properties.push(*c)
                }
            }
        }
        content.next();
    }
}

fn read_entity(
    content: &mut Peekable<Chars>,
    entity_type: EntityType,
) -> Result<(String, EntityDefinition), FGDerror> {
    let classname;
    let description;
    loop {
        skip_whitespace(content);
        let (property, value) = read_name_properties(content).unwrap();
        if property == "=" {
            // This only works if the "=" is seperated by whitespace
            // Maybe check inside the property
            skip_whitespace(content);
            classname = read_text(content);
            skip_whitespace(content);
            let next = read_text(content);
            if next == ":" {
                // Same issue as above
                let mut description_text = String::new();
                loop {
                    skip_whitespace(content);
                    // Not quite accurate since descriptions have to be in quotes, but it should work and is easy
                    description_text.push_str(&read_text(content));
                    skip_whitespace(content);
                    match read_text(content).as_str() {
                        "[" => {
                            break;
                        }
                        "+" => {}
                        x => {
                            return Err(FGDerror::InvalidSyntax(format!(
                                "Expected \"[\" or \"+\" after description block, found \"{x}\" (in \"{classname}\")"
                            )));
                        }
                    }
                }
                description = Some(description_text);
                break;
            } else if next == "[" {
                description = None;
                break;
            }
            return Err(FGDerror::InvalidSyntax(format!(
                "Expected \":\" or \"[\" after classname, found \"{next}\" (in \"{classname}\")"
            )));
        }
    }
    let mut keyvalues = vec![];
    let mut inputs = vec![];
    let mut outputs = vec![];
    loop {
        skip_whitespace(content);
        let (name, type_name) = read_name_properties(content)?;
        if name == "]" {
            break;
        }
        if name == "input" {
            inputs.push(read_io(content)?);
        } else if name == "output" {
            outputs.push(read_io(content)?);
        } else {
            let mut dispname = None;
            let mut default = None;
            let mut description = None;
            skip_whitespace(content);
            if content.peek() == Some(&':') {
                // Name exists
                content.next();
                skip_whitespace(content);
                dispname = Some(read_text(content));
                skip_whitespace(content);
                if content.peek() == Some(&':') {
                    // Default may exist
                    content.next();
                    skip_whitespace(content);
                    if content.peek() != Some(&':') {
                        // Default exists
                        default = Some(read_text(content));
                        skip_whitespace(content);
                    }
                    if content.peek() == Some(&':') {
                        // Description exists
                        content.next();
                        skip_whitespace(content);
                        description = Some(read_text(content));
                    }
                }
            }
            println!("{name}");
            let value_type = match type_name.as_str() {
                "string" => KeyvalueType::String,
                "integer" => KeyvalueType::Integer,
                "float" => KeyvalueType::Float,
                "boolean" => KeyvalueType::Boolean,
                "choices" => {
                    skip_whitespace(content);
                    if content.next() != Some('=') {
                        return Err(FGDerror::InvalidSyntax(format!(
                            "Expected \"=\" after choices keyvalue (in \"{classname}\")"
                        )));
                    }
                    skip_whitespace(content);
                    if content.next() != Some('[') {
                        return Err(FGDerror::InvalidSyntax(format!(
                            "Expected \"[\" after choices keyvalue (in \"{classname}\")"
                        )));
                    }
                    let mut options = vec![];
                    loop {
                        skip_whitespace(content);
                        let value = read_text(content);
                        if value == "]" {
                            break;
                        }
                        skip_whitespace(content);
                        if content.next() != Some(':') {
                            return Err(FGDerror::InvalidSyntax(format!(
                                "Expected \":\" after choices value (in \"{classname}\")"
                            )));
                        }
                        skip_whitespace(content);
                        let disp = read_text(content);
                        options.push((value, disp));
                    }
                    KeyvalueType::Choices(options)
                }
                "flags" => {
                    skip_whitespace(content);
                    if content.next() != Some('=') {
                        return Err(FGDerror::InvalidSyntax(format!(
                            "Expected \"=\" after flags keyvalue (in \"{classname}\")"
                        )));
                    }
                    skip_whitespace(content);
                    if content.next() != Some('[') {
                        return Err(FGDerror::InvalidSyntax(format!(
                            "Expected \"[\" after flags keyvalue (in \"{classname}\")"
                        )));
                    }
                    let mut flags = HashMap::new();
                    loop {
                        skip_whitespace(content);
                        let value = read_text(content);
                        if value == "]" {
                            break;
                        }
                        let mut value: u32 =
                            value.parse().or(Err(FGDerror::InvalidSyntax(format!(
                                "Flags value has to be int, found \"{value}\" (in \"{classname}\")"
                            ))))?;
                        let mut i: u8 = 0;
                        while value > 1 {
                            value >>= 1;
                            i += 1;
                        }
                        skip_whitespace(content);
                        if content.next() != Some(':') {
                            return Err(FGDerror::InvalidSyntax(format!(
                                "Expected \":\" after flags value (in \"{classname}\")"
                            )));
                        }
                        skip_whitespace(content);
                        let disp = read_text(content);
                        skip_whitespace(content);
                        if content.next() != Some(':') {
                            return Err(FGDerror::InvalidSyntax(format!(
                                "Expected \":\" after flags dispname (in \"{classname}\")"
                            )));
                        }
                        skip_whitespace(content);
                        let default = read_text(content) != "0";
                        flags.insert(i, (disp, default));
                    }
                    KeyvalueType::Flags(flags)
                }
                _ => return Err(FGDerror::UnknownType(type_name)),
            };
            keyvalues.push(EntityKeyvalue {
                name,
                value_type,
                dispname,
                default,
                description,
            })
        }
    }
    Ok((
        classname,
        EntityDefinition {
            entity_type,
            properties: vec![],
            description,
            keyvalues,
            inputs,
            outputs,
        },
    ))
}

fn read_io(content: &mut Peekable<Chars>) -> Result<EntityInputOutput, FGDerror> {
    skip_whitespace(content);
    let (name, type_name) = read_name_properties(content)?;
    let mut description = None;
    skip_whitespace(content);
    if content.peek() == Some(&':') {
        // Description exists
        content.next();
        skip_whitespace(content);
        description = Some(read_text(content));
    }
    let value_type = match type_name.as_str() {
        "void" => InputOutputType::Void,
        "integer" => InputOutputType::Integer,
        "float" => InputOutputType::Float,
        "string" => InputOutputType::String,
        "bool" => InputOutputType::Boolean,
        _ => return Err(FGDerror::UnknownType(type_name)),
    };
    Ok(EntityInputOutput {
        name,
        value_type,
        description,
    })
}
