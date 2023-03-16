use std::convert::TryInto;

pub trait BytecodeReader: std::marker::Sized {
    /// 从bytes中读取`len=size_of<Self>()`个字节并返回len对应的unsigned int
    /// 
    /// bytes将被替换为读取len个字节后的新bytes
    fn read(bytes: &mut &[u8]) -> Self;
}

pub type U1 = u8;
pub type U2 = u16;
pub type U4 = u32;
pub type U8 = u64;


impl BytecodeReader for U1 {
    fn read(bytes: &mut &[u8]) -> Self {
        let len = std::mem::size_of::<Self>();
        let (len_bytes, rest) = bytes.split_at(len);
        *bytes = rest;
        Self::from_be_bytes(len_bytes.try_into().unwrap())
    }
}

impl BytecodeReader for U2 {
    fn read(bytes: &mut &[u8]) -> Self {
        let len = std::mem::size_of::<Self>();
        let (len_bytes, rest) = bytes.split_at(len);
        *bytes = rest;
        Self::from_be_bytes(len_bytes.try_into().unwrap())
    }
}

impl BytecodeReader for U4 {
    fn read(bytes: &mut &[u8]) -> Self {
        let len = std::mem::size_of::<Self>();
        let (len_bytes, rest) = bytes.split_at(len);
        *bytes = rest;
        Self::from_be_bytes(len_bytes.try_into().unwrap())
    }
}

impl BytecodeReader for U8 {
    fn read(bytes: &mut &[u8]) -> Self {
        let len = std::mem::size_of::<Self>();
        let (len_bytes, rest) = bytes.split_at(len);
        *bytes = rest;
        Self::from_be_bytes(len_bytes.try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let bytes = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x37];
        let bytes = &mut bytes.as_slice();
        assert_eq!(202, U1::read(bytes));
        assert_eq!(254, U1::read(bytes));
    }

    #[test]
    fn basic_u2() {
        let bytes = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x37];
        let bytes = &mut bytes.as_slice();
        assert_eq!(51966, U2::read(bytes));
        assert_eq!(47806, U2::read(bytes));
    }

    #[test]
    fn basic_u4() {
        let bytes = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x37];
        let bytes = &mut bytes.as_slice();
        assert_eq!(3405691582, U4::read(bytes));
        assert_eq!(55, U4::read(bytes));
    }

    #[test]
    fn basic_u8() {
        let bytes = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x37];
        let bytes = &mut bytes.as_slice();
        // u8转换溢出
        assert_eq!([0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x37], U8::read(bytes).to_be_bytes());
    }
}