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

use std::fmt::Write;

use crate::color::Rgb;
use crate::image::RgbImage;

#[inline]
pub fn image_to_ansi(prev_frame: &RgbImage, image: &RgbImage, full_width: bool) -> String {
    let mut lines = String::new();
    image_to_ansi_into(prev_frame, image, full_width, &mut lines);
    lines
}

#[inline]
fn move_cursor(curr_x: u32, curr_line_y: u32, x: u32, line_y: u32, lines: &mut String) {
    if x != curr_x {
        if x > curr_x {
            let dx = x - curr_x;
            if dx == 1 {
                lines.push_str("\x1B[C");
            } else {
                let _ = write!(lines, "\x1B[{dx}C");
            }
        } else {
            let dx = curr_x - x;
            if dx == 1 {
                lines.push_str("\x1B[D");
            } else {
                let _ = write!(lines, "\x1B[{dx}D");
            }
        }
    }

    if line_y != curr_line_y {
        if line_y > curr_line_y {
            let dy = line_y - curr_line_y;
            if dy == 1 {
                lines.push_str("\x1B[B");
            } else {
                let _ = write!(lines, "\x1B[{dy}B");
            }
        } else {
            let dy = curr_line_y - line_y;
            if dy == 1 {
                lines.push_str("\x1B[A");
            } else {
                let _ = write!(lines, "\x1B[{dy}A");
            }
        }
    }
}

pub fn image_to_ansi_into(prev_frame: &RgbImage, image: &RgbImage, full_width: bool, lines: &mut String) {
    if prev_frame.width() < image.width() {
        panic!("prev_frame.width() < image.width(): {:?} < {:?}", prev_frame.width(), image.width());
    }

    if prev_frame.height() < image.height() {
        panic!("prev_frame.height() < image.height(): {:?} < {:?}", prev_frame.height(), image.height());
    }

    let row_count = image.height().div_ceil(2);

    lines.clear();

    if row_count == 0 {
        return;
    }

    let width = image.width();
    let line_len = (width as usize) * "\x1B[38;2;255;255;255\x1B[48;2;255;255;255m▄".len() + "\x1B[0m".len();

    lines.reserve(line_len * row_count as usize + "\x1B[0m".len());

    let mut curr_line_y = 0;
    let mut curr_x = 0;

    for line_y in 0..row_count {
        let y = line_y * 2;
        let mut line_start = true;
        if y + 1 == image.height() {
            let mut prev_color = Rgb([0, 0, 0]);
            for x in 0..image.width() {
                let color = image.get_pixel(x, y);
                if color != prev_frame.get_pixel(x, y) {
                    move_cursor(curr_x, curr_line_y, x, line_y, lines);
                    let Rgb([r, g, b]) = color;
                    if !line_start && color == prev_color {
                        lines.push('▀');
                    } else {
                        let _ = write!(lines, "\x1B[38;2;{r};{g};{b}m▀");
                        line_start = false;
                    }
                    prev_color = color;
                    // NOTE: Cursor location doesn't update at the end of the screen.
                    // This assumes that the image is rendered up to the end of the screen!
                    if full_width && (x + 1) == width {
                        curr_x = x;
                    } else {
                        curr_x = x + 1;
                    }
                    curr_line_y = line_y;
                }
            }
        } else {
            let mut prev_bg = Rgb([0, 0, 0]);
            let mut prev_fg = Rgb([0, 0, 0]);
            for x in 0..image.width() {
                let color_top    = image.get_pixel(x, y);
                let color_bottom = image.get_pixel(x, y + 1);

                if color_top != prev_frame.get_pixel(x, y) || color_bottom != prev_frame.get_pixel(x, y + 1) {
                    move_cursor(curr_x, curr_line_y, x, line_y, lines);
                    let Rgb([r1, g1, b1]) = color_top;

                    if color_top == color_bottom {
                        let _ = write!(lines, "\x1B[38;2;{r1};{g1};{b1}m█");
                        prev_fg = color_top;
                        prev_bg = color_top;
                        line_start = false;
                    } else {
                        let Rgb([r2, g2, b2]) = color_bottom;
                        if line_start {
                            let _ = write!(lines, "\x1B[48;2;{r1};{g1};{b1}m\x1B[38;2;{r2};{g2};{b2}m▄");
                            prev_fg = color_bottom;
                            prev_bg = color_top;
                            line_start = false;
                        } else if prev_fg == color_bottom && prev_bg == color_top {
                            let _ = write!(lines, "▄");
                        } else if prev_fg == color_top && prev_bg == color_bottom {
                            let _ = write!(lines, "▀");
                        } else if prev_fg == color_bottom {
                            let _ = write!(lines, "\x1B[48;2;{r1};{g1};{b1}m▄");
                            prev_bg = color_top;
                        } else if prev_fg == color_top {
                            let _ = write!(lines, "\x1B[48;2;{r2};{g2};{b2}m▀");
                            prev_bg = color_bottom;
                        } else if prev_bg == color_top {
                            let _ = write!(lines, "\x1B[38;2;{r2};{g2};{b2}m▄");
                            prev_fg = color_bottom;
                        } else if prev_bg == color_bottom {
                            let _ = write!(lines, "\x1B[38;2;{r1};{g1};{b1}m▀");
                            prev_fg = color_top;
                        } else {
                            let _ = write!(lines, "\x1B[48;2;{r1};{g1};{b1}m\x1B[38;2;{r2};{g2};{b2}m▄");
                            prev_fg = color_bottom;
                            prev_bg = color_top;
                        }
                    }
                    // NOTE: Cursor location doesn't update at the end of the screen.
                    // This assumes that the image is rendered up to the end of the screen!
                    if full_width && (x + 1) == width {
                        curr_x = x;
                    } else {
                        curr_x = x + 1;
                    }
                    curr_line_y = line_y;
                }
            }
        }
    }

    // Just to ensure that the cursor is at the correct position after
    // the image is rendered or when hitting Ctrl+C during sleep.
    let dx = image.width() - curr_x;
    if dx > 0 {
        if dx == 1 {
            lines.push_str("\x1B[C");
        } else {
            let _ = write!(lines, "\x1B[{dx}C");
        }
    }

    let dy = row_count - 1 - curr_line_y;
    if dy > 0 {
        if dy == 1 {
            lines.push_str("\x1B[B");
        } else {
            let _ = write!(lines, "\x1B[{dy}B");
        }
    }
}

