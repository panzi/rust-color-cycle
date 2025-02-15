# A Color Cycle Viewer for the terminal

Color Cycling is a technique to render images with color palette based
animations. It was used in 90ies video games. This program renders such
images to Unicode capable ANSI terminals. Windows is not supported, but
I'd accept a pull request for that.

This implementation only supports a the background layer (no overlays)
including time of day shifts, but no time based events (for now, maybe
I'll add that at some later time).

This viewer reads [Living Worlds Maker](https://magrathea.onrender.com/)
files (only the background layer) or JSON files similar to what the
[Canvas Cycle](https://experiments.withgoogle.com/canvas-cycle) demo
by Joseph Huckaby uses.

[Short Demo Video](https://www.youtube.com/watch?v=QMQ93uL1Fhk)

## Usage

```
Usage: color-cycle [OPTIONS] <PATH>...

Arguments:
  <PATH>...
          Path to a Canvas Cycle JSON file

Options:
  -f, --fps <FPS>
          Frames per second.

          Attempt to render in this number of frames per second. Actual FPS might be lower.

          [default: 25]

  -b, --blend
          Enable blend mode.

          This blends the animated color palette for smoother display.

  -o, --osd
          Enable On Screen Display.

          Displays messages when changing things like blend mode or FPS.


      --help-hotkeys
          Show list of hotkeys

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Hotkeys

| Hotkey | Description |
| :----- | :---------- |
| `B` | Toggle blend mode |
| `Q` or `Escape` | Quit program |
| `O` | Toggle On Screen Display of message |
| `N` | Open next file |
| `P` | Open previous file |
| `1` to `9` | Open file by index |
| `0` | Open last file |
| `+` | Increase frames per second by 1 |
| `-` | Decrease frames per second by 1 |
| `W` | Toogle fast forward (10000x speed). |
| `A` | Go back in time by 5 minutes. |
| `D` | Go forward in time by 5 minutes. |
| `S` | Go to current time and continue normal progression. |
| `Cursor Up` | Move view-port up by 1 pixel |
| `Cursor Down` | Move view-port down by 1 pixel |
| `Cursor Left` | Move view-port left by 1 pixel |
| `Cursor Right` | Move view-port right by 1 pixel |
| `Home` | Move view-port to left edge |
| `End` | Move view-port to right edge |
| `Ctrl`+`Home` | Move view-port to top |
| `Ctrl`+`End` | Move view-port to bottom |
| `Page Up` | Move view-port up by half a screen |
| `Page Down` | Move view-port down by half a screen |
| `Alt`+`Page Up` | Move view-port left by half a screen |
| `Alt`+`Page Down` | Move view-port right by half a screen |

## Related Projects

Other things I made that render Uinocde characters to the terminal:

- [Progress Pride Bar](https://github.com/panzi/progress-pride-bar) (Rust): A
  progress bar for the terminal that looks like the progress pride flag.
- [Term Flags](https://github.com/panzi/python-term-flags) (Python): A primitive
  sytem to render simple scalable flags on the terminal using Unicode.
- [Bad Apple!! but its the Unix Terminal](https://github.com/panzi/bad-apple-terminal)
  (C): A program that displays the Bad Apple!! animation on the terminal.
- [ANSI IMG](https://github.com/panzi/ansi-img) (Rust): Display images (including
  animated GIFs) on the terminal.
- [Unicode Bar Charts](https://github.com/panzi/js-unicode-bar-chart)
  (JavaScript): Draw bar charts on the terminal. With 8 steps per character and
  with colors.
- [Unicode Progress Bars](https://github.com/panzi/js-unicode-progress-bar)
  (JavaScript): Draw bar charts on the terminal. With 8 steps per character,
  border styles, and colors.
- [Unicode Unicode Plots](https://github.com/panzi/js-unicode-plot) (JavaScript):
  Very simple plotting on the terminal. No colors.
