// color-cycle - render color cycle images on the terminal
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

use std::{fmt::Display, ops::{Index, IndexMut}};

use crate::color::Rgb;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette(pub Box<[Rgb; 256]>);

impl Display for Palette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}", self[0])?;

        for color in &self.0[1..] {
            write!(f, ", {}", color)?;
        }

        "]".fmt(f)
    }
}

impl Default for Palette {
    #[inline]
    fn default() -> Self {
        Self(Box::new([Rgb::default(); 256]))
    }
}

impl Index<u8> for Palette {
    type Output = Rgb;

    #[inline]
    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for Palette {
    #[inline]
    fn index_mut(&mut self, index: u8) -> &mut Rgb {
        &mut self.0[index as usize]
    }
}

impl From<[Rgb; 256]> for Palette {
    #[inline]
    fn from(value: [Rgb; 256]) -> Self {
        Self(value.into())
    }
}

impl From<&[Rgb; 256]> for Palette {
    #[inline]
    fn from(value: &[Rgb; 256]) -> Self {
        Self(Box::new(*value))
    }
}

impl From<&[Rgb]> for Palette {
    #[inline]
    fn from(value: &[Rgb]) -> Self {
        let mut colors = Box::new([Rgb::default(); 256]);
        colors[0..value.len().min(256)].copy_from_slice(value);
        Self(colors)
    }
}

pub const LBM_CYCLE_RATE_DIVISOR: u32 = 280;

impl Palette {
    pub fn rotate_right(&mut self, low: u8, high: u8, distance: u32) {
        let slice = &mut self.0[low as usize..high as usize + 1];
        slice.rotate_right(distance as usize);
    }

    pub fn rotate_left(&mut self, low: u8, high: u8, distance: u32) {
        let slice = &mut self.0[low as usize..high as usize + 1];
        slice.rotate_left(distance as usize);
    }

    pub fn apply_cycle(&mut self, cycle: &Cycle, now: f64) {
        let low = cycle.low();
        let high = cycle.high();
        let rate = cycle.rate();
        if high > low && rate > 0 {
            let size = (high - low + 1) as f64;
            let rate = rate as f64 / LBM_CYCLE_RATE_DIVISOR as f64;
            let distance = ((rate * now) % size) as u32;
            if cycle.reverse() {
                self.rotate_left(low, high, distance);
            } else {
                self.rotate_right(low, high, distance);
            }
        }
    }

    pub fn apply_cycle_blended(&mut self, palette: &Palette, cycle: &Cycle, now: f64) {
        let low = cycle.low();
        let high = cycle.high();
        let rate = cycle.rate();
        if high > low && rate > 0 {
            let size = high as u32 - low as u32 + 1;
            let fsize = size as f64;
            let rate = rate as f64 / LBM_CYCLE_RATE_DIVISOR as f64;
            let fdistance = (rate * now) % fsize;
            let distance = fdistance as u32;
            let mid = fdistance - distance as f64;

            let src = &palette.0[low as usize..high as usize + 1];
            let dest = &mut self.0[low as usize..high as usize + 1];

            if cycle.reverse() {
                for dest_index in 0..size {
                    let src_index = dest_index + distance;
                    let src_index1 = src_index % size;
                    let src_index2 = (src_index + 1) % size;
                    dest[dest_index as usize] = crate::color::blend(src[src_index1 as usize], src[src_index2 as usize], mid);
                }
            } else {
                let inv = 1.0 - mid;
                for src_index1 in 0..size {
                    let dest_index = (src_index1 + distance) % size;
                    let src_index2 = (src_index1 + 1) % size;
                    dest[dest_index as usize] = crate::color::blend(src[src_index1 as usize], src[src_index2 as usize], inv);
                }
            }
        }
    }

    pub fn apply_cycles(&mut self, cycles: &[Cycle], now: f64) {
        for cycle in cycles {
            self.apply_cycle(cycle, now);
        }
    }

    pub fn apply_cycles_from(&mut self, palette: &Palette, cycles: &[Cycle], now: f64, blend: bool) {
        self.clone_from(palette);

        if blend {
            for cycle in cycles {
                self.apply_cycle_blended(palette, cycle, now);
            }
        } else {
            self.apply_cycles(cycles, now);
        }
    }
}

pub fn blend(p1: &Palette, p2: &Palette, mid: f64, output: &mut Palette) {
    for index in 0..256 {
        output.0[index] = crate::color::blend(p1.0[index], p2.0[index], mid);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Cycle {
    low: u8,
    high: u8,
    rate: u32,
    reverse: bool,
}

impl Cycle {
    #[inline]
    pub fn new(low: u8, high: u8, rate: u32, reverse: bool) -> Self {
        Self {
            low,
            high,
            rate,
            reverse,
        }
    }

    #[inline]
    pub fn low(&self) -> u8 {
        self.low
    }

    #[inline]
    pub fn high(&self) -> u8 {
        self.high
    }

    #[inline]
    pub fn rate(&self) -> u32 {
        self.rate
    }

    #[inline]
    pub fn reverse(&self) -> bool {
        self.reverse
    }
}
