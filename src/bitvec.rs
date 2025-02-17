// color-cycle - render color cycle images
// Copyright (C) 2025  Mathias Panzenböck
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[derive(Debug, Default)]
pub struct BitVec {
    len: usize,
    bits: Vec<u8>,
}

impl BitVec {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            len: 0,
            bits: Vec::with_capacity((capacity + 7) / 8),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.bits.capacity() * 8
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: bool) {
        if index >= self.len {
            panic!("index out of range: {index} >= {}", self.len);
        }

        self.set_unchecked(index, value);
    }

    fn set_unchecked(&mut self, index: usize, value: bool) {
        let byte_index = index / 8;
        let bit_index = index - byte_index * 8;
        let bits = (value as u8) << bit_index;
        let mask = !(1 << bit_index);

        let byte = &mut self.bits[byte_index];
        *byte = (*byte & mask) | bits;
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<bool> {
        if index >= self.len {
            return None;
        }
        let byte_index = index / 8;
        let bit_index = index - byte_index * 8;
        Some(((self.bits[byte_index] >> bit_index) & 1) != 0)
    }

    #[inline]
    pub fn push(&mut self, value: bool) {
        if self.len % 8 == 0 {
            self.bits.push(value as u8);
        } else {
            self.set_unchecked(self.len, value);
        }
        self.len += 1;
    }

    #[inline]
    pub fn first(&self) -> Option<bool> {
        if self.len == 0 {
            return None;
        }
        Some((self.bits[0] & 1) != 0)
    }

    #[inline]
    pub fn last(&self) -> Option<bool> {
        if self.len == 0 {
            return None;
        }
        let index = self.len - 1;
        let byte_index = index / 8;
        let bit_index = index - byte_index * 8;
        Some(((self.bits[byte_index] >> bit_index) & 1) != 0)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<bool> {
        let value = self.last();
        if value.is_some() {
            self.len -= 1;
        }
        value
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            self.len = len;
            self.bits.truncate((len + 7) / 8);
        }
    }

    #[inline]
    pub fn fill(&mut self, value: bool) {
        let byte = 0xFF * value as u8;
        self.bits.fill(byte);
    }

    #[inline]
    pub fn extend_from_bytes(&mut self, bytes: &[u8], bit_len: usize) {
        let last_byte_bits = self.len % 8;
        if last_byte_bits == 0 {
            self.bits.extend_from_slice(&bytes[..bit_len * 8]);
        } else {
            let empty_bits = 8 - last_byte_bits;
            let mask = 0xFFu8 >> empty_bits;

            for &byte in &bytes[..bit_len * 8] {
                let last = self.bits.last_mut().unwrap();
                *last = (*last & mask) | (byte << last_byte_bits);
                self.bits.push(byte >> empty_bits);
            }
        }
        self.len += bit_len;
    }

    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bits
    }

    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.bits.clone()
    }

    #[inline]
    pub fn iter(&self) -> BitVecIter {
        BitVecIter { index: 0, bitvec: self }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BitVecIter<'a> {
    index: usize,
    bitvec: &'a BitVec,
}

impl<'a> Iterator for BitVecIter<'a> {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.bitvec.get(self.index);
        if value.is_some() {
            self.index += 1;
        }
        value
    }
}
