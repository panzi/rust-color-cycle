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

pub mod image_to_ansi;
pub mod color;
pub mod image;
pub mod palette;
pub mod read;
pub mod ilbm;
pub mod bitvec;
pub mod error;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::{BufReader, Read, Seek, StdinLock, StdoutLock, Write};

#[cfg(not(windows))]
use std::mem::MaybeUninit;

use clap::Parser;
use image::{CycleImage, IndexedImage, LivingWorld, RgbImage};
use image_to_ansi::{image_to_ansi_into, simple_image_to_ansi_into};

#[cfg(not(windows))]
use libc;
use palette::Palette;

const MAX_FPS: u32 = 10_000;
const TIME_STEP: u64 = 5 * 60 * 1000;
const SMALL_TIME_STEP: u64 = 60 * 1000;
const DAY_DURATION: u64 = 24 * 60 * 60 * 1000;
const FAST_FORWARD_SPEED: u64 = 10_000;

pub struct NBTerm;

impl NBTerm {
    pub fn new() -> Result<Self, error::Error> {
        #[cfg(not(windows))]
        unsafe {
            let mut ttystate = MaybeUninit::<libc::termios>::zeroed();
            let res = libc::tcgetattr(libc::STDIN_FILENO, ttystate.as_mut_ptr());
            if res == -1 {
                let err = std::io::Error::last_os_error();
                return Err(err.into());
            }

            let ttystate = ttystate.assume_init_mut();

            // turn off canonical mode
            ttystate.c_lflag &= !(libc::ICANON | libc::ECHO);

            // minimum of number input read.
            ttystate.c_cc[libc::VMIN] = 0;
            ttystate.c_cc[libc::VTIME] = 0;

            let res = libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, ttystate);
            if res == -1 {
                let err = std::io::Error::last_os_error();
                return Err(err.into());
            }
        }

//        #[cfg(windows)]
//        unsafe {
//            use winapi::shared::minwindef::{DWORD, FALSE};
//
//            let handle = winapi::um::processenv::GetStdHandle(winapi::um::winbase::STD_INPUT_HANDLE);
//            if handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
//                let err = std::io::Error::last_os_error();
//                return Err(err);
//            }
//
//            let mut mode: DWORD = 0;
//
//            if winapi::um::consoleapi::GetConsoleMode(handle, &mut mode as *mut DWORD) == FALSE {
//                let err = std::io::Error::last_os_error();
//                return Err(err);
//            }
//
//            if winapi::um::consoleapi::SetConsoleMode(handle, mode & !(winapi::um::wincon::ENABLE_ECHO_INPUT | winapi::um::wincon::ENABLE_LINE_INPUT)) == FALSE {
//                let err = std::io::Error::last_os_error();
//                return Err(err);
//            }
//        }

        // CSI ? 25 l     Hide cursor (DECTCEM), VT220
        // CSI ?  7 l     No Auto-Wrap Mode (DECAWM), VT100.
        // CSI 2 J        Clear entire screen
        print!("\x1B[?25l\x1B[?7l\x1B[2J");

        Ok(Self)
    }
}

impl Drop for NBTerm {
    fn drop(&mut self) {
        #[cfg(not(windows))]
        unsafe {
            let mut ttystate = MaybeUninit::<libc::termios>::zeroed();
            let res = libc::tcgetattr(libc::STDIN_FILENO, ttystate.as_mut_ptr());
            if res == 0 {
                let ttystate = ttystate.assume_init_mut();

                // turn on canonical mode
                ttystate.c_lflag |= libc::ICANON | libc::ECHO;

                let _ = libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, ttystate);
            }
        }

//        #[cfg(windows)]
//        unsafe {
//            use winapi::shared::minwindef::{DWORD, FALSE};
//            let handle = winapi::um::processenv::GetStdHandle(winapi::um::winbase::STD_INPUT_HANDLE);
//            if handle != winapi::um::handleapi::INVALID_HANDLE_VALUE {
//                let mut mode: DWORD = 0;
//
//                if winapi::um::consoleapi::GetConsoleMode(handle, &mut mode as *mut DWORD) != FALSE {
//                    winapi::um::consoleapi::SetConsoleMode(handle, mode | winapi::um::wincon::ENABLE_ECHO_INPUT | winapi::um::wincon::ENABLE_LINE_INPUT);
//                }
//            }
//        }

