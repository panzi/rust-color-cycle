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

use crate::{color::Rgb, image::{living_world::TimedEvent, CycleImage, IndexedImage, LivingWorld}, palette::{Cycle, Palette}};

use std::{collections::HashMap, convert::TryInto};
use serde::{de::{Error, IgnoredAny, Visitor}, Deserializer, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FormatInfo {
    pub version: u32,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MagratheaWorldPaletteInfo {
    pub id: u32,
    pub name: String,
    pub colors: Palette,
    pub cycles: Box<[Cycle]>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MagratheaWorldData {
    pub name: String,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "paletteInfos")]
    pub palette_infos: Vec<MagratheaWorldPaletteInfo>,
    pub pixels: Box<[u8]>,

    // TODO: pub events: Vec<MagratheaWorldEvent>,
    // TODO: pub modes: Vec<MagratheaWorldMode>,
}

struct CycleImageVisitor;

impl<'de> Visitor<'de> for CycleImageVisitor {
    type Value = CycleImage;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a canvas cycle JSON file")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: serde::de::MapAccess<'de>, {
        let mut width = None;
        let mut height = None;
        let mut palette = None;
        let mut cycles = None;
        let mut image = None;
        let mut filename = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "width" => {
                    width = Some(map.next_value()?);
                }
                "height" => {
                    height = Some(map.next_value()?);
                }
                "colors" => {
                    palette = Some(map.next_value()?);
                }
                "cycles" => {
                    cycles = Some(map.next_value()?);
                }
                "pixels" => {
                    image = Some(map.next_value()?);
                }
                "filename" => {
                    filename = Some(map.next_value()?);
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }

        let Some(width) = width else {
            return Err(Error::missing_field("width"));
        };

        let Some(height) = height else {
            return Err(Error::missing_field("height"));
        };

        let Some(palette) = palette else {
            return Err(Error::missing_field("colors"));
        };

        let Some(cycles) = cycles else {
            return Err(Error::missing_field("cycles"));
        };

        let Some(image) = image else {
            return Err(Error::missing_field("pixels"));
        };

        let Some(indexed_image) = IndexedImage::from_buffer(width, height, image, palette) else {
            return Err(Error::custom("image buffer is too small for given width/height"));
        };

        Ok(CycleImage::new(filename, indexed_image, cycles))
    }
}

impl<'de> serde::de::Deserialize<'de> for CycleImage {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_map(CycleImageVisitor)
    }
}

#[derive(Debug)]
struct Timeline(pub Vec<(u32, String)>);

struct TimelineVisitor;

impl<'de> Visitor<'de> for TimelineVisitor {
    type Value = Timeline;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a Living Worlds timeline: map of seconds to palette names or list of seconds-names tuples")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where A: serde::de::SeqAccess<'de> {
        let mut timeline: Vec<(u32, String)> = if let Some(size) = seq.size_hint() {
            Vec::with_capacity(size)
        } else {
            Vec::new()
        };

        while let Some(item) = seq.next_element()? {
            timeline.push(item);
        }

        timeline.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(Timeline(timeline))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: serde::de::MapAccess<'de> {
        let mut timeline = if let Some(size) = map.size_hint() {
            Vec::with_capacity(size)
        } else {
            Vec::new()
        };

        while let Some(time_of_day) = map.next_key::<String>()? {
            let time_of_day: u32 = match time_of_day.parse() {
                Ok(value) => value,
                Err(err) => return Err(Error::custom(format_args!("illegal time of day in timeline: {:?}\n{}", time_of_day, err)))
            };
            let name = map.next_value()?;
            timeline.push((time_of_day, name));
        }

        timeline.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(Timeline(timeline))
    }
}

impl<'de> serde::de::Deserialize<'de> for Timeline {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_any(TimelineVisitor)
    }
}

struct LivingWorldVisitor;

impl<'de> Visitor<'de> for LivingWorldVisitor {
    type Value = LivingWorld;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a Living Worlds file")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: serde::de::MapAccess<'de> {
        let mut width = None;
        let mut height = None;
        let mut palette = None;
        let mut cycles = None;
        let mut image = None;
        let mut filename = None;
        let mut format: Option<FormatInfo> = None;
        let mut data: Option<MagratheaWorldData> = None;
        let mut base: Option<CycleImage> = None;
        let mut palettes_map: Option<HashMap<String, CycleImage>> = None;
        let mut named_timeline: Option<Timeline> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "width" => {
                    width = Some(map.next_value()?);
                }
                "height" => {
                    height = Some(map.next_value()?);
                }
                "colors" => {
                    palette = Some(map.next_value()?);
                }
                "cycles" => {
                    cycles = Some(map.next_value()?);
                }
                "pixels" => {
                    image = Some(map.next_value()?);
                }
                "format" => {
                    format = Some(map.next_value()?);
                }
                "data" => {
                    data = Some(map.next_value()?);
                }
                "base" => {
                    base = Some(map.next_value()?);
                }
                "palettes" => {
                    palettes_map = Some(map.next_value()?);
                }
                "timeline" => {
                    named_timeline = Some(map.next_value()?);
                }
                "filename" => {
                    filename = Some(map.next_value()?);
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }

