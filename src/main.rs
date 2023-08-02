use std::{env, path::Path};

use screwdriver::keyvalue::KeyValues;

fn main() {
    let args = &env::args().collect::<Vec<String>>();
    let kv = KeyValues::parse(Path::new(&args[1])).unwrap();
    // println!("{:#?}", kv);
    kv.write(Path::new(&args[2])).unwrap();
}
