pub mod linear_search;

use std::ops::{Index};
use ::buffer::{SizedBuffer};

pub use self::linear_search::{LinearSearcher};

#[derive(PartialEq, Eq, Debug)]
pub struct SearchResult {
    pub position: usize,
    pub length: usize,
}

pub trait Searcher {
    fn find_longest_match<B>(&mut self, buf: &B, key: &[u8]) -> Option<SearchResult> where B: SizedBuffer + Index<usize, Output = u8>;
}

