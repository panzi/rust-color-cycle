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

use super::CycleImage;

// render files from http://www.effectgames.com/demos/worlds/

#[derive(Debug, Clone)]
pub struct LivingWorld {
    name: Option<String>,
    base: CycleImage,
    palettes: Box<[CycleImage]>,
    timeline: Box<[TimedEvent]>,
}

impl LivingWorld {
    #[inline]
    pub fn new(name: Option<String>, base: CycleImage, palettes: Box<[CycleImage]>, timeline: Box<[TimedEvent]>) -> Self {
        Self { name, base, palettes, timeline }
    }

    #[inline]
    pub fn only_base(base: CycleImage) -> Self {
        Self {
            name: None,
            base,
            palettes: Box::new([]),
            timeline: Box::new([]),
        }
    }

    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[inline]
    pub fn base(&self) -> &CycleImage {
        &self.base
    }

    #[inline]
    pub fn palettes(&self) -> &[CycleImage] {
        &self.palettes
    }

    #[inline]
    pub fn timeline(&self) -> &[TimedEvent] {
        &self.timeline
    }

    #[inline]
    pub fn into_base(self) -> CycleImage {
        self.base
    }
}

impl From<CycleImage> for LivingWorld {
    #[inline]
    fn from(value: CycleImage) -> Self {
        LivingWorld::new(
            value.filename().map(|value| value.to_owned()),
            value,
            Box::new([]),
            Box::new([]),
        )
    }
}

impl From<LivingWorld> for CycleImage {
    #[inline]
    fn from(value: LivingWorld) -> Self {
        value.into_base()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimedEvent {
    /// time of day in seconds since midnight
    time_of_day: u32,
    palette_index: usize,
}

impl TimedEvent {
    #[inline]
    pub fn new(time_of_day: u32, palette_index: usize) -> Self {
        Self { time_of_day, palette_index }
    }

    #[inline]
    pub fn time_of_day(&self) -> u32 {
        self.time_of_day
    }

    #[inline]
    pub fn palette_index(&self) -> usize {
        self.palette_index
    }
}
