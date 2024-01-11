use crate::worker::TaskDisplayer;
use crate::worker::{Showcase, Task};
use egui::{Align2, Color32, DroppedFile, Id, LayerId, Order, TextStyle, Vec2};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::{
    fs,
    path::{Path, PathBuf},
};
use wpass::{WPass, WPassInstance};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    password_file_path: String,
    archive_executable_path: String,
    extraction_mode: ExtractionMode,
    delete_after_extract: bool,
    sanitize: bool,
}

impl AppConfig {
    pub fn calculate_output_path_for(&self, path: &PathBuf) -> PathBuf {
        match self.extraction_mode {
            ExtractionMode::Local => {
                let mut output_path = path.clone();
                output_path.pop();
                if !output_path.is_dir() {
                    output_path.push(".");
                }
                output_path
            }
            ExtractionMode::NewDirectory => {
                let mut output_path = path.clone().with_extension("");
                if output_path.exists() {
                    output_path.pop();
                    output_path.push(format!(
                        "{}_extracted",
                        path.file_stem().unwrap().to_str().unwrap()
                    ));
                }
                output_path
            }
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct WPassApp {
    config: AppConfig,
    #[serde(skip)]
    menu_state: MenuState,
    #[serde(skip)]
    passwords: Option<String>,
    #[serde(skip)]
    task_showcase: Showcase<bool>,
}

impl Default for WPassApp {
    fn default() -> Self {
        Self {
            config: AppConfig {
                password_file_path: String::new(),
                archive_executable_path: String::new(),
                extraction_mode: ExtractionMode::Local,
                delete_after_extract: false,
                sanitize: true,
            },
            menu_state: MenuState::Main,
            passwords: None,
            task_showcase: Showcase::new(),
        }
    }
}

impl WPassApp {
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

    pub fn init(&mut self) {
        debug!("Initializing app");
        self.update_passwords_from_file();
    }

    fn update_passwords_from_file(&mut self) {
        debug!("Updating passwords");
        let password_file_pathbuf = PathBuf::from(&self.config.password_file_path);
        if Path::is_file(&password_file_pathbuf) {
            self.passwords = Some(fs::read_to_string(&password_file_pathbuf).unwrap());
        } else {
            self.passwords = None;
        }
        debug!("Passwords updated to {:?}", self.passwords);
    }

    fn update_passwords_to_file(&mut self) {
        debug!("Writing passwords to {}", self.config.password_file_path);
        // Let filesystem handle the file sync issue.
        // Start a new thread to do the job?
        let password_file_pathbuf = PathBuf::from(&self.config.password_file_path);
        if Path::is_file(&password_file_pathbuf) && self.passwords.is_some() {
            fs::write(&password_file_pathbuf, self.passwords.as_ref().unwrap()).unwrap();
        }
    }

    fn try_sanitize_passwords(&mut self) {
        debug!("Sanitizing passwords");
        if self.config.sanitize {
            if let Some(passwords) = &mut self.passwords {
                let mut dict = passwords.split('\n').map(|s| s.trim()).collect::<Vec<_>>();
                dict.sort();
                dict.dedup();
                *passwords = dict.join("\n");
            }
        }
    }

    fn schedule_files(&mut self, files: &Vec<DroppedFile>) {
        let current_config = self.config.clone();
        let password_dict = if self.passwords.is_none() {
            debug!("No password file set, will try to express with dummy passwords");
            vec!["dummy".to_owned()]
        } else {
            debug!("Using password file");
            self.passwords
                .as_ref()
                .unwrap()
                .split('\n')
                .map(|s| s.trim().to_owned())
                .collect::<Vec<_>>()
        };
        files.iter().for_each(|file| {
            if let Some(path) = &file.path {
                debug!("Extracting file {:?}", path);
                let path = path.clone();
                let current_config = current_config.clone();
                let password_dict = password_dict.clone();
                let task = Task::new(path.display().to_string(), move || {
                    let wpass = WPassInstance::new(
                        password_dict,
                        current_config.archive_executable_path.clone().into(),
                    );
                    let output = current_config.calculate_output_path_for(&path);
                    let extract_result = wpass.try_extract(&path, &output);
                    match &extract_result {
                        Ok(_) => {
                            debug!("Extracted file {:?} to {:?}", path, output);
                            if current_config.delete_after_extract {
                                debug!("Deleting file {:?}", path);
                                fs::remove_file(path).unwrap();
                            }
                        }
                        Err(e) => {
                            debug!("Failed to extract file {:?}: {}", path, e);
                        }
                    }
                    extract_result
                });
                self.task_showcase.display(task);
            }
        });
    }
}

impl eframe::App for WPassApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                if ui.button("Main").clicked() {
                    self.menu_state = MenuState::Main;
                }
                if ui.button("Settings").clicked() {
                    self.menu_state = MenuState::Setting;
                }
                if ui.button("Passwords").clicked() {
                    self.menu_state = MenuState::Password;
                }
            });
        });
        match self.menu_state {
            MenuState::Main => {
                if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if self.task_showcase.length() == 0 {
                            ui.centered_and_justified(|ui| {
                                ui.label("Release here");
                            });
                        } else {
                            self.task_showcase.poll();
                            self.task_showcase.ui(ui);
                        }
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
                        self.schedule_files(&i.raw.dropped_files);
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
                                let response = ui.add_sized(
                                    ui.available_size() - Vec2::new(60.0, 0.0),
                                    egui::TextEdit::singleline(&mut self.config.password_file_path),
                                );
                                if response.lost_focus() {
                                    self.update_passwords_from_file();
                                }
                                if ui.button("Browse").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                                        self.config.password_file_path = path.display().to_string();
                                    }
                                    self.update_passwords_from_file();
                                }
                            });
                            ui.end_row();
                            ui.label("Path to 7z executable:");
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    ui.available_size() - Vec2::new(60.0, 0.0),
                                    egui::TextEdit::singleline(
                                        &mut self.config.archive_executable_path,
                                    ),
                                );
                                if ui.button("Browse").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                                        self.config.archive_executable_path =
                                            path.display().to_string();
                                    }
                                }
                            });
                            ui.end_row();
                            ui.label("Extraction destination:");
                            ui.vertical(|ui| {
                                ui.radio_value(
                                    &mut self.config.extraction_mode,
                                    ExtractionMode::Local,
                                    "Extract to the same directory",
                                );
                                ui.radio_value(
                                    &mut self.config.extraction_mode,
                                    ExtractionMode::NewDirectory,
                                    "Extract to a new directory",
                                );
                            });
                            ui.end_row();
                            ui.label("Delete archive file:");
                            ui.checkbox(&mut self.config.delete_after_extract, "");
                            ui.end_row();
                            ui.label("Sanitize password file:");
                            ui.checkbox(&mut self.config.sanitize, "");
                            ui.end_row();
                        });
                });
            }
            MenuState::Password => {
                // Check if password is set
                if self.passwords.is_some() {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let response = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(self.passwords.as_mut().unwrap()),
                        );
                        if response.lost_focus() {
                            self.try_sanitize_passwords();
                            self.update_passwords_to_file();
                        }
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
