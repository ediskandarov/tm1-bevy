# Building and testing

Bevy is pinned to **0.20.1**. The development toolchain is Rust **1.97.1**,
the minimum required by this Bevy release and its WESL dependencies.
The two platform-independent crates have separate lockfiles so their tests can
run without fetching the renderer dependencies.

## Build from source

Run these commands from the repository root. Cargo also compiles the C simulation with the host C compiler.

```sh
rustup toolchain install 1.97.1 --profile minimal --component rustfmt
cargo fetch --locked
cargo build --release --locked
```

Copy the resulting `target/release/tm1-bevy` into the disc directory and launch it. See the [player guide](playing.md) for CUE discovery and command-line options.

## Test and inspect the disc

```sh
cargo test --release --locked
cargo test --release --locked --manifest-path crates/tm1-assets/Cargo.toml
cargo test --release --locked --manifest-path crates/tm1-core/Cargo.toml
cargo run --locked --manifest-path crates/tm1-assets/Cargo.toml --bin inspect -- /absolute/path/to/disc.cue
TM1_CUE=/absolute/path/to/disc.cue cargo test --release --locked --manifest-path crates/tm1-assets/Cargo.toml --test disc_revision -- --ignored
```

The real-disc test is opt-in and requires an absolute CUE path. No game assets
are included in the repository. The source headers and parser layouts are based
on the recorded decompilation revision.

The driving comparison runs in release mode against verified player-owned bytes.

```sh
TM1_CUE=/absolute/path/to/disc.cue cargo test --release --locked --manifest-path crates/tm1-core/Cargo.toml -- --include-ignored --nocapture
```

## CI and GitHub Releases

Each push to `main` runs the application, asset reader and game core tests in
release mode. Tests marked as requiring the original disc are skipped on CI.

Pushing any Git tag runs the same tests and, on success, publishes a GitHub
Release for that tag. Download `tm1-bevy-macos-arm64.tar.gz` for Apple Silicon
or `tm1-bevy-windows-x64.zip` for 64-bit Intel or AMD Windows PCs. Each archive
contains the executable. Supply your own supported BIN/CUE files when playing.

[GitHub Actions workflow](../.github/workflows/ci.yml)
