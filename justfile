clean:
    find target/ -maxdepth 1 -type f -delete
    find target/debug/ -mindepth 1 -delete
    find target/release/ \
        -mindepth 1 -maxdepth 1 \
        -not \( -name dogky -prune \) \
        -exec rm -rf {} +

build-debug:
    cargo build

build-release:
    cargo build --release

main-debug:
    cargo run

main-release:
    cargo run --release
