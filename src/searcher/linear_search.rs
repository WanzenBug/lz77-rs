use std::ops::{Index};
use std::default::{Default};
use super::{Searcher, SearchResult};
use ::buffer::{SizedBuffer};

pub struct LinearSearcher {}

impl Default for LinearSearcher {
    fn default() -> Self {
        LinearSearcher {}
    }
}

impl Searcher for LinearSearcher {
    fn find_longest_match<B>(&mut self, buf: &B, key: &[u8]) -> Option<SearchResult> where B: SizedBuffer + Index<usize, Output = u8> {
        let mut best: Option<SearchResult> = None;
        if buf.len() > (key.len() + 2) {
            for i in 0..(buf.len() - key.len() - 2) {
                let mut cur_len = 0;
                let mut j = 0;
                while j < key.len() && (i + j) < buf.len() && key[j] == buf[i + j] {
                    cur_len += 1;
                    j += 1;
                }
                best = match (best, cur_len) {
                    (Some(ref x), len) if x.length < len  => {
                        Some(SearchResult {
                            position: i,
                            length: len,
                        })
                    },
                    (None, len) if len > 1 => {
                        Some(SearchResult {
                            position: i,
                            length: len,
                        })
                    }
                    (x, _) => {
                        x
                    },
                }
            }
        }

        best
    }
}

#[cfg(test)]
mod tests {
    use super::{LinearSearcher};
    use ::buffer::{RingBuffer};
    use ::searcher::{SearchResult, Searcher};

    #[test]
    fn test_linear_search() {
        let mut buffer = RingBuffer::new(6);
        buffer.push(1);
        buffer.push(100);
        buffer.push(101);
        buffer.push(200);
        buffer.push(100);
        buffer.push(100);

        let mut searcher = LinearSearcher {};
        let key = vec![1, 100, 101];
        let res = searcher.find_longest_match(&buffer, &key);
        assert!(res.is_some());
        let search_res = res.unwrap();
        assert_eq!(search_res, SearchResult {
            position: 0,
            length: 3,
        });
    }
}