pub fn simple_image_to_ansi_into(image: &RgbImage, lines: &mut String) {
    let row_count = image.height().div_ceil(2);

    lines.clear();

    if row_count == 0 {
        return;
    }

    let width = image.width();
    let line_len = (width as usize) * "\x1B[38;2;255;255;255\x1B[48;2;255;255;255m▄".len() + "\x1B[1234D\x1B[1B".len();

    lines.reserve(line_len * row_count as usize + "\x1B[0m".len());

    for line_y in 0..row_count {
        if line_y > 0 {
            let _ = write!(lines, "\x1B[{}D\x1B[1B", width);
        }
        let y = line_y * 2;
        if y + 1 == image.height() {
            let mut prev_color = Rgb([0, 0, 0]);
            for x in 0..image.width() {
                let color = image.get_pixel(x, y);
                let Rgb([r, g, b]) = color;
                if x > 0 && color == prev_color {
                    lines.push('▀');
                } else {
                    let _ = write!(lines, "\x1B[38;2;{r};{g};{b}m▀");
                }
                prev_color = color;
            }
        } else {
            let mut prev_bg = Rgb([0, 0, 0]);
            let mut prev_fg = Rgb([0, 0, 0]);
            for x in 0..image.width() {
                let color_top    = image.get_pixel(x, y);
                let color_bottom = image.get_pixel(x, y + 1);

                let Rgb([r1, g1, b1]) = color_top;

                if color_top == color_bottom {
                    let _ = write!(lines, "\x1B[38;2;{r1};{g1};{b1}m█");
                    prev_fg = color_top;
                    prev_bg = color_top;
                } else {
                    let Rgb([r2, g2, b2]) = color_bottom;
                    if x == 0 {
                        let Rgb([r2, g2, b2]) = color_bottom;
                        let _ = write!(lines, "\x1B[48;2;{r1};{g1};{b1}m\x1B[38;2;{r2};{g2};{b2}m▄");
                        prev_fg = color_bottom;
                        prev_bg = color_top;
                    } else if prev_fg == color_bottom && prev_bg == color_top {
                        let _ = write!(lines, "▄");
                    } else if prev_fg == color_top && prev_bg == color_bottom {
                        let _ = write!(lines, "▀");
                    } else if prev_fg == color_bottom {
                        let _ = write!(lines, "\x1B[48;2;{r1};{g1};{b1}m▄");
                        prev_bg = color_top;
                    } else if prev_fg == color_top {
                        let _ = write!(lines, "\x1B[48;2;{r2};{g2};{b2}m▀");
                        prev_bg = color_bottom;
                    } else if prev_bg == color_top {
                        let _ = write!(lines, "\x1B[38;2;{r2};{g2};{b2}m▄");
                        prev_fg = color_bottom;
                    } else if prev_bg == color_bottom {
                        let _ = write!(lines, "\x1B[38;2;{r1};{g1};{b1}m▀");
                        prev_fg = color_top;
                    } else {
                        let _ = write!(lines, "\x1B[48;2;{r1};{g1};{b1}m\x1B[38;2;{r2};{g2};{b2}m▄");
                        prev_fg = color_bottom;
                        prev_bg = color_top;
                    }
                }
            }
        }
    }

    lines.push_str("\x1B[0m");
}
