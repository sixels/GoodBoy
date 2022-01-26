use std::sync::mpsc;

use eframe::{egui, epi};

use crate::{io::IoEvent, utils};

pub struct MenuBar {
    io: mpsc::Sender<IoEvent>,
}

impl MenuBar {
    pub fn new(io_sender: mpsc::Sender<IoEvent>) -> Self {
        Self { io: io_sender }
    }

    pub fn render(&self, ui: &mut egui::Ui, frame: &epi::Frame) {
        let io_sender = self.io.clone();

        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Load ROM File").clicked() {
                    let dialog = rfd::AsyncFileDialog::new()
                        .add_filter("ROM", &["gb", "gbc"])
                        .pick_file();

                    utils::spawn(async move {
                        let file = dialog.await;

                        println!("Loading file: {file:?}");

                        if let Some(file) = file {
                            let buffer = file.read().await;
                            io_sender.send(IoEvent::InsertCartridge(buffer)).ok();
                        }
                    })
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        frame.quit();
                    }
                }
            });
            ui.menu_button("Save", |ui| {
                if ui.button("Quick Save").clicked() {
                    ();
                }
                if ui.button("Quick Load").clicked() {
                    ();
                }
                ui.menu_button("Select Slot", |ui| {
                    for i in 0..10 {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            if ui.button(format!("Slot {i}")).clicked() {
                                ();
                            }
                        });
                    }
                });
            })
        });
    }
}
