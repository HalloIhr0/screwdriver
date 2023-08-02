use std::{env, path::Path};

use screwdriver::keyvalue::KeyValues;

fn main() {
    let args = &env::args().collect::<Vec<String>>();
    let kv = KeyValues::parse(Path::new(&args[1])).unwrap();
    test_get(&kv).unwrap();
    kv.write(Path::new(&args[2])).unwrap();
}

fn test_get(kv: &KeyValues) -> Option<()> {
    println!(
        "{}",
        kv.get("versioninfo")?
            .get("mapversion")?
            .get_value()?
    );
    println!(
        "{:#?}",
        kv.get("world")?
            .get_all("solid")
    );
    Some(())
}
