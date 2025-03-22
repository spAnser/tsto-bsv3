# TSTO BSV3 Tool

## Download

1. Check the [releases](https://github.com/spAnser/tsto-bsv3/releases) page for the latest binaries.
2. The [bsv3.bt](./bsv3.bt) file is an 010 Editor file defining the BSV3 format.

## Usage

1. Drag & drop a BSV3/RGB file onto the window. It expects a matching BSV3 & RGB file in the same folder.
2. Left click and drag to pan the image.
3. Right click to cycle through animations.
4. Mouse wheel to zoom in/out.
5. Press `F` to freeze the current animation.
6. Press `B` to toggle the background color between gray, green, and blue.

## Setup

1. You may need to copy the `SDL2.dll` in the `./SDL2/` folder into `./target/debug/` & `./target/release/`.
2. `cargo run` or `cargo build --release`
