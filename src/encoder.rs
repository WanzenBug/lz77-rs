use std::io;
use std::io::{Write};
use ::{Lz77Options};
use searcher::{Searcher};
use buffer::{RingBuffer, CombinedBuffer, SizedBuffer};
use codeword::{CodeWord};

enum Lz77EncoderToken {
    None,
    Match(usize, usize),
}

pub struct Lz77Encoder<W, S> where W: io::Write, S: Searcher + Default {
    out: W,
    searcher: S,
    window: RingBuffer<u8>,
    unmatched_data: Vec<u8>,
    forward_search_size: usize,
    output_buffer: Vec<u8>,
    options: Lz77Options,
}

impl<W, S> Lz77Encoder<W, S> where W: io::Write, S: Searcher + Default {
    pub fn new(output: W, options: Lz77Options) -> Self {
        let size: usize = (1 << options.window_size as usize) - 1;
        let forward_search_size = 1 << (16 - options.window_size);
        let ring_buffer = RingBuffer::new(size);
        let searcher = S::default();
        Lz77Encoder {
            out: output,
            window: ring_buffer,
            unmatched_data: Vec::with_capacity(forward_search_size + 1),
            forward_search_size: forward_search_size,
            searcher: searcher,
            output_buffer: Vec::with_capacity(forward_search_size + 1),
            options: options,
        }
    }

    fn fill_forward_buffer(&mut self, buf: &[u8]) -> Option<(usize)> {
        let mut extra_cap = self.forward_search_size + 1 - self.unmatched_data.len();
        if extra_cap > buf.len() {
            extra_cap = buf.len();
        }
        for item in &buf[0..extra_cap] {
            self.unmatched_data.push(*item)
        }
        if self.forward_search_size + 1 - self.unmatched_data.len() == 0 {
            Some(extra_cap)
        } else {
            None
        }
    }

    fn move_unmatched_to_window(&mut self, n: usize) {
        for _ in 0..n {
            self.window.push(self.unmatched_data.remove(0));
        }
    }

    fn write_output_buffer(&mut self) -> io::Result<()> {
        if self.output_buffer.len() > 0 {
            let code = CodeWord::new_with_data(16 - self.options.window_size, 0u16, self.output_buffer.len() as u16 - 1).expect("Somebody screwed up with the CodeWord size");
            self.out.write_all(&code.as_bytes()[..])?;
            self.out.write_all(&mut self.output_buffer[..])?;
            self.output_buffer.clear();
        }
        Ok(())
    }

    fn write_to_inner(&mut self, token: Lz77EncoderToken, next: u8) -> io::Result<()> {
        if self.output_buffer.len() == self.forward_search_size {
            self.write_output_buffer()?;
        }

        match token {
            Lz77EncoderToken::Match(dist, len) => {
                if self.output_buffer.len() > 0 {
                    self.write_output_buffer()?;
                }
                let code = match CodeWord::new_with_data(16 - self.options.window_size, dist as u16, len as u16) {
                    Ok(cw) => cw,
                    Err(e) => {
                        panic!("Somebody screwed up with the CodeWord size ({}, {}), Err = {}", dist, len, e);
                    }
                };
                self.out.write_all(&code.as_bytes()[..])?;
                self.out.write_all(&[next])?;
            },
            Lz77EncoderToken::None => {
                self.output_buffer.push(next);
            }
        }
        Ok(())
    }
}

impl<W, S> io::Write for Lz77Encoder<W, S> where W: io::Write, S: Searcher + Default {
    fn write(&mut self, full_buf: &[u8]) -> io::Result<usize> {
        let size = full_buf.len();
        let mut buf = full_buf;
        while let Some(n) = self.fill_forward_buffer(buf) {
            let search_result = {
                let search_buf = CombinedBuffer(&self.window, &self.unmatched_data);
                self.searcher.find_longest_match(&search_buf, &self.unmatched_data[..(self.forward_search_size - 1)])
            };

            let fw = match search_result {
                Some(res) => {
                    let dist = self.window.len() - res.position;
                    let next = self.unmatched_data[res.length];
                    self.write_to_inner(Lz77EncoderToken::Match(dist, res.length), next)?;
                    res.length + 1
                },
                None => {
                    let next = self.unmatched_data[0];
                    self.write_to_inner(Lz77EncoderToken::None, next)?;
                    1
                }
            };

            self.move_unmatched_to_window(fw);
            buf = &buf[n..];
        }
        Ok(size)
    }

    fn flush(&mut self) -> io::Result<()> {
        while self.unmatched_data.len() != 0 {
            let search_result = {
                let search_buf = CombinedBuffer(&self.window, &self.unmatched_data);
                let search_size = self.unmatched_data.len() - 1;
                self.searcher.find_longest_match(&search_buf, &self.unmatched_data[..(search_size)])
            };

            let fw = match search_result {
                Some(res) => {
                    let dist = self.window.len() - res.position;
                    let next = self.unmatched_data[res.length];
                    self.write_to_inner(Lz77EncoderToken::Match(dist, res.length), next)?;
                    res.length + 1
                },
                None => {
                    let next = self.unmatched_data[0];
                    self.write_to_inner(Lz77EncoderToken::None, next)?;
                    1
                }
            };
            self.move_unmatched_to_window(fw);
        }
        self.write_output_buffer()?;
        self.out.flush()
    }
}

impl<W, S> Drop for Lz77Encoder<W,S> where W: io::Write, S: Searcher + Default {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
