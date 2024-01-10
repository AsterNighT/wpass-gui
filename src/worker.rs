use anyhow::{anyhow, Result};
use egui::{scroll_area::ScrollBarVisibility, Grid, ScrollArea, Widget};
use std::{error::Error, future::Future, process::Output, sync::mpsc};

#[derive(Debug)]
enum TaskState<T: Send + 'static> {
    NotStarted,
    Running,
    Finished(T),
    Failed(anyhow::Error),
}

pub struct Task<T: Send + 'static> {
    state: TaskState<T>,
    description: String,
    task: mpsc::Receiver<Result<T>>,
}

impl<T: Send + 'static> Task<T> {
    pub fn new(description: String, task: impl FnOnce() -> Result<T> + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel();
        tokio::spawn(async move {
            let result = task();
            tx.send(result).unwrap();
        });
        Self::new_inner(description, rx)
    }
    fn new_inner(description: String, rx: mpsc::Receiver<Result<T>>) -> Self {
        Self {
            state: TaskState::NotStarted,
            description,
            task: rx,
        }
    }
    fn ui(&self, ui: &mut egui::Ui) {
        ui.label(self.description.clone());
        ui.label(match &self.state {
            TaskState::NotStarted => "_".to_string(),
            TaskState::Running => "...".to_string(),
            TaskState::Finished(_) => "√".to_string(),
            TaskState::Failed(e) => e.to_string(),
        });
    }
}

trait TaskDisplayer<T: Send + 'static> {
    fn display_task(&mut self, task: Task<T>);
}

pub struct Showcase<T: Send + 'static> {
    tasks: Vec<Task<T>>,
}

impl<T: Send + 'static> Showcase<T> {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }
    fn poll(&mut self) {
        for task in self.tasks.iter_mut() {
            match task.state {
                TaskState::NotStarted => {
                    task.state = TaskState::Running;
                }
                TaskState::Running => match task.task.try_recv() {
                    Ok(result) => match result {
                        Ok(value) => {
                            task.state = TaskState::Finished(value);
                        }
                        Err(error) => {
                            task.state = TaskState::Failed(error);
                        }
                    },
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => {
                        task.state = TaskState::Failed(anyhow!("Task channel disconnected"));
                    }
                },
                TaskState::Finished(_) => {}
                TaskState::Failed(_) => {}
            }
        }
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical()
            .auto_shrink(false)
            .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
            .show(ui, |ui| {
                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([4.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for task in self.tasks.iter() {
                            task.ui(ui);
                            ui.end_row();
                        }
                    });
            });
    }
}

impl<T: Send + 'static> TaskDisplayer<T> for Showcase<T> {
    fn display_task(&mut self, task: Task<T>) {
        self.tasks.push(task);
    }
}