        if let Some(base) = base {
            let palettes_len: usize = if let Some(palettes) = &palettes_map { palettes.len() } else { 0 };

            let mut palettes = Vec::with_capacity(palettes_len);
            let mut index_map = HashMap::with_capacity(palettes_len);
            if let Some(palettes_map) = palettes_map {
                for (index, (key, image)) in palettes_map.into_iter().enumerate() {
                    index_map.insert(key, index);
                    palettes.push(image);
                }
            }

            let timeline_len = if let Some(Timeline(timeline)) = &named_timeline { timeline.len() } else { 0 };
            let mut timeline = Vec::with_capacity(timeline_len);
            if let Some(Timeline(named_timeline)) = named_timeline {
                for (time_of_day, palette_name) in named_timeline {
                    if let Some(palette_index) = index_map.get(&palette_name) {
                        timeline.push(TimedEvent::new(time_of_day, *palette_index));
                    } else {
                        return Err(Error::custom(format_args!("missing palette name referenced in timeline: {:?}", palette_name)));
                    }
                }
            }

            return Ok(LivingWorld::new(
                base.filename().map(|name| name.to_owned()),
                base,
                palettes.into_boxed_slice(), timeline.into_boxed_slice()),
            );
        }

        if let Some(format) = format {
            if format.version != 2 {
                return Err(Error::custom(format_args!("unsupported version: {}, expected: 2", format.version)));
            }

            let Some(data) = data else {
                return Err(Error::missing_field("data"));
            };

            let Some(palette_info) = data.palette_infos.into_iter().next() else {
                return Err(Error::custom("need at least one palette definition"));
            };

            let Some(indexed_image) = IndexedImage::from_buffer(data.width, data.height, data.pixels, palette_info.colors) else {
                return Err(Error::custom("image buffer is too small for given width/height"));
            };

            return Ok(CycleImage::new(Some(data.name), indexed_image, palette_info.cycles).into());
        }

        let Some(width) = width else {
            return Err(Error::missing_field("width"));
        };

        let Some(height) = height else {
            return Err(Error::missing_field("height"));
        };

        let Some(palette) = palette else {
            return Err(Error::missing_field("colors"));
        };

        let Some(cycles) = cycles else {
            return Err(Error::missing_field("cycles"));
        };

        let Some(image) = image else {
            return Err(Error::missing_field("pixels"));
        };

        let Some(indexed_image) = IndexedImage::from_buffer(width, height, image, palette) else {
            return Err(Error::custom("image buffer is too small for given width/height"));
        };

        Ok(CycleImage::new(filename, indexed_image, cycles).into())
    }
}

impl<'de> serde::de::Deserialize<'de> for LivingWorld {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_map(LivingWorldVisitor)
    }
}

struct RgbVisitor;

impl<'de> Visitor<'de> for RgbVisitor {
    type Value = Rgb;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("RGB value as list of 3 numbers, each in the range of 0 to 255")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where A: serde::de::SeqAccess<'de>, {
        let Some(r) = seq.next_element()? else {
            return Err(Error::missing_field("r"));
        };

        let Some(g) = seq.next_element()? else {
            return Err(Error::missing_field("g"));
        };

        let Some(b) = seq.next_element()? else {
            return Err(Error::missing_field("b"));
        };

        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(Error::custom("superfluous elements in RGB value"));
        };

        Ok(Rgb([r, g, b]))
    }
}

impl<'de> serde::de::Deserialize<'de> for Rgb {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_seq(RgbVisitor)
    }
}

struct PaletteVisitor;

impl<'de> Visitor<'de> for PaletteVisitor {
    type Value = Palette;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a list of 256 RGB values")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where A: serde::de::SeqAccess<'de>, {
        let mut colors = Vec::with_capacity(256);

        while let Some(rgb) = seq.next_element()? {
            colors.push(rgb);
        }

        let colors: Box<[Rgb; 256]> = match colors.try_into() {
            Ok(colors) => colors,
            Err(_) => return Err(Error::custom("the color palette needs to have exactly 256 color values"))
        };

        Ok(Palette(colors))
    }
}

impl<'de> serde::de::Deserialize<'de> for Palette {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_seq(PaletteVisitor)
    }
}

struct CycleVisitor;

impl<'de> Visitor<'de> for CycleVisitor {
    type Value = Cycle;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a color cycle definition")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: serde::de::MapAccess<'de>, {
        let mut reverse = false;
        let mut rate = 0;
        let mut low = None;
        let mut high = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "reverse" => {
                    let value: i32 = map.next_value()?;
                    if value == 0 {
                        reverse = false;
                    } else if value == 2 {
                        reverse = true;
                    } else {
                        return Err(Error::invalid_value(
                            serde::de::Unexpected::Signed(value as i64),
                            &"0 or 2"));
                    }
                }
                "rate" => {
                    rate = map.next_value()?;
                }
                "low" => {
                    low = Some(map.next_value()?);
                }
                "high" => {
                    high = Some(map.next_value()?);
                }
                _ => {
                    let _ = map.next_value::<IgnoredAny>()?;
                }
            }
        }

        let Some(low) = low else {
            return Err(Error::missing_field("low"));
        };

        let Some(high) = high else {
            return Err(Error::missing_field("high"));
        };

        Ok(Cycle::new(low, high, rate, reverse))
    }
}

impl<'de> serde::de::Deserialize<'de> for Cycle {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_map(CycleVisitor)
    }
}
