use std::{fs, env};

use crate::keyvalue::KeyValues;

mod keyvalue;

fn main() {
    let path = &env::args().collect::<Vec<String>>()[1];
    let content = fs::read_to_string(path).unwrap();
    println!("{:#?}", KeyValues::parse(&content));
}
