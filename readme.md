# Dogky
A custom *Conky*-like *Linux* system monitor written in *Rust*.

This project exists to:

- Replace the following in my *Conky* configuration with *Rust*:
    - Shell commands.
    - Custom *Python* code.
- *Conky* has many *C* macros, e.g. `OBJ` in `src/core.cc`, making things hard to refactor.
- *Conky* attempts to support *X.org*, *MacOS*, introducing lots of code we don't need.

This project isn't meant to be as configurable as *Conky*. Only colors are configurable, in case of changes of desktop background.

# Usage
Copy `src/show-ram-frequency.service` to `/etc/systemd/system/show-ram-frequency.service`. Enable it:

    $ systemctl enable show-ram-frequency.service

# Development Setup
To set up the project for development, run:

    $ cd dogky/
    $ pnpm install conventional-changelog-conventionalcommits
    $ pre-commit install --hook-type pre-commit --hook-type commit-msg

The GUI is designed using *Cambalache*.
