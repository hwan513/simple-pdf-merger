use eframe::{
    egui::{Button, CentralPanel, Context, Label, Ui},
    App,
};
use rfd::FileDialog;
use std::path::PathBuf;

pub struct FilesToOpen {
    files: Vec<PathBuf>,
}

impl FilesToOpen {
    pub fn new() -> FilesToOpen {
        FilesToOpen { files: vec![] }
    }

    // TODO See if need to be removed
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

    // TODO See if need to be removed
    // pub fn _display(&self) {
    //     for val in self.files.iter() {
    //         println!("{:?}", val);
    //     }
    // }

    pub fn render_files(&self, ui: &mut Ui) {
        for file in &self.files {
            ui.add(Label::new(file.to_str().unwrap()));
            ui.add_space(5.0);
        }
    }

    pub fn ui_file_drag_drop(&mut self, ctx: &Context) {
        if ctx.input().raw.dropped_files.is_empty() {
            return;
        }
        let dropped_files = &ctx.input().raw.dropped_files.clone();
        for file in dropped_files {
            if let Some(path) = file.clone().path {
                if path.extension().unwrap_or_default() == "pdf" {
                    self.files.push(path);
                }
            }
        }
    }
}

impl App for FilesToOpen {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("PDF Merger");
            ui.add_space(10.0);
            if ui.add(Button::new("Add Files")).clicked() {
                self.open_file();
            }
            ui.add_space(10.0);
            self.render_files(ui);
            self.ui_file_drag_drop(ctx);
        });
    }
}
