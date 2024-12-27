#!/usr/bin/env python
import argparse
import time

from ewmh import EWMH
from Xlib.xobject.drawable import Window


WINDOW_VISIBLE_POLL_INTERVAL = 0.1  # Seconds


def find_window_sync(ewmh: EWMH, win_instance: str, win_class: str) -> Window:
    while True:
        windows = ewmh.getClientList()
        for window in windows:
            assert isinstance(window, Window)
            if ewmh.getWmDesktop(window) == 0xFFFFFFFF:
                continue
            if window.get_wm_class() == (win_instance, win_class):
                return window
        time.sleep(WINDOW_VISIBLE_POLL_INTERVAL)


def main() -> None:
    arg_parser = argparse.ArgumentParser(description='Move the *Dogky* window in position.')
    arg_parser.add_argument('monitor_width', type=int)
    arg_parser.add_argument('monitor_height', type=int)
    arg_parser.add_argument('window_width', type=int)
    arg_parser.add_argument('window_height', type=int)

    args = arg_parser.parse_args()

    ewmh = EWMH()
    window = find_window_sync(ewmh, 'dogky', 'dogky')
    for state in [
        '_NET_WM_STATE_STICKY',
        '_NET_WM_STATE_SKIP_TASKBAR',
        '_NET_WM_STATE_SKIP_PAGER',
        '_NET_WM_STATE_BELOW',
    ]:
        ewmh.setWmState(window, 1, state)
    ewmh.setMoveResizeWindow(
        window, gravity=0, x=args.monitor_width - args.window_width, y=0, w=args.window_width, h=args.window_height
    )
    ewmh.display.flush()


if __name__ == '__main__':
    main()
