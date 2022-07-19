use rfd::FileDialog;
use std::path::PathBuf;

pub struct FilesToOpen {
    files: Vec<PathBuf>,
}

impl FilesToOpen {
    pub fn new() -> FilesToOpen {
        FilesToOpen { files: vec![] }
    }

    // fn add(&mut self, new_files: &mut Vec<PathBuf>) {
    //     self.files.append(new_files)
    // }

    pub fn open_file(&mut self) {
        if let Some(mut files) = FileDialog::new()
            .add_filter("pdf", &["pdf"])
            .set_directory("/")
            .pick_files()
        {
            self.files.append(&mut files);
        }
    }

    pub fn display(&self) {
        for val in self.files.iter() {
            println!("{:?}", val);
        }
    }
}
