clean:
    find target/ -maxdepth 1 -type f -delete
    find target/debug/ -mindepth 1 -delete
    find target/release/ \
        -not \( -name dogky -prune \) \
        -not \( -name move_window.py -prune \) \
        -mindepth 1 -maxdepth 1 -exec rm -rf {} +

build-debug:
    cargo build

build-release:
    cargo build --release

main-debug:
    cargo run

main-release:
    cargo run --release
