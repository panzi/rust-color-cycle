#!/usr/bin/env python3

import os
import termios

C_IFLAG = 0
C_OFLAG = 1
C_CFLAG = 2
C_LFLAG = 3
C_LINE  = 4
C_CC    = 6

VTIME = 5
VMIN  = 6

ttyinfo = termios.tcgetattr(0)
print(ttyinfo)
ttyinfo[C_IFLAG] |= ~(termios.BRKINT | termios.ICRNL | termios.INPCK | termios.ISTRIP | termios.IXON)
ttyinfo[C_OFLAG] |= termios.ONLCR
ttyinfo[C_LFLAG] |= termios.CS8
ttyinfo[C_LFLAG] &= ~(termios.ECHO | termios.ICANON | termios.IEXTEN | termios.ISIG)
ttyinfo[C_CC][VMIN]  = 0
ttyinfo[C_CC][VTIME] = 0
termios.tcsetattr(0, termios.TCSANOW, ttyinfo)

try:
    fp = os.fdopen(0, "rb", buffering=0)
    while True:
        buf = fp.read()
        if buf:
            print(repr(buf))
except KeyboardInterrupt:
    print("^C")

finally:
    ttyinfo[3] |= termios.ICANON | termios.ECHO
    termios.tcsetattr(0, termios.TCSANOW, ttyinfo)
