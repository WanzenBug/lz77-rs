use std::io;
use std::cmp;
use ::Lz77Options;
use codeword::CodeWord;
use buffer::{RingBuffer, SizedBuffer};

enum DecoderState {
    NoData,
    Data(usize),
    Drained,
}

pub struct Lz77Decoder<R>
    where R: io::Read
{
    inner: R,
    window: RingBuffer<u8>,
    state: DecoderState,
    options: Lz77Options,
}

impl<R> Lz77Decoder<R>
    where R: io::Read
{
    pub fn new(reader: R, options: Lz77Options) -> Self {
        let size: usize = (1 << options.window_size as usize) - 1;
        Lz77Decoder {
            inner: reader,
            window: RingBuffer::new(size),
            state: DecoderState::NoData,
            options: options,
        }
    }

    fn read_token(&mut self) -> io::Result<CodeWord> {
        let mut token_buf: [u8; 2] = [0; 2];
        self.inner.read_exact(&mut token_buf[..])?;
        let mut cw = CodeWord::new(16 - self.options.window_size)
            .expect("Misaligned length for codewords");
        cw.read(&token_buf[..]);
        Ok(cw)
    }
}

impl<R> io::Read for Lz77Decoder<R>
    where R: io::Read
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut pos = 0;

        while pos < buf.len() {
            match self.state {
                DecoderState::Drained => {
                    break;
                }
                DecoderState::Data(size) => {
                    let copy_size = cmp::min(buf.len() - pos, size);
                    let window_start = self.window.len() - size;
                    for i in 0..copy_size {
                        buf[pos + i] = self.window[window_start + i]
                    }
                    if size - copy_size == 0 {
                        self.state = DecoderState::NoData;
                    } else {
                        self.state = DecoderState::Data(size - copy_size);
                    }
                    pos += copy_size;
                }
                DecoderState::NoData => {
                    let token = match self.read_token() {
                        Ok(x) => x,
                        Err(e) => {
                            match e.kind() {
                                io::ErrorKind::UnexpectedEof => {
                                    self.state = DecoderState::Drained;
                                    break;
                                }
                                _ => {
                                    return Err(e.into());
                                }
                            }
                        }
                    };
                    if token.get_distance() == 0 {
                        self.window.read_to_buf(&mut self.inner, token.get_length() as usize)?;
                    } else {
                        for _ in 0..token.get_length() {
                            let idx = self.window.len() - token.get_distance() as usize;
                            let c = self.window[idx];
                            self.window.push(c);
                        }
                    }
                    let mut b: [u8; 1] = [0];
                    self.inner.read_exact(&mut b[..])?;
                    self.window.push(b[0]);
                    self.state = DecoderState::Data(token.get_length() as usize + 1);
                }
            }
        }
        Ok(pos)
    }
}
