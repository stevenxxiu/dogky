#!/usr/bin/env python
import argparse
import json
import subprocess
from typing import TypedDict


class Rect(TypedDict):
    x: int
    y: int
    width: int
    height: int


class Workspace(TypedDict):
    rect: Rect
    focused: bool


def get_workspace_rect() -> Rect:
    workspaces_str = subprocess.check_output(['swaymsg', '--type', 'get_workspaces'], encoding='utf-8')
    workspaces: list[Workspace] = json.loads(workspaces_str)  # pyright: ignore[reportAny]
    for workspace in workspaces:
        if workspace['focused']:
            rect: Rect = workspace['rect']
            return rect
    raise ValueError


class CurrentMode(TypedDict):
    width: int
    height: int
    refresh: int
    picture_aspect_ratio: str


class Output(TypedDict):
    scale: float
    current_mode: CurrentMode
    focused: bool


class ParsedArgs(argparse.Namespace):
    window_width: int  # pyright: ignore [reportUninitializedInstanceVariable]
    window_height: int  # pyright: ignore [reportUninitializedInstanceVariable]


WINDOW_CRITERIA = '[app_id="dogky"]'


def main() -> None:
    arg_parser = argparse.ArgumentParser(description='Move the *Dogky* window in position.')
    _ = arg_parser.add_argument('window_width', type=int)
    args = arg_parser.parse_args(namespace=ParsedArgs)

    outputs_str = subprocess.check_output(['swaymsg', '--type', 'get_outputs'], encoding='utf-8')
    outputs: list[Output] = json.loads(outputs_str) # pyright: ignore[reportAny]

    output_width, output_height = 0, 0
    for output in outputs:
        if output['focused']:
            output_width = round(output['current_mode']['width'] / output['scale'])
            output_height = round(output['current_mode']['height'] / output['scale'])

    workspace_rect = get_workspace_rect()
    bars_height = output_height - workspace_rect['height'] # Possibly 0

    output_height -= bars_height
    pos_x = output_width - args.window_width
    pos_y = bars_height

    _ = subprocess.check_call(['swaymsg', ';'.join([
        f'for_window {WINDOW_CRITERIA} resize set {args.window_width} {output_height}',
        f'for_window {WINDOW_CRITERIA} move absolute position {pos_x} {pos_y}',
    ])])


if __name__ == '__main__':
    main()
