#[feature(globs)];
use std::cast;
use std::vec;
use std::io;
use std::io::{BufReader, IoResult};

/* automatically generated by rust-bindgen */

use std::libc::*;
pub type iconv_t = *mut c_void;
#[link(name = "iconv")]
extern "C" {
    fn iconv_open(__tocode: *c_schar, __fromcode: *c_schar) -> iconv_t;
    fn iconv(__cd: iconv_t, __inbuf: *mut *mut c_schar,
                 __inbytesleft: *mut size_t, __outbuf: *mut *mut c_schar,
                 __outbytesleft: *mut size_t) -> size_t;
    fn iconv_close(__cd: iconv_t) -> c_int;
}

/* automatically generated ends */

// pub struct IconvEncoder {
//     i: iconv_t
// }

// impl Iconv {
//     pub fn new(to: &str, from: &str) -> Iconv {
//         let hd = unsafe { iconv_open(

pub struct IconvReader<R> {
    priv inner: R,
    priv h: iconv_t,
    priv eof: bool,
}

impl<R:Reader> IconvReader<R> {
    pub fn new(r: R, from: &str, to: &str) -> IconvReader<R> {
        let handle = from.with_c_str(|from_encoding| {
                to.with_c_str(|to_encoding| unsafe {
                        iconv_open(to_encoding, from_encoding)
                    })
            });
        IconvReader { inner: r, h: handle , eof: false }
    }
}

impl<R:Reader> Reader for IconvReader<R> {
     fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        let out_size = buf.len();
        let out_bytes_left = buf.len() as size_t;
        let mut tbuf = vec::from_elem(out_size / 3, 0u8);
        match self.inner.read(tbuf) {
            Ok(in_size) if in_size > 0 => {
                let in_bytes_left = in_size as size_t;
                println!("in_size => {}", in_size);
                let ret = unsafe { iconv(
                        self.h,
                        cast::transmute(& &tbuf[0]), cast::transmute(&in_bytes_left),
                        cast::transmute(& &buf[0]), cast::transmute(&out_bytes_left))
                };
                println!("ret => {:?}", ret);
                if in_bytes_left > 0 {
                    fail!("left in_bytes => {:?}", in_bytes_left);
                }
                Ok(out_size - out_bytes_left as uint)
            }
            Ok(0)          => Ok(0),
            Err(e)         => {
                if !self.eof && e.kind == io::EndOfFile {
                    unsafe { iconv_close(self.h) };
                    self.eof = true;
                }
                Err(e)
            },
            _ => unreachable!()
        }
    }
}

//
pub trait IconvEncodable {
    fn encode_with_encoding(&self, encoding: &str) -> Option<~[u8]>;
}

impl<'a> IconvEncodable for &'a str {
    fn encode_with_encoding(&self, encoding: &str) -> Option<~[u8]> {
        let mut reader = IconvReader::new(BufReader::new(self.as_bytes()),
                                          "UTF-8", encoding);
        match reader.read_to_end() {
            Ok(content) => Some(content),
            Err(_)      => None,
        }
    }
}

impl<'a> IconvEncodable for &'a [u8] {
    // assume UTF-8 bytes
    fn encode_with_encoding(&self, encoding: &str) -> Option<~[u8]> {
        let mut reader = IconvReader::new(BufReader::new(*self),
                                          "UTF-8", encoding);
        match reader.read_to_end() {
            Ok(content) => Some(content),
            Err(_)      => None,
        }
    }
}

pub trait IconvDecodable {
    fn decode_with_encoding(&self, encoding: &str) -> Option<~str>;
}

impl<'a> IconvDecodable for &'a [u8] {
    fn decode_with_encoding(&self, encoding: &str) -> Option<~str> {
        let mut reader = IconvReader::new(BufReader::new(*self),
                                          encoding, "UTF-8");
        match reader.read_to_str() {
            Ok(content) => Some(content),
            Err(_)      => None,
        }
    }
}

#[test]
fn test_encoder() {
    let a = "哈哈";
    assert_eq!(a.encode_with_encoding("gbk").unwrap(), ~[0xb9, 0xfe, 0xb9, 0xfe]);
    let b = ~[0xe5, 0x93, 0x88, 0xe5, 0x93, 0x88];
    assert_eq!(b.encode_with_encoding("gbk").unwrap(), ~[0xb9, 0xfe, 0xb9, 0xfe]);
}

#[test]
fn test_decoder() {
    let b = ~[0xb9, 0xfe, 0xb9, 0xfe];
    assert_eq!(b.decode_with_encoding("gbk").unwrap(), ~"哈哈");
}
