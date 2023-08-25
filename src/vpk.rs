use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
};

#[derive(Debug)]
struct FileInfo {
    archive_index: u16,
    offset: u32,
    lenght: u32,
    preload_data: Vec<u8>,
}

#[derive(Debug)]
pub struct VPK {
    base_path: String,
    tree_size: u32,
    file_info: HashMap<(String, String), FileInfo>,
}

impl VPK {
    pub fn parse(base_path: &str) -> Option<Self> {
        let mut file = File::open(base_path.to_string() + "_dir.vpk").ok()?;

        assert_eq!(read_uint(&mut file)?, 0x55AA1234); // Signature
        let tree_size;
        match read_uint(&mut file)? {
            1 => {
                tree_size = read_uint(&mut file)?;
            }
            2 => {
                tree_size = read_uint(&mut file)?;
                read_uint(&mut file)?; // FileDataSectionSize
                read_uint(&mut file)?; // ArchiveMD5SectionSize
                read_uint(&mut file)?; // OtherMD5SectionSize
                read_uint(&mut file)?; // SignatureSectionSize
            }
            x => {
                eprintln!("Invalid VPK version: {x}");
                return None;
            }
        }
        let mut files = HashMap::new();
        loop {
            let extension = read_string(&mut file)?;
            if extension.is_empty() {
                break;
            }
            loop {
                let path = read_string(&mut file)?;
                if path.is_empty() {
                    break;
                }
                loop {
                    let filename = read_string(&mut file)?;
                    if filename.is_empty() {
                        break;
                    }
                    read_uint(&mut file); // CRC
                    let preload_len = read_ushort(&mut file)?;
                    let archive_index = read_ushort(&mut file)?;
                    let offset = read_uint(&mut file)?;
                    let lenght = read_uint(&mut file)?;
                    assert_eq!(read_ushort(&mut file)?, 0xFFFF); // Terminator
                    let mut preload_data = vec![0u8; preload_len as usize];
                    file.read_exact(&mut preload_data).ok()?;
                    files.insert(
                        (
                            format!("{}/{}", path, filename).replace('\\', "/"),
                            extension.clone(),
                        ), // Replace just to make sure bugs doesn't happen
                        FileInfo {
                            archive_index,
                            offset,
                            lenght,
                            preload_data,
                        },
                    );
                }
            }
        }

        Some(Self {
            base_path: base_path.to_string(),
            tree_size,
            file_info: files,
        })
    }

    pub fn get(&self, path: &str, extension: &str) -> Option<Vec<u8>> {
        let info = self
            .file_info
            .get(&(path.replace('\\', "/"), String::from(extension)))?;
        let mut result = info.preload_data.clone();
        if info.archive_index == 0x7FFF {
            // Data follows tree
            let mut file = File::open(self.base_path.to_string() + "_dir.vpk").ok()?;
            file.seek(SeekFrom::Current(28)).ok()?; // Header size
            file.seek(SeekFrom::Current(self.tree_size as i64)).ok()?;
            file.seek(SeekFrom::Current(info.offset as i64)).ok()?;
            let mut data = vec![0u8; info.lenght as usize];
            file.read_exact(&mut data).ok()?;
            result.append(&mut data);
        } else {
            let mut file =
                File::open(format!("{}_{:03}.vpk", self.base_path, info.archive_index)).ok()?;
            file.seek(SeekFrom::Current(info.offset as i64)).ok()?;
            let mut data = vec![0u8; info.lenght as usize];
            file.read_exact(&mut data).ok()?;
            result.append(&mut data);
        }
        Some(result)
    }
}

// From https://developer.valvesoftware.com/wiki/VPK_(file_format)#Tree
fn read_string(file: &mut File) -> Option<String> {
    let mut result = vec![];
    loop {
        let mut buf = [0u8];
        file.read_exact(&mut buf).ok()?;
        let char = buf[0];
        if char == 0 {
            return Some(String::from_utf8_lossy(&result).to_string());
        }
        result.push(char);
    }
}

fn read_uint(file: &mut File) -> Option<u32> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf).ok()?;
    Some(
        (buf[0] as u32)
            | ((buf[1] as u32) << 8)
            | ((buf[2] as u32) << 16)
            | ((buf[3] as u32) << 24),
    )
}

fn read_ushort(file: &mut File) -> Option<u16> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf).ok()?;
    Some((buf[0] as u16) | ((buf[1] as u16) << 8))
}
