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

use crate::palette::Palette;

use super::RgbImage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedImage {
    width: u32,
    height: u32,
    data: Box<[u8]>,
    palette: Palette,
}

impl From<IndexedImage> for Box<[u8]> {
    #[inline]
    fn from(value: IndexedImage) -> Self {
        value.data
    }
}

impl IndexedImage {
    pub fn new(width: u32, height: u32, palette: Palette) -> Self {
        Self {
            width,
            height,
            data: vec![0; width as usize * height as usize].into(),
            palette,
        }
    }

    pub fn from_index(width: u32, height: u32, index: u8, palette: Palette) -> Self {
        Self {
            width,
            height,
            data: vec![index; width as usize * height as usize].into(),
            palette,
        }
    }

    pub fn from_buffer(width: u32, height: u32, image: Box<[u8]>, palette: Palette) -> Option<Self> {
        let size = width as usize * height as usize;
        if image.len() < size {
            return None;
        }

        Some(Self {
            width,
            height,
            data: if image.len() > size { image[..size].into() } else { image },
            palette,
        })
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    #[inline]
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    #[inline]
    pub fn palette_mut(&mut self) -> &mut Palette {
        &mut self.palette
    }

    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    pub fn get_index(&self, x: u32, y: u32) -> u8 {
        let offset = self.width as usize * y as usize + x as usize;
        self.data[offset]
    }

    #[inline]
    pub fn set_index(&mut self, x: u32, y: u32, index: u8) {
        let offset = self.width as usize * y as usize + x as usize;
        self.data[offset] = index;
    }

    #[inline]
    pub fn fill(&mut self, index: u8) {
        self.data.fill(index);
    }

    pub fn get_rect_data(&self, x: u32, y: u32, width: u32, height: u32) -> Box<[u8]> {
        if x >= self.width || y >= self.height {
            return Box::new([]);
        }

        let width = width.min(self.width - x);
        let height = height.min(self.height - y);
        let size = width as usize * height as usize;

        let mut data = unsafe { Box::new_uninit_slice(size).assume_init() };

        for new_y in 0..height.min(self.height) {
            let old_offset = (y + new_y) as usize * self.width as usize + x as usize;
            let new_offset = new_y as usize * width as usize;
            let copy_width = width.min(self.width);
            data[new_offset..new_offset + copy_width as usize].copy_from_slice(&self.data[old_offset..old_offset + copy_width as usize]);
        }

        data
    }

    #[inline]
    pub fn get_rect(&self, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: self.get_rect_data(x, y, width, height),
            palette: self.palette.clone(),
        }
    }

    #[inline]
    pub fn get_rect_from(&mut self, x: u32, y: u32, width: u32, height: u32, other: &IndexedImage) {
        let width = width.min(other.width - x);
        let height = height.min(other.height - y);
        self.width = width;
        self.height = height;
        self.data = other.get_rect_data(x, y, width, height);
    }

    #[inline]
    pub fn apply_with_palette(&self, image: &mut RgbImage, palette: &Palette) {
        image.draw_indexed_image_with_palette(self, palette);
    }

    #[inline]
    pub fn apply(&self, image: &mut RgbImage) {
        image.draw_indexed_image(self);
    }

    pub fn resize(&mut self, width: u32, height: u32, index: u8) {
        if width == self.width && height == self.height {
            return;
        }

        let size = width as usize * height as usize;
        let mut data: Box<[u8]> = vec![index; size].into();

        for new_y in 0..height {
            let old_offset = new_y as usize * self.width as usize;
            let new_offset = new_y as usize * width as usize;
            data[new_offset..new_offset + width as usize].copy_from_slice(&self.data[old_offset..old_offset + width as usize]);
        }

        self.width = width;
        self.height = height;
        self.data = data;
    }

    pub fn column_swap(&mut self) {
        let columns = (self.width / 8) as usize;
        for y in 0..self.height {
            let y_offset = y as usize * self.width as usize;
            for col in 0..columns {
                let index = y_offset + col * 8;
                self.data[index..index + 8].reverse();
            }

            let index = columns * 8;
            let rem = self.width as usize - index;
            if rem > 0 {
                self.data[index..index + rem].reverse();
            }
        }
    }
}

impl From<IndexedImage> for RgbImage {
    #[inline]
    fn from(value: IndexedImage) -> Self {
        RgbImage::from_indexed_image(&value)
    }
}
