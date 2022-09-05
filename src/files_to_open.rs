use crate::merge_files;
use eframe::{
    egui::{self, Button, CentralPanel, Context, Label, Layout, ScrollArea, TopBottomPanel, Ui},
    emath::Align,
    epaint::Vec2,
    App,
};
use egui_toast::Toasts;
use rfd::FileDialog;
use std::{mem, ops::Add, path::PathBuf, time::Duration, usize};

pub struct FilesToOpen {
    files: Vec<PathBuf>,
    save_path: Option<PathBuf>,
}

impl FilesToOpen {
    pub fn new() -> FilesToOpen {
        FilesToOpen {
            files: vec![],
            save_path: None,
        }
    }

    pub fn open_file(&mut self) {
        if let Some(mut files) = FileDialog::new()
            .add_filter("pdf", &["pdf"])
            .set_directory("/Users/henrywang/Documents/")
            .pick_files()
        {
            self.files.append(&mut files);
        }
    }

    pub fn render_files(&mut self, ui: &mut Ui) {
        // TODO this looks bad
        // TODO Make reorderable paths
        let mut remove_index: Vec<usize> = vec![];
        for (index, file) in self.files.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                    ui.add(Label::new(file.file_name().unwrap().to_str().unwrap()))
                });
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    if ui.add(Button::new("Remove")).clicked() {
                        remove_index.push(index);
                    }
                });
            });
            ui.add_space(5.0);
        }
        for index in remove_index {
            self.files.remove(index);
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
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::style::Visuals::dark());
        self.ui_file_drag_drop(ctx);

        let mut toasts = Toasts::new(ctx)
            .anchor(
                frame
                    .info()
                    .window_info
                    .size
                    .add(Vec2::new(-10.0, -10.0))
                    .to_pos2(),
            )
            .direction(egui::Direction::BottomUp)
            .align_to_end(true);

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // TODO Extract into function
            ui.add_space(10.0);
            ui.heading("PDF Merger");
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                    if ui.add(Button::new("Add Files")).clicked() {
                        self.open_file();
                    }
                });
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    if ui.add(Button::new("Merge Files")).clicked() {
                        if self.files.is_empty() {
                            return;
                        }
                        if self.save_path.is_none() {
                            self.save_path =
                                FileDialog::new().add_filter("pdf", &["pdf"]).save_file();
                        }
                        if self.save_path.is_some() {
                            toasts.info(
                                format!(
                                    "Merging {} pdf documents. Creating {}",
                                    self.files.len(),
                                    &self
                                        .save_path
                                        .clone()
                                        .unwrap()
                                        .file_name()
                                        .unwrap()
                                        .to_str()
                                        .unwrap()
                                ),
                                Duration::from_secs(10),
                            );
                            merge_files::start(
                                mem::take(&mut self.files),
                                mem::take(&mut self.save_path).unwrap(),
                            );
                        }
                        // if let Some(save_path) = self.save_path.clone() {
                        //     merge_files::start(mem::take(&mut self.files), save_path);
                        // }
                    }
                });
                toasts.show();
            });
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
                        self.save_path = Some(save_dir)
                    }
                }
                if let Some(save_path) = self.save_path.clone() {
                    ui.add(Label::new(save_path.to_str().unwrap()));
                }
            });
            ui.add_space(5.0);
        });
    }
}
