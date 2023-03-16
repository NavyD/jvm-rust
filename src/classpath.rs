use io::prelude::*;
use std::{env, fmt, fs, io, path};
use zip::ZipArchive;

pub use std::path::MAIN_SEPARATOR as SEPARATOR;

pub trait Entry: fmt::Display {
    /// 从classpath中找指定的class_name并返回class bytes
    ///
    /// class_name是路径格式：class_name=java.lang.Object
    fn read_class(&self, class_name: &str) -> io::Result<Vec<u8>>;
}

/// 将class名称转为path形式：class_name=java.lang.Object =>java/lang/Object.class
fn class_to_path(class_name: &str) -> String {
    class_name.replace(".", &SEPARATOR.to_string()) + ".class"
}

pub fn new_entry(path: &str) -> Box<dyn Entry> {
    let path = path.trim();
    if path.contains(&SEPARATOR.to_string()) {
        return Box::new(CompositeEntry::new(path));
    }
    if let Some(_) = path.strip_suffix("*") {
        return Box::new(CompositeEntry::with_wildcard_path(path));
    }
    return Box::new(DirEntry::new(path));
}

/// 实现
struct DirEntry {
    abs_dir: path::PathBuf,
}

impl DirEntry {
    /// 用当前目录path创建一个entry
    ///
    /// # panic
    ///
    /// 如果path不存在或不是目录
    fn new(path: &str) -> Self {
        match fs::canonicalize(path) {
            Ok(p) => {
                if p.is_dir() {
                    Self { abs_dir: p }
                } else {
                    panic!("{} is a file", p.to_str().unwrap())
                }
            }
            Err(e) => panic!("path error: {}", e),
        }
    }
}

impl Entry for DirEntry {
    fn read_class(&self, class_name: &str) -> io::Result<Vec<u8>> {
        let class_path = class_to_path(class_name);
        let file = self.abs_dir.join(class_path);
        fs::read(file)
    }
}

impl fmt::Display for DirEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.abs_dir.to_str().expect("path to str error");
        write!(f, "({})", s)
    }
}

#[cfg(test)]
mod dir_entry_tests {
    use super::*;

    #[test]
    fn basics() {
        let entry = DirEntry::new("resources/test/classpath/user");
        let data = entry.read_class("xyz.navyd.ClassFileTest");
        assert!(data.is_ok());
        // HelloWorld.class bytes length
        assert_eq!(770, data.unwrap().len());
    }

    #[test]
    #[should_panic]
    fn new_panic_with_path_not_found() {
        DirEntry::new("no_path_");
    }

    #[test]
    #[should_panic]
    fn new_panic_with_file() {
        DirEntry::new("resources/test/classpath/user/HelloWorld.jar");
    }

    #[test]
    fn read_class_not_found() {
        let entry = DirEntry::new("resources/test/classpath/user");
        assert!(entry.read_class("hello").is_err());
    }
}

struct ZipEntry {
    abs_dir: path::PathBuf,
}

impl ZipEntry {
    /// 用当前zip或jar文件创建一个entry
    ///
    /// # panic
    ///
    /// 如果path不是一个文件
    fn new(path: &str) -> Self {
        match fs::canonicalize(path) {
            Ok(path) if path.is_file() => Self { abs_dir: path },
            _ => panic!("{} is not a file", path),
        }
    }
}

impl Entry for ZipEntry {
    fn read_class(&self, class_name: &str) -> io::Result<Vec<u8>> {
        let class_path = class_to_path(class_name);
        // zip file
        let mut zip = ZipArchive::new(fs::File::open(self.abs_dir.as_path())?)?;
        for i in 0..zip.len() {
            match zip.by_index(i) {
                // the file name is path+filename: xyz/navyd/HelloWorld.class
                Ok(mut file) if file.name() == class_path => {
                    let mut buf = vec![0; file.size() as usize];
                    file.read(&mut buf)?;
                    return Ok(buf);
                }
                Err(e) => return Err(io::Error::from(e)),
                _ => continue,
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "not found class: ".to_string()
                + class_name
                + " in path: "
                + self.abs_dir.to_str().unwrap(),
        ))
    }
}

impl fmt::Display for ZipEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.abs_dir.to_str().expect("path to str error"))
    }
}

#[cfg(test)]
mod zip_entry_tests {
    use super::*;

    #[test]
    fn basics() {
        let entry = ZipEntry::new("resources/test/classpath/user/ClassFileTest.jar");
        let data = entry.read_class("xyz.navyd.ClassFileTest");
        assert!(data.is_ok());
        // HelloWorld.class length
        assert_eq!(770, data.unwrap().len());
    }

    #[should_panic]
    #[test]
    fn new_panic_with_dir() {
        ZipEntry::new("resources/test/classpath/user");
    }

    #[test]
    fn read_class_not_found_error() {
        let entry = ZipEntry::new("resources/test/classpath/user/ClassFileTest.jar");
        let data = entry.read_class("a.b.C");
        assert!(data.is_err());
        assert_eq!(io::ErrorKind::NotFound, data.unwrap_err().kind());
    }

    #[test]
    fn read_class_non_zip_file_error() {
        let entry = ZipEntry::new("resources/test/classpath/user/xyz/navyd/ClassFileTest.class");
        let data = entry.read_class("HelloWorld");
        assert!(data.is_err());
    }
}

// todo
struct CompositeEntry {
    entrys: Vec<Box<dyn Entry>>,
}

impl CompositeEntry {
    fn new(path_list: &str) -> Self {
        let mut entrys = vec![];
        let paths: Vec<&str> = path_list.split(SEPARATOR).collect();
        for path in paths {
            entrys.push(new_entry(path));
        }
        Self { entrys }
    }

    fn with_wildcard_path(path: &str) -> Self {
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
        let boot_classpath = new_entry(jre_lib_path.to_str().unwrap());
        // jre/lib/ext/*
        let jre_ext_path = jre_dir.join("lib").join("ext");
        let ext_classpath = new_entry(jre_ext_path.to_str().unwrap());
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
        new_entry(path)
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

#[cfg(test)]
mod classpath_tests {
    #[test]
    fn basics() {}
}
