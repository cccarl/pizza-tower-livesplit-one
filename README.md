# Pizza Tower Livesplit One Autosplitter

Experimental Autosplitter for Pizza Tower using the new autosplitting runtime for LiveSplit One and LiveSplit. If you only want to use it for full game runs with the original LiveSplit, it's recommend to simply use the official AutoSplitter that can be found in the "Edit Splits" menu instead!

## Features:

* 4 Game Time modes for LiveSplit: Full Game, Individual Level, New Game+ and Individual World. Remember to use the launch option "-livesplit" in Pizza Tower for this!
* Customizable start, split and reset events using the new GUI for the autosplitting runtime.
* Tick Rate of 240hz, ASL splitters struggle to keep up with a 60hz tick rate.

## How to add to LiveSplit:

1. Right Click.
2. Edit Layout...
3. \+ Button -> Control -> Auto Splitting Runtime.
4. Open the added component and look for the WASM file using the file explorer at the top of the window.


# Building the WASM file

To build you need to add to the rust toolchain:

* `$ rustup target add wasm32-unknown-unknown`

Recommended to use cargo watch while developing:

* ` $ cargo watch -x "build --target wasm32-unknown-unknown"`

To build for release (also works with `cargo watch`):

* `$ cargo build --release --target wasm32-unknown-unknown`

You can find the resulting WASM file in the target forlder.
