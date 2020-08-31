use std::{fmt, fs, io, path::PathBuf};

use zip::ZipArchive;

pub use std::path::MAIN_SEPARATOR as SEPARATOR;
pub trait Entry: fmt::Display {
    fn read_class(&self, class_name: &str) -> io::Result<Vec<u8>>;
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

impl fmt::Display for DirEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl fmt::Display for ZipEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.abs_dir.to_str().expect("path to str error");
        write!(f, "({})", s)
    }
}

struct CompositeEntry {
    entrys: Vec<Box<dyn Entry>>,
}

pub fn new_entry(path: &str) -> Result<Box<dyn Entry>, String> {
    todo!()
}

impl CompositeEntry {
    fn new(path_list: &str) -> Self {
        let mut entrys = vec![];
        let paths: Vec<&str> = path_list.split(SEPARATOR).collect();
        for path in paths {
            if let Err(e) = new_entry(path).map(|entry|entrys.push(entry)) {
                println!("{}", e);
            }
        }
        Self { entrys }
    }

    fn with_wildcard_path(path: &str) -> io::Result<Self> {
        let mut entrys = vec![];
        // 去除 *
        let base_dir = &path.trim()[..path.len() - 1];
        fs::read_dir(base_dir).map(|dir| for entry in dir {
            if let Err(e) = entry.map(|e| {
                e.file_name().to_str().and_then(|name| if name.strip_suffix(".jar").is_some() {
                    Some(entrys.push(ZipEntry::new(path)))
                } else {
                    None
                })
            }) {
                println!("{}", e);
            }
        }).expect_err("path read error");
        todo!()
    }
}

impl Entry for CompositeEntry {
    fn read_class(&self, class_name: &str) -> io::Result<Vec<u8>> {
        for entry in &self.entrys {
            if let Ok(val) = entry.read_class(class_name) {
                return Ok(val);
            } else {
                println!("{} in {} not found.", class_name, entry);
            }
        }
        Err(io::Error::from(io::ErrorKind::NotFound))
    }
}

impl fmt::Display for CompositeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in &self.entrys {
            write!(f, "{}{}", entry, SEPARATOR)?;
        }
        Ok(())
    }
}
