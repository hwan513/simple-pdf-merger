mod files_to_open;
mod merge_files;

use eframe::{run_native, NativeOptions};
use files_to_open::FilesToOpen;

fn main() {
    let native_options = NativeOptions::default();
    run_native(
        "PDF Merger UI",
        native_options,
        Box::new(|_cc| Box::new(FilesToOpen::new())),
    )
    .unwrap();
}
