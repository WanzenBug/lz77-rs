#![recursion_limit = "1024"]
#[macro_use] extern crate error_chain;

mod buffer;
mod codeword;
pub mod errors;
pub mod searcher;
pub mod encoder;
pub mod decoder;

pub use encoder::{Lz77Encoder};
pub use decoder::{Lz77Decoder};
pub use searcher::{LinearSearcher};

pub struct Lz77Options {
    pub window_size: u8,
}


#[cfg(test)]
mod tests {
    use encoder::{Lz77Encoder};
    use decoder::{Lz77Decoder};
    use ::{Lz77Options};
    use std::io::{copy, Write, Cursor};
    use searcher::linear_search::{LinearSearcher};

    #[test]
    fn test_write() {
        let mut buf = Vec::new();
        {
            let opts = Lz77Options {
                window_size: 12,
            };
            let mut encoder = Lz77Encoder::<_, LinearSearcher>::new(&mut buf, opts);
            assert!(write!(encoder, "aaabcabcaaaa abc abc abc aaacccdddbla b,asfdsafsafs fsadfsdfasf").is_ok());
            assert!(encoder.flush().is_ok());
        }
        {
            let opts = Lz77Options {
                window_size: 12,
            };
            let mut cursor = Cursor::new(buf);
            let mut decoder = Lz77Decoder::new(&mut cursor, opts);
            let mut output = Vec::new();
            assert!(copy(&mut decoder, &mut output).is_ok());
            let string = String::from_utf8(output).expect("No valid utf8");
            assert_eq!(string, "aaabcabcaaaa abc abc abc aaacccdddbla b,asfdsafsafs fsadfsdfasf".to_owned());
            println!("String::from_utf8(output) = {:#?}", string);
        }
    }
}