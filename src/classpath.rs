use std::{fmt::*, fs, io, path::PathBuf};

use zip::ZipArchive;

pub use std::path::MAIN_SEPARATOR as SEPARATOR;
pub trait Entry: Display {
    fn read_class(&self, class_name: &str) -> io::Result<Vec<u8>>;
}

pub fn new_entry(path: &str) -> Box<dyn Entry> {
    todo!()
}

/// 实现
struct DirEntry {
    abs_dir: PathBuf,
}

impl DirEntry {
    fn new(path: &str) -> Self {
        match fs::canonicalize(path) {
            Ok(p) => Self { abs_dir: p },
            Err(e) => panic!("path error: {}", e),
        }
    }
}

impl Entry for DirEntry {
    fn read_class(&self, class_name: &str) -> io::Result<Vec<u8>> {
        let file = self.abs_dir.join(class_name);
        fs::read(file)
    }
}

impl Display for DirEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let s = self.abs_dir.to_str().expect("path to str error");
        write!(f, "({})", s)
    }
}

use io::prelude::*;
struct ZipEntry {
    abs_dir: PathBuf,
}

impl ZipEntry {
    fn new(path: &str) -> Self {
        match fs::canonicalize(path) {
            Ok(p) => Self { abs_dir: p },
            Err(e) => panic!("path error: {}", e),
        }
    }
}

impl Entry for ZipEntry {
    fn read_class(&self, class_name: &str) -> io::Result<Vec<u8>> {
        let mut zip = ZipArchive::new(fs::File::open(self.abs_dir.as_path())?)?;
        for i in 0..zip.len() {
            match zip.by_index(i) {
                Ok(mut file) if file.name() == class_name => {
                    let mut buf = vec![];
                    file.read(&mut buf)?;
                    return Ok(buf);
                }
                Err(e) => return Err(io::Error::from(e)),
                _ => continue,
            }
        }
        todo!()
    }
}

impl Display for ZipEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let s = self.abs_dir.to_str().expect("path to str error");
        write!(f, "({})", s)
    }
}
