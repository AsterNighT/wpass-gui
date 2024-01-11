use egui::{Align2, Color32, DroppedFile, Id, LayerId, Order, TextStyle, Vec2};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum ExtractionMode {
    Local,
    NewDirectory,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum MenuState {
    Main,
    Setting,
    Password,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct WpassApp {
    password_file_path: String,
    archive_executable_path: String,
    extraction_mode: ExtractionMode,
    delete_after_extract: bool,
    sanitize: bool,
    #[serde(skip)] // This how you opt-out of serialization of a field
    menu_state: MenuState,
    #[serde(skip)] // This how you opt-out of serialization of a field
    passwords: Option<String>,
}

impl Default for WpassApp {
    fn default() -> Self {
        Self {
            password_file_path: String::new(),
            archive_executable_path: String::new(),
            extraction_mode: ExtractionMode::Local,
            delete_after_extract: false,
            sanitize: true,
            menu_state: MenuState::Main,
            passwords: None,
        }
    }
}

impl WpassApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn generate_reg(&self) {}
    fn update_passwords_from_file(&mut self) {
        // This should happen only when password menu is clicked.
        let password_file_pathbuf = PathBuf::from(&self.password_file_path);
        if Path::is_file(&password_file_pathbuf) {
            // A chance is that the password is retrieved and updated. In this function we always assume the password file is the most up-to-date. The memory-to-file overwrite happens elsewhere.
            self.passwords = Some(fs::read_to_string(&password_file_pathbuf).unwrap());
        } else {
            self.passwords = None;
        }
    }

    fn update_passwords_to_file(&mut self) {
        // Let filesystem handle the file sync issue.
        // Start a new thread to do the job? 
        let password_file_pathbuf = PathBuf::from(&self.password_file_path);
        if Path::is_file(&password_file_pathbuf) && self.passwords.is_some() {
            fs::write(&password_file_pathbuf, self.passwords.as_ref().unwrap()).unwrap();
        }
    }

    fn handle_files(&mut self, files: &Vec<DroppedFile>) {
        
    }
}

impl eframe::App for WpassApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                if ui.button("Main").clicked() {
                    self.update_passwords_to_file();
                    self.menu_state = MenuState::Main;
                }
                if ui.button("Settings").clicked() {
                    self.menu_state = MenuState::Setting;
                }
                if ui.button("Passwords").clicked() {
                    self.update_passwords_from_file();
                    self.menu_state = MenuState::Password;
                }
                if cfg!(windows) {
                    if ui.button("Install rightclick menu").clicked() {
                        self.generate_reg();
                    }
                }
            });
        });
        match self.menu_state {
            MenuState::Main => {
                if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        // The central panel the region left after adding TopPanel's and SidePanel's
                        // Preview hovering files:
                        ui.centered_and_justified(|ui| {
                            ui.label("Release here");
                        });
                    });
                    let text = ctx.input(|i| {
                        let mut text = "Dropping files:\n".to_owned();
                        for file in &i.raw.hovered_files {
                            if let Some(path) = &file.path {
                                write!(text, "\n{}", path.display()).ok();
                            } else if !file.mime.is_empty() {
                                write!(text, "\n{}", file.mime).ok();
                            } else {
                                text += "\n???";
                            }
                        }
                        text
                    });

                    let painter = ctx.layer_painter(LayerId::new(
                        Order::Foreground,
                        Id::new("file_drop_target"),
                    ));

                    let screen_rect = ctx.screen_rect();
                    painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
                    painter.text(
                        screen_rect.center(),
                        Align2::CENTER_CENTER,
                        text,
                        TextStyle::Body.resolve(&ctx.style()),
                        Color32::WHITE,
                    );
                } else {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        // The central panel the region left after adding TopPanel's and SidePanel's
                        // Preview hovering files:
                        ui.centered_and_justified(|ui| {
                            ui.label("Drag & Drop a file here");
                        });
                    });
                }
                ctx.input(|i| {
                    if !i.raw.dropped_files.is_empty() {
                        self.handle_files(&i.raw.dropped_files);
                    }
                });
            }
            // Showing settings
            MenuState::Setting => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // The central panel the region left after adding TopPanel's and SidePanel's
                    egui::Grid::new("my_grid")
                        .num_columns(2)
                        .spacing([20.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Path to password file:");
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    ui.available_size() - Vec2::new(60.0, 0.0),
                                    egui::TextEdit::singleline(&mut self.password_file_path),
                                );
                                if ui.button("Browse").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                                        self.password_file_path = path.display().to_string();
                                    }
                                }
                            });
                            ui.end_row();
                            ui.label("Path to 7z executable:");
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    ui.available_size() - Vec2::new(60.0, 0.0),
                                    egui::TextEdit::singleline(&mut self.archive_executable_path),
                                );
                                if ui.button("Browse").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                                        self.archive_executable_path = path.display().to_string();
                                    }
                                }
                            });
                            ui.end_row();
                            ui.label("Extraction destination:");
                            ui.vertical(|ui| {
                                ui.radio_value(
                                    &mut self.extraction_mode,
                                    ExtractionMode::Local,
                                    "Extract to the same directory",
                                );
                                ui.radio_value(
                                    &mut self.extraction_mode,
                                    ExtractionMode::NewDirectory,
                                    "Extract to a new directory",
                                );
                            });
                            ui.end_row();
                            ui.label("Delete archive file:");
                            ui.checkbox(&mut self.delete_after_extract, "");
                            ui.end_row();
                            ui.label("Sanitize password file:");
                            ui.checkbox(&mut self.sanitize, "");
                            ui.end_row();
                        });
                });
            }
            MenuState::Password => {
                // Check if password file is set
                if let Some(passwords) = &mut self.passwords {
                    // Ok, file exists, read the file and port into a textbox
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.add_sized(ui.available_size(), egui::TextEdit::multiline(passwords));
                    });
                } else {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        // The central panel the region left after adding TopPanel's and SidePanel's
                        ui.centered_and_justified(|ui| {
                            ui.label("Invalid password file, check it in settings.");
                        });
                    });
                }
            }
        }

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::warn_if_debug_build(ui);
            powered_by_egui_and_eframe_and_lint_to_github(ui);
        });
    }
}

fn powered_by_egui_and_eframe_and_lint_to_github(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(". ");
        ui.label("The source code can be found at ");
        ui.hyperlink_to("wpass-gui", "https://github.com/asternight/wpass-gui");
    });
}
