use std::{env, path::Path};

use crate::keyvalue::KeyValues;

mod keyvalue;

fn main() {
    let path = &env::args().collect::<Vec<String>>()[1];
    println!("{:#?}", KeyValues::parse(Path::new(path)));
}
