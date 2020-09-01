use std::{env, fmt, fs, io, path};

use zip::ZipArchive;

pub use std::path::MAIN_SEPARATOR as SEPARATOR;
pub trait Entry: fmt::Display {
    fn read_class(&self, class_name: &str) -> io::Result<Vec<u8>>;
}

/// 实现
struct DirEntry {
    abs_dir: path::PathBuf,
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
    abs_dir: path::PathBuf,
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

pub fn new_entry<P: AsRef<path::Path>>(path: P) -> Result<Box<dyn Entry>, String> {
    todo!()
}

impl CompositeEntry {
    fn new(path_list: &str) -> Self {
        let mut entrys = vec![];
        let paths: Vec<&str> = path_list.split(SEPARATOR).collect();
        for path in paths {
            if let Err(e) = new_entry(path).map(|entry| entrys.push(entry)) {
                println!("{}", e);
            }
        }
        Self { entrys }
    }

    fn with_wildcard_path(path: &str) -> io::Result<Self> {
        let mut entrys = vec![];
        // 去除 *
        let base_dir = &path.trim()[..path.len() - 1];
        fs::read_dir(base_dir)
            .map(|dir| {
                for entry in dir {
                    if let Err(e) = entry.map(|e| {
                        e.file_name().to_str().and_then(|name| {
                            if name.strip_suffix(".jar").is_some() {
                                Some(entrys.push(ZipEntry::new(path)))
                            } else {
                                None
                            }
                        })
                    }) {
                        println!("{}", e);
                    }
                }
            })
            .expect_err("path read error");
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

pub struct Classpath {
    pub boot_classpath: Box<dyn Entry>,
    pub ext_classpath: Box<dyn Entry>,
    pub user_classpath: Box<dyn Entry>,
}

impl Classpath {
    /// 从option: jre, cp中构建Classpath对象
    /// 
    /// # panic
    /// 
    /// 如果jre, cp没有可用路径
    pub fn new(jre_option: &str, cp_option: &str) -> Self {
        let (bc, ec) = Self::parse_boot_and_ext_classpath(jre_option);
        Self {
            user_classpath: Self::parse_user_classpath(cp_option),
            boot_classpath: bc,
            ext_classpath: ec,
        }
    }

    /// 从jre option中创建出boot classpath, ext classpath entry
    /// 
    /// # panic
    /// 
    /// 如果任一个classpath entry创建失败
    fn parse_boot_and_ext_classpath(jre_option: &str) -> (Box<dyn Entry>, Box<dyn Entry>) {
        let jre_dir = Classpath::get_jre_dir(jre_option);
        // jre/lib/*
        let jre_lib_path = jre_dir.join("lib").join("*");
        let boot_classpath = match new_entry(jre_lib_path) {
            Ok(entry) => entry,
            Err(e) => panic!("{}", e),
        };
        // jre/lib/ext/*
        let jre_ext_path = jre_dir.join("lib").join("ext");
        let ext_classpath = match new_entry(jre_ext_path) {
            Ok(entry) => entry,
            Err(e) => panic!("{}", e),
        };
        (boot_classpath, ext_classpath)
    }

    /// 从cp option中创建出一个entry
    ///
    /// 如果cp option为空，则默认.当前路径
    ///
    /// # panic
    ///
    /// cp_option无法创建entry时
    fn parse_user_classpath(cp_option: &str) -> Box<dyn Entry> {
        let path = if cp_option.trim().is_empty() {
            "."
        } else {
            cp_option
        };
        match new_entry(path) {
            Ok(entry) => entry,
            Err(e) => panic!("{}", e),
        }
    }

    /// 从jre option中返回可用的jre path。
    ///
    /// 如果jre option不可用，则读取当前路径下的./jre。
    ///
    /// 如果./jre不可用则返回系统JAVA_HOME变量jre path
    ///
    /// # panic
    ///
    /// 当上面3个jre path都不可用时
    fn get_jre_dir(jre_option: &str) -> path::PathBuf {
        // jre option
        let path = path::Path::new(jre_option);
        if path.exists() {
            return path.to_path_buf();
        }
        // default ./jre
        let path = path::Path::new("./jre");
        if path.exists() {
            return path.to_path_buf();
        }
        // JAVA_HOME
        if let Ok(path) = env::var("JAVA_HOME") {
            let path = path::Path::new(&path).join("jre");
            if path.exists() {
                return path;
            }
        }
        panic!("can not find jre folder")
    }
}

impl Entry for Classpath {
    /// 依次从boot,ext,user classpath中寻找class。
    /// 
    /// 如果未找到则返回user classpath中的Err
    fn read_class(&self, class_name: &str) -> io::Result<Vec<u8>> {
        let class_name = &(class_name.to_string() + ".class");
        if let Ok(data) = self.boot_classpath.read_class(class_name) {
            return Ok(data);
        }
        if let Ok(data) = self.ext_classpath.read_class(class_name) {
            return Ok(data);
        }
        self.user_classpath.read_class(class_name)
    }
}

impl fmt::Display for Classpath {
    /// 返回user classpath类路径表示
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.user_classpath.fmt(f)
    }
}