        // CSI 0 m        Reset or normal, all attributes become turned off
        // CSI ? 25 h     Show cursor (DECTCEM), VT220
        // CSI ?  7 h     Auto-Wrap Mode (DECAWM), VT100
        println!("\x1B[0m\x1B[?25h\x1B[?7h");
    }
}

fn interruptable_sleep(duration: Duration) -> bool {
    #[cfg(unix)]
    {
        let req = libc::timespec {
            tv_sec:  duration.as_secs() as libc::time_t,
            tv_nsec: duration.subsec_nanos() as i64,
        };
        let ret = unsafe { libc::nanosleep(&req, std::ptr::null_mut()) };
        return ret == 0;
    }

    #[cfg(not(unix))]
    {
        std::thread::sleep(duration);
        return true;
    }
}

#[cfg(windows)]
extern {
    fn _getch() -> core::ffi::c_char;
    fn _kbhit() -> core::ffi::c_int;
}

#[cfg(windows)]
fn nb_read_byte(mut _reader: impl Read) -> std::io::Result<Option<u8>> {
    unsafe {
        if _kbhit() == 0 {
            return Ok(None);
        }

        let ch = _getch();
        Ok(Some(ch as u8))
    }
}

#[cfg(not(windows))]
fn nb_read_byte(mut reader: impl Read) -> std::io::Result<Option<u8>> {
    let mut buf = [0u8];
    loop {
        return match reader.read(&mut buf) {
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::WouldBlock => Ok(None),

                    #[cfg(not(windows))]
                    std::io::ErrorKind::Other if err.raw_os_error() == Some(libc::EAGAIN) => Ok(None),

                    std::io::ErrorKind::Interrupted => continue,
                    _ => Err(err)
                }
            }
            Ok(count) => if count == 0 {
                Ok(None)
            } else {
                Ok(Some(buf[0]))
            }
        };
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, after_help = "\
color-cycle  Copyright (C) 2025  Mathias Panzenböck
License: GPL-3.0
Bugs: https://github.com/panzi/rust-color-cycle/issues"
)]
pub struct Args {
    /// Frames per second.
    /// 
    /// Attempt to render in this number of frames per second.
    /// Actual FPS might be lower.
    #[arg(short, long, default_value_t = 60, value_parser = clap::value_parser!(u32).range(1..MAX_FPS as i64))]
    pub fps: u32,

    /// Enable blend mode.
    /// 
    /// This blends the animated color palette for smoother display.
    #[arg(short, long, default_value_t = false)]
    pub blend: bool,

    /// Enable On Screen Display.
    /// 
    /// Displays messages when changing things like blend mode or FPS.{n}
    #[arg(short, long, default_value_t = false)]
    pub osd: bool,

    /// Swap direction of 8 pixel columns.
    /// 
    /// The current implementation of ILBM files is broken for some files and
    /// swaps the pixels in columns like that. I haven't figured out how do load
    /// those files correctly (how to detect its such a file), but this option
    /// can be used to fix the display of those files.
    #[arg(long, default_value_t = false)]
    pub ilbm_column_swap: bool,

    /// Show list of hotkeys.
    #[arg(long, default_value_t = false)]
    pub help_hotkeys: bool,

    /// Path to a Canvas Cycle JSON file.
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,
}

struct GlobalState {
    running: Arc<AtomicBool>,
    current_time: Option<u64>,
    time_speed: u64,
    stdin: StdinLock<'static>,
    stdout: StdoutLock<'static>,
}

