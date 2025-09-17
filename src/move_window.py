#!/usr/bin/env python
import argparse
import inspect
import json
import subprocess
from dataclasses import dataclass
from typing import Any, Self, override


class FromDictMixin:
    @classmethod
    def from_dict(cls, env: dict[str, Any]) -> Self:  # pyright: ignore[reportExplicitAny]
        return cls(
            **{
                k: v
                for k, v in env.items()  # pyright: ignore[reportAny]
                if k in inspect.signature(cls).parameters
            }
        )


@dataclass
class Rect:
    x: int
    y: int
    width: int
    height: int


@dataclass
class Workspace(FromDictMixin):
    rect: Rect
    focused: bool

    @classmethod
    @override
    def from_dict(cls, env: dict[str, Any]) -> Self:  # pyright: ignore[reportExplicitAny]
        env['rect'] = Rect(**env['rect'])  # pyright: ignore[reportAny]
        return super().from_dict(env)


def get_workspace_rect() -> Rect:
    workspaces_str = subprocess.check_output(['swaymsg', '--type', 'get_workspaces'], encoding='utf-8')
    workspaces = [Workspace.from_dict(obj) for obj in json.loads(workspaces_str)] # pyright: ignore[reportAny]
    for workspace in workspaces:
        if workspace.focused:
            return workspace.rect
    raise ValueError


@dataclass
class CurrentMode:
    width: int
    height: int
    refresh: int
    picture_aspect_ratio: str


@dataclass
class Output(FromDictMixin):
    scale: float
    current_mode: CurrentMode
    focused: bool

    @classmethod
    @override
    def from_dict(cls, env: dict[str, Any]) -> Self:  # pyright: ignore[reportExplicitAny]
        env['current_mode'] = CurrentMode(**env['current_mode'])  # pyright: ignore[reportAny]
        return super().from_dict(env)


class ParsedArgs(argparse.Namespace):
    window_width: int  # pyright: ignore [reportUninitializedInstanceVariable]
    window_height: int  # pyright: ignore [reportUninitializedInstanceVariable]


WINDOW_CRITERIA = '[app_id="dogky"]'


def main() -> None:
    arg_parser = argparse.ArgumentParser(description='Move the *Dogky* window in position.')
    _ = arg_parser.add_argument('window_width', type=int)
    args = arg_parser.parse_args(namespace=ParsedArgs)

    outputs_str = subprocess.check_output(['swaymsg', '--type', 'get_outputs'], encoding='utf-8')
    outputs = [Output.from_dict(obj) for obj in json.loads(outputs_str)] # pyright: ignore[reportAny]

    output_width, output_height = 0, 0
    for output in outputs:
        if output.focused:
            output_width, output_height = output.current_mode.width, output.current_mode.height
            output_width = round(output_width / output.scale)
            output_height = round(output_height / output.scale)

    workspace_rect = get_workspace_rect()
    bars_height = output_height - workspace_rect.height # Possibly 0

    output_height -= bars_height
    pos_x = output_width - args.window_width
    pos_y = bars_height

    _ = subprocess.check_call(['swaymsg', ';'.join([
        f'for_window {WINDOW_CRITERIA} resize set {args.window_width} {output_height}',
        f'for_window {WINDOW_CRITERIA} move absolute position {pos_x} {pos_y}',
    ])])


if __name__ == '__main__':
    main()
