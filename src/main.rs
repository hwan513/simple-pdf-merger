mod files_to_open;

use files_to_open::FilesToOpen;

fn main() {
    let mut files = FilesToOpen::new();
    files.open_file();
    files.display();
}
