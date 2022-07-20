use eframe::{
    egui::{Button, CentralPanel, Context, Label, Layout, ScrollArea, TopBottomPanel, Ui},
    App,
};
use rfd::FileDialog;
use std::{path::PathBuf, usize};

pub struct FilesToOpen {
    files: Vec<PathBuf>,
    save_dir: Option<PathBuf>,
}

impl FilesToOpen {
    pub fn new() -> FilesToOpen {
        FilesToOpen {
            files: vec![],
            save_dir: None,
        }
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

    fn remove_file_from_vec(&mut self, index: usize) {
        self.files.remove(index);
    }

    // TODO See if need to be removed
    // pub fn _display(&self) {
    //     for val in self.files.iter() {
    //         println!("{:?}", val);
    //     }
    // }

    pub fn render_files(&mut self, ui: &mut Ui) {
        // TODO this looks bad
        let mut remove_index: Vec<usize> = vec![];
        for (index, file) in self.files.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new(file.file_name().unwrap().to_str().unwrap()))
                });
                ui.with_layout(Layout::right_to_left(), |ui| {
                    if ui.add(Button::new("Remove")).clicked() {
                        remove_index.push(index);
                    }
                });
            });
            ui.add_space(5.0);
        }
        for index in remove_index {
            self.remove_file_from_vec(index);
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
        self.ui_file_drag_drop(ctx);
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // TODO Extract into function
            ui.add_space(10.0);
            ui.heading("PDF Merger");
            ui.add_space(5.0);
            if ui.add(Button::new("Add Files")).clicked() {
                self.open_file();
            }
            ui.add_space(5.0);
        });
        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    self.render_files(ui);
                });
            ui.add_space(10.0);
        });
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            // TODO Extract into function
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.add(Button::new("Configure Save Path")).clicked() {
                    if let Some(save_dir) = FileDialog::new().set_directory("/").pick_folder() {
                        self.save_dir = Some(save_dir)
                    }
                }
                if let Some(save_dir) = self.save_dir.clone() {
                    ui.add(Label::new(save_dir.to_str().unwrap()));
                }
            });
            ui.add_space(5.0);
        });
    }
}
