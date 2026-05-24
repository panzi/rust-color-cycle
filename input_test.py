#!/usr/bin/env python3

import os
import sys
import select
import termios

C_IFLAG = 0
C_OFLAG = 1
C_CFLAG = 2
C_LFLAG = 3
C_LINE  = 4
C_CC    = 6

VTIME = 5
VMIN  = 6

orig_ttyinfo = termios.tcgetattr(0)
print(orig_ttyinfo)
ttyinfo = orig_ttyinfo.copy()
ttyinfo[C_CC] = orig_ttyinfo[C_CC].copy()

ttyinfo[C_IFLAG] |= ~(termios.BRKINT | termios.ICRNL | termios.INPCK | termios.ISTRIP | termios.IXON)
ttyinfo[C_OFLAG] |= termios.ONLCR
ttyinfo[C_LFLAG] |= termios.CS8
ttyinfo[C_LFLAG] &= ~(termios.ECHO | termios.ICANON | termios.IEXTEN | termios.ISIG)
ttyinfo[C_CC][VMIN]  = 0
ttyinfo[C_CC][VTIME] = 0
termios.tcsetattr(0, termios.TCSANOW, ttyinfo)

# enable mouse tracking (click and move)
sys.stdout.write('\x1B[?1000h')
sys.stdout.write('\x1B[?1003h')
sys.stdout.write('\x1B[?1006h')
sys.stdout.flush()

try:
    fp = os.fdopen(0, "rb", buffering=0)
    epoll = select.epoll(flags=select.EPOLL_CLOEXEC)
    epoll.register(0, select.EPOLLIN | select.EPOLLRDHUP)

    running = True
    while running:
        has_data = False

        for efd, eflags in epoll.poll():
            if eflags & select.EPOLLRDHUP:
                running = False

            if eflags & select.EPOLLIN:
                has_data = True

        if has_data and (buf := fp.read()) and buf:
            print(repr(buf))
            if buf == b'\x1B' or buf == b'q':
                break

except KeyboardInterrupt:
    print("^C")

finally:
    # disable mouse tracking (click and move)
    sys.stdout.write('\x1B[?1001l')
    sys.stdout.flush()

    termios.tcsetattr(0, termios.TCSANOW, orig_ttyinfo)
