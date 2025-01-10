# Dogky
A custom *Conky*-like *Linux* system monitor written in *Rust*.

## Aim
This project exists to replace my *Conky* configuration.

Issues with *Conky*:

- My configuration requires:
    - Shell commands for many parts that *Conky* macros do not cover.
    - Custom *Python* code for the weather panel.
- *Conky* has many *C* macros, e.g. `OBJ` in `src/core.cc`, making things hard to refactor.
- *Conky* attempts to support *X.org*, *MacOS*. This introduces lots of code we don't need.

## Configuration
- `~/.config/dogky/dogky.yaml`
    - See `src/config.rs` for options.
    - Commands, e.g. `cpu_memory.process_list.top_command`, have environment variable support.
- `~/.config/dogky/styles.yaml`
    - See `src/styles_config.rs` for options.

The UI layout isn't configurable.

## Usage
Copy `src/show-ram-frequency.service` to `/etc/systemd/system/show-ram-frequency.service`. Enable it:

    $ systemctl enable show-ram-frequency.service

The UI is clickable. The cursor icon changes where this applies.

- Weather panel
    - Opens the weather forecast in a browser.
- Process list
    - Run a user-specified command. The intention is to launch some version of *Top*.
- Hardware names
    - Click to copy.

## Development Setup
To set up the project for development, run:

    $ cd dogky/
    $ pnpm install conventional-changelog-conventionalcommits
    $ pre-commit install --hook-type pre-commit --hook-type commit-msg
