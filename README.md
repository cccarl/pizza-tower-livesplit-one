# Pizza Tower Livesplit One Autosplitter

To be used with LiveSplit's WASM runtime. Can be downloaded in releases or build it yourself.

## Features:

* Full game autosplitter mode, splits when exiting the last room of each level, starts when starting a new file and resets when going back to the main menu.
* IL autosplitter mode, starts when entering any level, resets on level restarts or going back to the hub, and splits on every new room the player enters. 
* Reads the game time of the game, also compatible with the Speedrun IGT Mod: https://gamebanana.com/mods/445080
* Robust against game updates.

## How to add to LiveSplit:

1. Right Click.
2. Edit Layout...
3. \+ Button -> Control -> Auto Splitting Runtime.
4. Open the added component and look for the WASM file using the file explorer at the top of the window.

Note: WAR's final split is -0.21 seconds late, an offset could fix that.


# Building the WASM file

To build you need to add to the rust toolchain:

* `$ rustup target add wasm32-unknown-unknown`

Recommended to use cargo watch while developing:

* `cargo watch -x "build --target wasm32-unknown-unknown"`

To build for release:

* `$ cargo build --release --target wasm32-unknown-unknown`

You can find the resulting WASM file in the target forlder.
