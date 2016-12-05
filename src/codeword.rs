use std::mem;
use std::fmt;
use std::u16;
use errors::*;

extern crate byteorder;

use self::byteorder::{BigEndian, ByteOrder};

pub struct CodeWord {
    inner: u16,
    len_field_size: u8,
}

impl CodeWord {
    pub fn new(length_field_size: u8) -> Result<Self> {
        let max_len = mem::size_of::<u16>() * 8;
        if length_field_size as usize <= 0 && length_field_size as usize >= max_len {
            Err("CodeWord needs length_field has invalid size".into())
        } else {
            Ok(CodeWord {
                inner: 0,
                len_field_size: length_field_size,
            })
        }
    }

    pub fn new_with_data(length_field_size: u8, dist: u16, length: u16) -> Result<Self> {
        let mut ret = Self::new(length_field_size)?;
        ret.set_distance(dist)?;
        ret.set_length(length)?;
        Ok(ret)
    }

    pub fn get_distance(&self) -> u16 {
        self.inner >> self.len_field_size
    }

    pub fn get_length(&self) -> u16 {
        let mask = (1u16 << self.len_field_size) - 1;
        self.inner & mask
    }

    pub fn set_distance(&mut self, dist: u16) -> Result<()> {
        if (dist << self.len_field_size) >> self.len_field_size == dist {
            let l = self.get_length();
            self.inner = (dist << self.len_field_size) ^ l;
            Ok(())
        } else {
            Err("Distance field to long for CodeWord".into())
        }
    }

    pub fn set_length(&mut self, length: u16) -> Result<()> {
        let mask = ((1u16) << self.len_field_size) - 1;
        if length & mask == length {
            self.inner = (self.inner & !mask) ^ length;
            Ok(())
        } else {
            Err("Length field to long for CodeWord".into())
        }
    }

    pub fn write(&self, buf: &mut [u8]) {
        BigEndian::write_u16(&mut buf[..], self.inner)
    }

    pub fn read(&mut self, buf: &[u8]) {
        self.inner = BigEndian::read_u16(&buf[..]);
    }

    pub fn as_bytes(&self) -> [u8; 2] {
        let mut res: [u8; 2] = [0, 0];
        self.write(&mut res[..]);
        res
    }
}

impl fmt::Debug for CodeWord {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({}, {})", self.get_distance(), self.get_length())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codeword_set() {
        let mut cw = CodeWord::new(8).expect("CodeWord is not valid");
        assert_eq!(cw.get_length(), 0);
        assert_eq!(cw.get_distance(), 0);
        assert!(cw.set_length(255).is_ok());
        assert!(cw.set_distance(1).is_ok());
        assert_eq!(cw.get_length(), 255);
        assert_eq!(cw.get_distance(), 1);
        assert!(cw.set_length(1).is_ok());
        assert!(cw.set_distance(255).is_ok());
        assert_eq!(cw.get_length(), 1);
        assert_eq!(cw.get_distance(), 255);
        assert!(cw.set_length(256).is_err());
        assert!(cw.set_distance(256).is_err());
    }

    #[test]
    fn test_conversion() {
        let cw = CodeWord::new_with_data(4, 2048, 15).expect("CodeWord not valid");
        let buf = cw.as_bytes();
        let mut cw_copy = CodeWord::new(4).expect("CodeWord not valid");
        cw_copy.read(&buf[..]);

        assert_eq!(cw.get_length(), cw_copy.get_length());
        assert_eq!(cw.get_distance(), cw.get_distance());
    }
}
