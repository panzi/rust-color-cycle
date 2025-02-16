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

use crate::palette::{Cycle, Palette};

use super::IndexedImage;

#[derive(Debug, Clone)]
pub struct CycleImage {
    filename: Option<String>,
    indexed_image: IndexedImage,
    cycles: Box<[Cycle]>,
}

impl CycleImage {
    #[inline]
    pub fn new(filename: Option<String>, indexed_image: IndexedImage, cycles: Box<[Cycle]>) -> Self {
        Self {
            filename,
            indexed_image,
            cycles,
        }
    }

    #[inline]
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    #[inline]
    pub fn indexed_image(&self) -> &IndexedImage {
        &self.indexed_image
    }

    #[inline]
    pub fn cycles(&self) -> &[Cycle] {
        &self.cycles
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.indexed_image.width()
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.indexed_image.height()
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    #[inline]
    pub fn palette(&self) -> &Palette {
        &self.indexed_image.palette()
    }

    #[inline]
    pub fn palette_mut(&mut self) -> &mut Palette {
        self.indexed_image.palette_mut()
    }

    #[inline]
    pub fn get_index(&self, x: u32, y: u32) -> u8 {
        self.indexed_image().get_index(x, y)
    }

    #[inline]
    pub fn get_rect(&self, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            filename: None,
            indexed_image: self.indexed_image.get_rect(x, y, width, height),
            cycles: self.cycles.clone(),
        }
    }

    #[inline]
    pub fn get_rect_from(&mut self, x: u32, y: u32, width: u32, height: u32, other: &CycleImage) {
        self.indexed_image.get_rect_from(x, y, width, height, &other.indexed_image);
    }

    #[inline]
    pub fn resize(&mut self, width: u32, height: u32, index: u8) {
        self.indexed_image.resize(width, height, index);
    }
}
