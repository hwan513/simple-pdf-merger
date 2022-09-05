# Description

simple-pdf-merger is a desktop app that can merge multiple PDF's together into one file with an easy to use GUI.
I created this app for personal use and to consolidate my skills programming in Rust.
As such the PDF merging capabilities are limited and the app isn't the most polished.
There is an option to remove duplicate pages in the merged PDF which can be useful for content like lecture slides which may have repeated pages.

## Using the app

Since this app is for personal use, I haven't put much thought into distribution.
If you want to run this app for yourself make sure you have the latest version of rust installed.
See https://rustup.rs for more information.

### Instructions

1. `git clone https://github.com/hwan513/simple-pdf-merger.git`
2. `cd simple-pdf-merger`
3. `cargo run --release`

This will compile and run the app.
