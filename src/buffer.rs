use std::ops::{Index, IndexMut};
use std::fmt;
use std::io;
use std::cmp;

pub trait SizedBuffer {
    fn len(&self) -> usize;
}

impl<T> SizedBuffer for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

pub struct RingBuffer<T> {
    cur_start: usize,
    buf: Vec<T>,
}

impl fmt::Debug for RingBuffer<u8> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.len() {
            write!(fmt, "{:?}", self[i] as char)?;
        }
        Ok(())
    }
}


impl<T> SizedBuffer for RingBuffer<T> {
    fn len(&self) -> usize {
        self.buf.len()
    }
}


impl<T> Index<usize> for RingBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let n = self.buf.len();
        &self.buf[(self.cur_start + index) % n]
    }
}

impl<T> IndexMut<usize> for RingBuffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let n = self.buf.len();
        &mut self.buf[(self.cur_start + index) % n]
    }
}

impl<T> RingBuffer<T> {
    pub fn new(size: usize) -> RingBuffer<T> {
        RingBuffer::from_vec(Vec::with_capacity(size))
    }

    pub fn from_vec(vec: Vec<T>) -> RingBuffer<T> {
        RingBuffer {
            cur_start: 0,
            buf: vec,
        }
    }

    pub fn push(&mut self, item: T) {
        let mut i = self.buf.len();
        if i == self.buf.capacity() {
            i = self.cur_start;
            self.cur_start = (self.cur_start + 1) % self.buf.capacity();
            self.buf[i] = item;
        } else {
            self.buf.push(item);
        }
    }
}

impl RingBuffer<u8> {
    pub fn read_to_buf<R>(&mut self, reader: &mut R, size: usize) -> io::Result<()> where R: io::Read {
        let mut pos = 0;

        while pos < size {
            if self.buf.len() == self.buf.capacity() {
                let to_copy = cmp::min(self.buf.capacity() - self.cur_start, size - pos);
                reader.read_exact(&mut self.buf[self.cur_start..(self.cur_start + to_copy)])?;
                self.cur_start = (self.cur_start + to_copy) % self.buf.capacity();
                pos += to_copy;
            } else {
                let to_copy = cmp::min(self.buf.capacity() - self.buf.len(), size - pos);
                let buf_end = self.buf.len();
                unsafe { self.buf.set_len(buf_end + to_copy) };
                reader.read_exact(&mut self.buf[buf_end..(buf_end + to_copy)])?;
                pos += to_copy;
            }
        }
        Ok(())
    }
}

pub struct CombinedBuffer<'a, 'b, A, B, T>(pub &'a A, pub &'b B) where A: 'a + SizedBuffer + Index<usize, Output = T>, B: 'b + SizedBuffer + Index<usize, Output = T>;


impl<'a, 'b, A, B, T> Index<usize> for CombinedBuffer<'a, 'b, A, B, T> where A: 'a + SizedBuffer + Index<usize, Output = T>, B: 'b + SizedBuffer + Index<usize, Output = T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.0.len() {
            &self.0[index]
        } else {
            &self.1[index - self.0.len()]
        }
    }
}

impl<'a, 'b, A, B, T> SizedBuffer for CombinedBuffer<'a, 'b, A, B, T> where A: 'a + SizedBuffer + Index<usize, Output = T>, B: 'b + SizedBuffer + Index<usize, Output = T> {
    fn len(&self) -> usize {
        self.0.len() + self.1.len()
    }
}

impl<'a, 'b, A, B, T> fmt::Debug for CombinedBuffer<'a, 'b, A, B, T> where A: 'a + SizedBuffer + Index<usize, Output = T> + fmt::Debug, B: 'b + SizedBuffer + Index<usize, Output = T> + fmt::Debug {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(fmt)?;
        self.1.fmt(fmt)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor};

    #[test]
    fn test_ring_buffer() {
        let mut ring = RingBuffer::new(3);
        assert_eq!(ring.len(), 0);
        ring.push(1);
        assert_eq!(ring[0], 1);
        ring.push(2);
        assert_eq!(ring[1], 2);
        ring.push(3);
        assert_eq!(ring[2], 3);
        ring.push(4);
        assert_eq!(ring[0], 2);
        assert_eq!(ring[1], 3);
        assert_eq!(ring[2], 4);
    }

    #[test]
    fn test_combined_buffer() {
        let a_buff = RingBuffer::from_vec(vec![1, 2, 3]);
        let b_buff = RingBuffer::from_vec(vec![0, 255]);
        let combined = CombinedBuffer(&a_buff, &b_buff);
        assert_eq!(combined.len(), 5);
        assert_eq!(combined[0], 1);
        assert_eq!(combined[1], 2);
        assert_eq!(combined[2], 3);
        assert_eq!(combined[3], 0);
        assert_eq!(combined[4], 255);
    }

    #[test]
    fn test_ring_buffer_read() {
        let mut ring = RingBuffer::new(4);
        let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut cursor = Cursor::new(&vec);
        assert!(ring.read_to_buf(&mut cursor, 10).is_ok());
        assert_eq!(ring[0], 7);
        assert_eq!(ring[1], 8);
        assert_eq!(ring[2], 9);
        assert_eq!(ring[3], 10);

        let mut cursor2 = Cursor::new(&vec[..4]);
        assert!(ring.read_to_buf(&mut cursor2, 4).is_ok());
        assert_eq!(ring[0], 1);
        assert_eq!(ring[1], 2);
        assert_eq!(ring[2], 3);
        assert_eq!(ring[3], 4);
    }
}