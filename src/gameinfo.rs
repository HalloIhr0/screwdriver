use std::{
    ffi::OsString,
    fs,
    path::{self, Path, PathBuf},
};

use crate::{keyvalue::KeyValues, vpk::VPK};

pub struct Gameinfo {
    search_paths: Vec<SearchPathProvider>,
}

impl Gameinfo {
    pub fn parse(file: &Path) -> Option<Self> {
        let kv = KeyValues::parse(file).ok()?;

        let mut search_paths = vec![];
        for (keys, path) in kv
            .get("GameInfo")?
            .get("FileSystem")?
            .get("SearchPaths")?
            .get_all_kv_pairs()
        {
            // Just make this lowercase to be sure it doesn't break. Even the Valve Developer Community page on gameinfo uses upper- and lowercase
            let keys: Vec<String> = keys.split('+').map(|x| x.to_lowercase()).collect();
            // TF2 (and maybe other games) uses download to store assets downloaded from community servers
            // I've decided to exclude these, so that mappers dont accidentially use them (although stock hammer doesn't exclude them (This might be the first intentional difference (Why do i write bracket sentences in other bracket sentences)))
            if (keys.contains(&String::from("game")) || keys.contains(&String::from("mod")))
                && !keys.contains(&String::from("download"))
            {
                //Since the source engine was made for windows, but this might not run on windows, we must lowercase/ignore the case for every filename
                let mut path = path
                    .get_value()?
                    .to_lowercase()
                    .replace("|all_source_engine_paths|", "")
                    .replace(
                        "|gameinfo_path|",
                        &format!("{}/", file.parent()?.file_name()?.to_str()?),
                    );
                if path.ends_with(".vpk") {
                    path.replace_range((path.len() - 4).., "_dir.vpk");
                }
                let path = Path::new(&path);
                let root = file.parent()?.parent()?.canonicalize().ok()?; // if gameinfo is in GameDir/mod/gameinfo.txt, this gets the absolute path to GameDir

                for file in get_file_case_insensitive(&root, path)? {
                    if file.is_dir() {
                        println!("Dir {:?}", file);
                        search_paths.push(SearchPathProvider::Dir(file));
                    } else if file.extension() == Some(&OsString::from("vpk")) {
                        let file = file.to_str()?;
                        if !file.ends_with("_dir.vpk") {
                            eprintln!("Unsupported VPK: {}", file);
                            continue;
                        }
                        println!("VPK {:?}", &file[..(file.len() - 8)]);
                        search_paths.push(SearchPathProvider::Vpk(VPK::parse(
                            &file[..(file.len() - 8)],
                        )?)) // len("_dir.vpk") = 8
                    }
                }
            }
        }
        Some(Self { search_paths })
    }

    pub fn get_file(&self, path: &str, extension: &str) -> Option<Vec<u8>> {
        for source in &self.search_paths {
            if let Some(content) = source.get(path, extension) {
                return Some(content);
            }
        }
        None
    }
}

#[derive(Debug)]
enum SearchPathProvider {
    Dir(PathBuf),
    Vpk(VPK),
}

impl SearchPathProvider {
    pub fn get(&self, path: &str, extension: &str) -> Option<Vec<u8>> {
        match self {
            SearchPathProvider::Dir(root) => {
                let full_path =
                    get_file_case_insensitive(root, Path::new(&format!("{}.{}", path, extension)))?;
                let full_path = full_path.get(0)?;
                fs::read(full_path).ok()
            }
            SearchPathProvider::Vpk(vpk) => vpk.get(path, extension),
        }
    }
}

/// case_ignored_path should always come from canonicalize
fn get_file_case_insensitive(root_path: &Path, case_ignored_path: &Path) -> Option<Vec<PathBuf>> {
    if case_ignored_path.is_absolute() {
        todo!()
    }

    let mut all_current = vec![root_path.to_path_buf()];

    for component in case_ignored_path.components() {
        match component {
            path::Component::Normal(name) => {
                let mut new = vec![];
                for current in all_current {
                    for path in fs::read_dir(&current).ok()? {
                        let path = path.ok()?;
                        if path.file_name().to_ascii_lowercase() == name.to_ascii_lowercase()
                            || name == "*"
                        {
                            new.push(current.join(path.file_name()));
                        }
                    }
                }
                all_current = new;
            }
            _ => todo!(),
        }
    }

    Some(all_current)
}
