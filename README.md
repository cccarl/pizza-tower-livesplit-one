# Pizza Tower Livesplit One Autosplitter

To be used with LiveSplit's WASM runtime. Can be downloaded in releases or build it yourself.

Recommendation: Start IL runs with an offset of -0.21 and full game runs with an offset of -0.35 for better accuracy. Note that it resets by reading the score going back to 0, which might be inconvenient in levels like the tutorial.

To do: share splits so that you don't have to guess how many you will need for ILs. It's usually around 20, I recommend having more than needed, do a run, then remove the remaining empty splits.

To build you need to add to the rust toolchain:

* `$ rustup target add wasm32-unknown-unknown`

Recommended to use cargo watch while developing:

* `cargo watch -x "build --target wasm32-unknown-unknown"`

To build for release:

* `$ cargo build --release --target wasm32-unknown-unknown`

You can find the resulting WASM file in the target forlder.