fn main() {
    let mut args = Args::parse();

    if args.help_hotkeys {
        println!("\
Hotkeys
=======
B              Toggle blend mode
Q or Escape    Quit program
O              Toggle On Screen Display
N              Open next file
P              Open previous file
1 to 9         Open file by index
0              Open last file
+              Increase frames per second by 1
-              Decrease frames per second by 1
W              Toogle fast forward ({FAST_FORWARD_SPEED}x speed)
A              Go back in time by 5 minutes
Shift+A        Go back in time by 1 minute
D              Go forward in time by 5 minutes
Shift+D        Go forward in time by 1 minute
S              Go to current time and continue normal progression
I              Reverse pixels in columns of 8.
               This is a hack fix for images that appear to be
               broken like that.
Cursor Up      Move view-port up by 1 pixel
Cursor Down    Move view-port down by 1 pixel
Cursor Left    Move view-port left by 1 pixel
Cursor Right   Move view-port right by 1 pixel
Home           Move view-port to left edge
End            Move view-port to right edge
Ctrl+Home      Move view-port to top
Ctrl+End       Move view-port to bottom
Page Up        Move view-port up by half a screen
Page Down      Move view-port down by half a screen
Alt+Page Up    Move view-port left by half a screen
Alt+Page Down  Move view-port right by half a screen");
        return;
    }

    let mut state = GlobalState {
        running: Arc::new(AtomicBool::new(true)),
        stdin: std::io::stdin().lock(),
        stdout: std::io::stdout().lock(),
        current_time: None,
        time_speed: 1,
    };

    {
        let running = state.running.clone();
        let _ = ctrlc::set_handler(move || {
            running.store(false, Ordering::Relaxed);
        });
    }

    let mut file_index = 0;

    let res = match NBTerm::new() {
        Err(err) => Err(err),
        Ok(_nbterm) => {
            loop {
                match show_image(&mut args, &mut state, file_index) {
                    Ok(Action::Goto(index)) => {
                        file_index = index;
                    }
                    Ok(Action::Quit) => {
                        break Ok(());
                    }
                    Err(err) => {
                        break Err(err);
                    }
                }
            }
        }
    };

    if let Err(err) = res {
        eprintln!("{}: {}", args.paths[file_index].to_string_lossy(), err);
        std::process::exit(1);
    }
}

enum Action {
    Goto(usize),
    Quit,
}

fn get_time_of_day_msec(time_speed: u64) -> u64 {
    #[cfg(not(windows))]
    unsafe {
        let mut tod = MaybeUninit::<libc::timespec>::zeroed();
        if libc::clock_gettime(libc::CLOCK_REALTIME, tod.as_mut_ptr()) != 0 {
            return 0;
        }
        let tod = tod.assume_init_ref();
        let mut tm = MaybeUninit::<libc::tm>::zeroed();
        if libc::localtime_r(&tod.tv_sec, tm.as_mut_ptr()).is_null() {
            return 0;
        }
        let tm = tm.assume_init_ref();
        let mut now = Duration::new(tod.tv_sec as u64, tod.tv_nsec as u32);

        if tm.tm_gmtoff > 0 {
            now += Duration::from_secs(tm.tm_gmtoff as u64);
        } else {
            now -= Duration::from_secs((-tm.tm_gmtoff) as u64);
        }

        ((now.as_millis() * time_speed as u128) % DAY_DURATION as u128) as u64
    }

    #[cfg(windows)]
    unsafe {
        let mut tm = MaybeUninit::<winapi::um::minwinbase::SYSTEMTIME>::zeroed();
        winapi::um::sysinfoapi::GetLocalTime(tm.as_mut_ptr());
        let tm = tm.assume_init_ref();

        (
            tm.wHour as u64 * 60 * 60 * 1000 +
            tm.wMinute as u64 * 60 * 1000 +
            tm.wSecond as u64 * 1000 +
            tm.wMilliseconds as u64
        ) * time_speed % DAY_DURATION
    }
}

fn get_hours_mins(time_of_day: u64) -> (u32, u32) {
    let mins = (time_of_day / (60 * 1000)) as u32;
    let hours = mins / 60;
    (hours, mins - hours * 60)
}

const MESSAGE_DISPLAY_DURATION: Duration = Duration::from_secs(3);
const ERROR_MESSAGE_DISPLAY_DURATION: Duration = Duration::from_secs(1000 * 365 * 24 * 60 * 60);

fn show_image(args: &mut Args, state: &mut GlobalState, file_index: usize) -> Result<Action, error::Error> {
    let path = &args.paths[file_index];
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let living_world: Result<LivingWorld, error::Error> = match ilbm::ILBM::read(&mut reader) {
        Ok(ilbm) => {
            let res: Result<CycleImage, _> = ilbm.try_into();
            match res {
                Ok(image) => Ok(image.into()),
                Err(err) => Err(err.into())
            }
        }
        Err(err) => {
            if err.kind() != ilbm::ErrorKind::UnsupportedFileFormat {
                Err(err.into())
            } else if let Err(err) = reader.seek(std::io::SeekFrom::Start(0)) {
                Err(err.into())
            } else {
                match serde_json::from_reader(&mut reader) {
                    Ok(image) => Ok(image),
                    Err(err) => Err(err.into())
                }
            }
        }
    };
    drop(reader);

    let filename = path.file_name().map(|f| f.to_string_lossy()).unwrap_or_else(|| path.to_string_lossy());
    let mut message = String::new();
    let mut message_end_ts = Instant::now();
    let mut living_world = match living_world {
        Ok(living_world) => {
            use std::fmt::Write;

            if living_world.base().width() == 0 || living_world.base().height() == 0 {
                message_end_ts += ERROR_MESSAGE_DISPLAY_DURATION;
                let _ = write!(message, " {filename}: image of size {} x {} ",
                    living_world.base().width(),
                    living_world.base().height());
                CycleImage::new(None, IndexedImage::new(80, 25, Palette::default()), Box::new([])).into()
            } else {
                if args.osd {
                    if let Some(name) = living_world.name() {
                        let _ = write!(message, " {name} ({filename}) ");
                    } else {
                        let _ = write!(message, " {filename} ");
                    }
                    message_end_ts += MESSAGE_DISPLAY_DURATION
                }

                living_world
            }
        },
        Err(err) => {
            use std::fmt::Write;
            message_end_ts += ERROR_MESSAGE_DISPLAY_DURATION;
            let _ = write!(message, " {filename}: {err} ");
            CycleImage::new(None, IndexedImage::new(80, 25, Palette::default()), Box::new([])).into()
        }
    };
    // TODO: implement full worlds demo support
    let cycle_image = living_world.base();
    let mut blended_palette = cycle_image.palette().clone();
    let mut cycled_palette1 = blended_palette.clone();
    let mut cycled_palette2 = blended_palette.clone();

    let mut frame_duration = Duration::from_secs_f64(1.0 / (args.fps as f64));
    let mut linebuf = String::new();

    let img_width = cycle_image.width();
    let img_height = cycle_image.height();
    let (term_width, term_height) = {
        let term_size = term_size::dimensions();
        if let Some((columns, rows)) = term_size {
            (columns as u32, rows as u32 * 2)
        } else {
            (img_width, img_height)
        }
    };

    // initial blank screen
    let _ = write!(state.stdout, "\x1B[1;1H\x1B[38;2;0;0;0m\x1B[48;2;0;0;0m\x1B[2J");
    let _ = state.stdout.flush();

    let mut x = 0;
    let mut y = 0;

    if img_width > term_width {
        x = (img_width - term_width) / 2;
    }

    if img_height > term_height {
        y = (img_height - term_height) / 2;
    }

    let mut viewport = cycle_image.get_rect(
        x, y,
        img_width.min(term_width),
        img_height.min(term_height));

    let mut frame = RgbImage::new(viewport.width(), viewport.height());
    let mut prev_frame = RgbImage::new(viewport.width(), viewport.height());

    let mut old_term_width = term_width;
    let mut old_term_height = term_height;

    let mut message_shown = args.osd;

    let loop_start_ts = Instant::now();
    let mut message_end_ts = if args.osd {
        loop_start_ts + MESSAGE_DISPLAY_DURATION
    } else {
        loop_start_ts
    };

    while state.running.load(Ordering::Relaxed) {
        let frame_start_ts = Instant::now();
        let mut time_of_day = if let Some(current_time) = state.current_time {
            current_time
        } else {
            get_time_of_day_msec(state.time_speed)
        };

        // process input
        let term_size = term_size::dimensions();
        let (term_width, term_height) = if let Some((columns, rows)) = term_size {
            (columns as u32, rows as u32 * 2)
        } else {
            (img_width, img_height)
        };

        let old_message_len = message.len();
        let old_x = x;
        let old_y = y;

        let mut viewport_x = 0;
        let mut viewport_y = 0;

        if img_width <= term_width {
            x = 0;
            viewport_x = (term_width - img_width) / 2;
        } else if x > img_width - term_width {
            x = img_width - term_width;
        }

        if img_height <= term_height {
            y = 0;
            viewport_y = (term_height - img_height) / 2;
        } else if y > img_height - term_height {
            y = img_height - term_height;
        }

        let mut updated_message = false;
        macro_rules! show_message {
            ($($args:expr),+) => {
                if args.osd {
                    message_end_ts = frame_start_ts + MESSAGE_DISPLAY_DURATION;
                    message.clear();
                    use std::fmt::Write;
                    message.push_str(" ");
                    let _ = write!(&mut message, $($args),+);
                    message.push_str(" ");
                    updated_message = true;
                }
            };
        }

        loop {
            // TODO: Windows support, maybe with ReadConsoleInput()?
            let Some(byte) = nb_read_byte(&mut state.stdin)? else {
                break;
            };
            match byte {
                b'q' => return Ok(Action::Quit),
                b'b' => {
                    args.blend = !args.blend;

                    show_message!("Blend Mode: {}", if args.blend { "Enabled" } else { "Disabled" });
                }
                b'o' => {
                    if args.osd {
                        show_message!("OSD: Disabled");
                        args.osd = false;
                    } else {
                        args.osd = true;
                        show_message!("OSD: Enabled");
                    }
                }
                b'+' => {
                    if args.fps < MAX_FPS {
                        args.fps += 1;
                        frame_duration = Duration::from_secs_f64(1.0 / args.fps as f64);

                        show_message!("FPS: {}", args.fps);
                    }
                }
                b'-' => {
                    if args.fps > 1 {
                        args.fps -= 1;
                        frame_duration = Duration::from_secs_f64(1.0 / args.fps as f64);

                        show_message!("FPS: {}", args.fps);
                    }
                }
                b'n' => {
                    let new_index = file_index + 1;
                    if new_index >= args.paths.len() {
                        show_message!("Already at last file.");
                    } else {
                        return Ok(Action::Goto(new_index));
                    }
                }
                b'p' => {
                    if file_index == 0 {
                        show_message!("Already at first file.");
                    } else {
                        return Ok(Action::Goto(file_index - 1));
                    }
                }
                b'a' | b'A' => {
                    let time_step = if byte.is_ascii_uppercase() { SMALL_TIME_STEP } else { TIME_STEP };
                    let rem = time_of_day % time_step;
                    let new_time = time_of_day - rem;
                    if new_time == time_of_day {
                        if new_time < time_step {
                            time_of_day = DAY_DURATION - time_step;
                        } else {
                            time_of_day = new_time - time_step;
                        }
                    } else {
                        time_of_day = new_time;
                    }
                    state.time_speed = 1;
                    state.current_time = Some(time_of_day);
                    let (hours, mins) = get_hours_mins(time_of_day);
                    show_message!("{hours}:{mins:02}");
                }
                b'd' | b'D' => {
                    let time_step = if byte.is_ascii_uppercase() { SMALL_TIME_STEP } else { TIME_STEP };
                    let rem = time_of_day % time_step;
                    let new_time = time_of_day - rem + time_step;
                    if new_time >= DAY_DURATION {
                        time_of_day = 0;
                    } else {
                        time_of_day = new_time;
                    }
                    state.time_speed = 1;
                    state.current_time = Some(time_of_day);
                    let (hours, mins) = get_hours_mins(time_of_day);
                    show_message!("{hours}:{mins:02}");
                }
                b's' => {
                    state.time_speed = 1;
                    state.current_time = None;
                    time_of_day = get_time_of_day_msec(state.time_speed);
                    let (hours, mins) = get_hours_mins(time_of_day);
                    show_message!("{hours}:{mins:02}");
                }
                b'w' => {
                    if state.time_speed == 1 {
                        state.time_speed = FAST_FORWARD_SPEED;
                        state.current_time = None;
                        time_of_day = get_time_of_day_msec(state.time_speed);
                        show_message!("Fast Forward: ON");
                    } else {
                        state.time_speed = 1;
                        state.current_time = Some(time_of_day);
                        show_message!("Fast Forward: OFF");
                    }
                }
                b'i' => {
                    living_world.column_swap();
                    viewport.get_rect_from(x, y, term_width, term_height, living_world.base());
                }
                0x1b => {
                    match nb_read_byte(&mut state.stdin)? {
                        Option::None => return Ok(Action::Quit),
                        Some(0x1b) => return Ok(Action::Quit),
                        Some(b'[') => {
                            match nb_read_byte(&mut state.stdin)? {
                                Option::None => break,
                                Some(b'A') => {
                                    // Up
                                    if img_height > term_height && y > 0 {
                                        y -= 1;
                                    }
                                }
                                Some(b'B') => {
                                    // Down
                                    if img_height > term_height && y < (img_height - term_height) {
                                        y += 1;
                                    }
                                }
                                Some(b'C') => {
                                    // Right
                                    if img_width > term_width && x < (img_width - term_width) {
                                        x += 1;
                                    }
                                }
                                Some(b'D') => {
                                    // Left
                                    if img_width > term_width && x > 0 {
                                        x -= 1;
                                    }
                                }
                                Some(b'H') => {
                                    // Home
                                    if img_width > term_width {
                                        x = 0;
                                    }
                                }
                                Some(b'F') => {
                                    // End
                                    if img_width > term_width {
                                        x = img_width - term_width;
                                    }
                                }
                                Some(b'1') => {
                                    match nb_read_byte(&mut state.stdin)? {
                                        Option::None => break,
                                        Some(b';') => {
                                            match nb_read_byte(&mut state.stdin)? {
                                                None => break,
                                                Some(b'5') => {
                                                    match nb_read_byte(&mut state.stdin)? {
                                                        None => break,
                                                        Some(b'H') => {
                                                            // Ctrl+Home
                                                            if img_height > term_height {
                                                                y = 0;
                                                            }
                                                        }
                                                        Some(b'F') => {
                                                            // Ctrl+End
                                                            if img_height > term_height {
                                                                y = img_height - term_height;
                                                            }
                                                        }
                                                        _ => break,
                                                    }
                                                }
                                                _ => break,
                                            }
                                        }
                                        _ => break,
                                    }
                                }
                                Some(b'5') => {
                                    match nb_read_byte(&mut state.stdin)? {
                                        Option::None => break,
                                        Some(b'~') => {
                                            // Page Up
                                            if img_height > term_height {
                                                let half = term_height / 2;
                                                if y > half {
                                                    y -= half;
                                                } else {
                                                    y = 0;
                                                }
                                            }
                                        }
                                        Some(b';') => {
                                            match nb_read_byte(&mut state.stdin)? {
                                                Option::None => break,
                                                Some(b'3') => {
                                                    match nb_read_byte(&mut state.stdin)? {
                                                        Option::None => break,
                                                        Some(b'~') => {
                                                            // Alt+Page Up
                                                            if img_width > term_width {
                                                                let half = term_width / 2;
                                                                if x > half {
                                                                    x -= half;
                                                                } else {
                                                                    x = 0;
                                                                }
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                Some(b'6') => {
                                    match nb_read_byte(&mut state.stdin)? {
                                        Option::None => break,
                                        Some(b'~') => {
                                            // Page Down
                                            if img_height > term_height {
                                                let half = term_height / 2;
                                                let max_y = img_height - term_height;
                                                y += half;
                                                if y > max_y {
                                                    y = max_y;
                                                }
                                            }
                                        }
                                        Some(b';') => {
                                            match nb_read_byte(&mut state.stdin)? {
                                                Option::None => break,
                                                Some(b'3') => {
                                                    match nb_read_byte(&mut state.stdin)? {
                                                        Option::None => break,
                                                        Some(b'~') => {
                                                            // Alt+Page Down
                                                            if img_width > term_width {
                                                                let half = term_width / 2;
                                                                let max_x = img_width - term_width;
                                                                x += half;
                                                                if x > max_x {
                                                                    x = max_x;
                                                                }
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                Some(byte) => {
                                    if byte.is_ascii_digit() || byte == b';' {
                                        // eat whole unsupported escape input sequence
                                        loop {
                                            let Some(byte) = nb_read_byte(&mut state.stdin)? else {
                                                break;
                                            };

                                            if !byte.is_ascii_digit() && byte != b';' {
                                                break;
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                b'0' => {
                    return Ok(Action::Goto(args.paths.len() - 1));
                }
                b'1' => {
                    return Ok(Action::Goto(0));
                }
                _ if byte >= b'2' && byte <= b'9' => {
                    let index = (byte - b'1') as usize;
                    if index >= args.paths.len() {
                        show_message!("Only {} files opened!", args.paths.len());
                    } else {
                        return Ok(Action::Goto(index));
                    }
                }
                _ => {}
            }
        }

        // render frame
        let mut full_redraw = false;
        let viewport_row = viewport_y / 2 + 1;
        let viewport_column = viewport_x + 1;
        if old_x != x || old_y != y || old_term_width != term_width || old_term_height != term_height {
            viewport.get_rect_from(x, y, term_width, term_height, living_world.base());
            frame = RgbImage::new(viewport.width(), viewport.height());

            if old_term_width != term_width || old_term_height != term_height {
                prev_frame = RgbImage::new(viewport.width(), viewport.height());
                full_redraw = true;

                //let _ = write!(state.stdout, "\x1B[38;2;0;0;0m\x1B[48;2;0;0;0m\x1B[2J");
                if viewport.width() < term_width || viewport.height() < term_height {
                    let _ = write!(state.stdout, "\x1B[38;2;0;0;0m\x1B[48;2;0;0;0m");

                    if viewport_y > 0 {
                        let _ = write!(state.stdout, "\x1B[{};1H\x1B[1J", viewport_row);
                    }

                    let viewport_rows = (viewport.height() + 1) / 2;
                    let viewport_end_row = viewport_row + viewport_rows;
                    if viewport_x > 0 {
                        let column = viewport_column - 1;
                        for row in viewport_row..viewport_end_row {
                            let _ = write!(state.stdout, "\x1B[{};{}H\x1B[1K", row, column);
                        }
                    }

                    if viewport_x + viewport.width() < term_width {
                        let viewport_end_column = viewport_column + viewport.width();
                        for row in viewport_row..viewport_end_row {
                            let _ = write!(state.stdout, "\x1B[{};{}H\x1B[0K", row, viewport_end_column);
                        }
                    }

                    if (viewport_y + viewport.height() + 1) / 2 < term_height / 2 {
                        let _ = write!(state.stdout, "\x1B[{};1H\x1B[0J", viewport_end_row);
                    }
                }
            }
        }

        let blend_cycle = (frame_start_ts - loop_start_ts).as_secs_f64();
        if !living_world.timeline().is_empty() {
            let mut palette1 = &living_world.palettes()[living_world.timeline().last().unwrap().palette_index()];
            let mut palette2 = palette1;
            let mut prev_time_of_day = 0;
            let mut next_time_of_day = 0;

            // TODO: binary search?
            let mut found = false;
            for event in living_world.timeline() {
                prev_time_of_day = next_time_of_day;
                next_time_of_day = event.time_of_day() as u64 * 1000;
                palette1 = palette2;
                palette2 = &living_world.palettes()[event.palette_index()];
                if next_time_of_day > time_of_day {
                    found = true;
                    break;
                }
            }

            if !found {
                prev_time_of_day = next_time_of_day;
                next_time_of_day = DAY_DURATION;
                palette1 = palette2;
                palette2 = &living_world.palettes()[living_world.timeline().first().unwrap().palette_index()];
            }

            let current_span = next_time_of_day - prev_time_of_day;
            let time_in_span = time_of_day - prev_time_of_day;
            let blend_palettes = time_in_span as f64 / current_span as f64;

            cycled_palette1.apply_cycles_from(palette1.palette(), palette1.cycles(), blend_cycle, args.blend);
            cycled_palette2.apply_cycles_from(palette2.palette(), palette2.cycles(), blend_cycle, args.blend);

            crate::palette::blend(&cycled_palette1, &cycled_palette2, blend_palettes, &mut blended_palette);

            viewport.indexed_image().apply_with_palette(&mut frame, &blended_palette);
        } else {
            cycled_palette1.apply_cycles_from(&blended_palette, living_world.base().cycles(), blend_cycle, args.blend);
            viewport.indexed_image().apply_with_palette(&mut frame, &cycled_palette1);
        }

        let full_width = viewport.width() >= term_width;
        if full_redraw {
            simple_image_to_ansi_into(&frame, &mut linebuf);
        } else {
            image_to_ansi_into(&prev_frame, &frame, full_width, &mut linebuf);
        }

        std::mem::swap(&mut frame, &mut prev_frame);

        let _ = write!(state.stdout, "\x1B[{};{}H{linebuf}", viewport_row, viewport_column);

        old_term_width  = term_width;
        old_term_height = term_height;

        if state.time_speed != 1 && message.is_empty() {
            let (hours, mins) = get_hours_mins(time_of_day);
            show_message!("{hours}:{mins:02}");
        }

        if message_end_ts >= frame_start_ts {
            if updated_message && old_message_len > message.len() {
                // full redraw next frame by faking old term size of 0x0
                old_term_width  = 0;
                old_term_height = 0;
            } else {
                let msg_len = message.len();

                let column = if msg_len < term_width as usize {
                    (term_width as usize - msg_len) / 2 + 1
                } else { 1 };

                let message = if msg_len > term_width as usize {
                    &message[..term_width as usize]
                } else {
                    &message
                };

                let _ = write!(state.stdout,
                    "\x1B[{};{}H\x1B[38;2;255;255;255m\x1B[48;2;0;0;0m{}",
                    term_height, column, message);
                message_shown = true;
            }
        } else if message_shown {
            // full redraw next frame by faking old term size of 0x0
            old_term_width  = 0;
            old_term_height = 0;
            message_shown = false;
        }

        let _ = state.stdout.flush();

        // sleep for rest of frame
        let elapsed = frame_start_ts.elapsed();
        if frame_duration > elapsed && !interruptable_sleep(frame_duration - elapsed) {
            return Ok(Action::Quit);
        }
    }

    Ok(Action::Quit)
}
