#!/usr/bin/env python
import argparse
import inspect
import json
import subprocess
from dataclasses import dataclass
from typing import Any


class ParsedArgs(argparse.Namespace):
    window_width: int  # pyright: ignore [reportUninitializedInstanceVariable]
    window_height: int  # pyright: ignore [reportUninitializedInstanceVariable]


@dataclass
class CurrentMode:
    width: int
    height: int
    refresh: int
    picture_aspect_ratio: str


@dataclass
class Output:
    scale: float
    current_mode: CurrentMode
    focused: bool

    @classmethod
    def from_dict(cls, env: dict[str, Any]):  # pyright: ignore[reportExplicitAny]
        env['current_mode'] = CurrentMode(**env['current_mode'])  # pyright: ignore[reportAny]
        return cls(
            **{  # pyright: ignore[reportAny]
                k: v
                for k, v in env.items()  # pyright: ignore[reportAny]
                if k in inspect.signature(cls).parameters
            }
        )


WAYBAR_HEIGHT = 34
WINDOW_CRITERIA = '[title="Freya App"]'

def main() -> None:
    arg_parser = argparse.ArgumentParser(description='Move the *Dogky* window in position.')
    _ = arg_parser.add_argument('window_width', type=int)
    args = arg_parser.parse_args(namespace=ParsedArgs)

    outputs_str = subprocess.check_output(['swaymsg', '--type', 'get_outputs'], encoding='utf-8')
    outputs = [Output.from_dict(obj) for obj in json.loads(outputs_str)] # pyright: ignore[reportAny]

    width, height = 0, 0
    waybar_height = WAYBAR_HEIGHT
    for output in outputs:
        if output.focused:
            width, height = output.current_mode.width, output.current_mode.height
            width = round(width / output.scale)
            height = round(height / output.scale)
            waybar_height = round(waybar_height / output.scale)

    height -= waybar_height
    pos_x = width - args.window_width
    pos_y = waybar_height

    _ = subprocess.check_call(['swaymsg', ';'.join([
        f'for_window {WINDOW_CRITERIA} resize set {args.window_width}',
        f'for_window {WINDOW_CRITERIA} resize set height {height}',
        f'for_window {WINDOW_CRITERIA} move absolute position {pos_x} {pos_y}',
    ])])


if __name__ == '__main__':
    main()
