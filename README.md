# Description

simple-pdf-merger is a desktop app that can merge multiple PDF's together with simple GUI.
This app is mainly intended for personal use and as such has hard coded duplicate page remove functionality.

## Buliding from source

This app requires rust to first compile the app

### Instructions

1. Clone this app using `git clone https://github.com/hwan513/simple-pdf-merger.git && cd simple-pdf-merger`
2. Compile the application using `cargo build --release`
3. Run the application using `cargo run --release`

Alternatively after step 2, you can copy the standalone executable located at `./target/release/simple-pdf-merger` to anywhere on your system.
Note on macOS: you can prevent a terminal instance from launching when opening the app by adding the `.app` file extension to the executable.
