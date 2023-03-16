use super::*;
/// class 文件版本：先minor，后major。顺序不可修改
#[derive(Debug)]
pub struct Version {
    pub minor: U2,
    pub major: U2,
}
