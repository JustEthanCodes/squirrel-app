This is a stupid app I made that it literally just an image of an unclosable squirrel.
Features:

Added Squirrel
thats it
Oh, and don't worry! A pro version is coming soon with way more features.

_Note: windows defender says its dangerous, but its not. I made it myself and it's open source so you can even see the code_

























































(Please use this responsibly. Also it _is_ closable via task manager and by pressing `F9`.

## Linux and ARM64

Linux builds are supported on x86_64 and ARM64 (`aarch64-unknown-linux-gnu`). Windows ARM64 builds are also supported and produced by the `Build Windows Executable` GitHub Actions workflow as the `bradley-windows-arm64-exe` artifact. On Debian or Ubuntu, install the desktop build dependencies first:

```sh
sudo apt install build-essential pkg-config libgtk-3-dev libxdo-dev libappindicator3-dev
```

Then build or run with Cargo:

```sh
cargo build --release
cargo run --release --bin bradley
```

The `please help me` helper uses `pkill` on Linux (provided by `procps` on most distributions). The October 9 birthday behavior uses the system's local date. On Wayland, the compositor may restrict window positioning or always-on-top behavior; X11 provides the fullest support for bouncing and positioning.

## macOS

macOS is supported on Intel (`x86_64`) and Apple Silicon (`arm64`). The `Build macOS executables` GitHub Actions workflow produces `bradley-macos-x86_64` and `bradley-macos-arm64` artifacts. The `please help me` helper uses macOS's `pkill` command.
