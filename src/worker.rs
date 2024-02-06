use anyhow::{anyhow, Result};
use egui::{scroll_area::ScrollBarVisibility, ScrollArea};
use log::debug;
use std::fmt::Debug;
use std::sync::mpsc;
#[derive(Debug)]
enum TaskState<T: Send + 'static> {
    Running,
    Finished(T),
    Failed(anyhow::Error),
}

#[derive(Debug)]
pub struct Task<T: Send + 'static> {
    state: TaskState<T>,
    description: String,
    task: mpsc::Receiver<Result<T>>,
}

impl<T: Send + 'static> Task<T> {
    pub fn new(description: String, task: impl FnOnce() -> Result<T> + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = task();
            tx.send(result).unwrap();
        });
        Self::new_inner(description, rx)
    }
    fn new_inner(description: String, rx: mpsc::Receiver<Result<T>>) -> Self {
        Self {
            state: TaskState::Running,
            description,
            task: rx,
        }
    }
    fn ui(&self, ui: &mut egui::Ui) {
        ui.label(self.description.clone());
        ui.centered_and_justified(|ui| {
            ui.label(match &self.state {
                TaskState::Running => "...".to_string(),
                TaskState::Finished(_) => "√".to_string(),
                TaskState::Failed(e) => e.to_string(),
            });
        });
    }
}

pub trait TaskDisplayer<T: Send + 'static> {
    fn display(&mut self, task: Task<T>);
}

#[derive(Debug)]
pub struct Showcase<T: Send + 'static> {
    tasks: Vec<Task<T>>,
}

impl<T: Send + Debug + 'static> Showcase<T> {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }
    pub fn poll(&mut self) {
        for task in self.tasks.iter_mut() {
            match task.state {
                TaskState::Running => match task.task.try_recv() {
                    Ok(result) => {
                        debug!(
                            "Task {} finished with result {:?}",
                            task.description, result
                        );
                        match result {
                            Ok(value) => {
                                task.state = TaskState::Finished(value);
                            }
                            Err(error) => {
                                task.state = TaskState::Failed(error);
                            }
                        }
                    }
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
                    .max_col_width(ui.available_width() / 2.0)
                    .striped(true)
                    .show(ui, |ui| {
                        for task in self.tasks.iter() {
                            task.ui(ui);
                            ui.end_row();
                        }
                    });
            });
    }
    pub fn length(&self) -> usize {
        self.tasks.len()
    }
}

impl<T: Send + 'static> TaskDisplayer<T> for Showcase<T> {
    fn display(&mut self, task: Task<T>) {
        self.tasks.push(task);
    }
}

#[cfg(test)]
mod test {
    use crate::worker::TaskDisplayer;

    #[test]
    fn should_schedule_successful_task() {
        let mut showcase = super::Showcase::new();
        let task = super::Task::new(
            "The answer to the ultimate question of life the universe and everything".to_string(),
            || {
                std::thread::sleep(std::time::Duration::from_secs(1));
                Ok("42")
            },
        );
        showcase.display(task);
        assert_eq!(showcase.length(), 1);
        showcase.poll();
        assert!(matches!(showcase.tasks[0].state, super::TaskState::Running));
        std::thread::sleep(std::time::Duration::from_secs(2));
        showcase.poll();
        assert!(matches!(
            showcase.tasks[0].state,
            super::TaskState::Finished("42")
        ));
    }

    #[test]
    fn should_schedule_failed_task() {
        let mut showcase: crate::worker::Showcase<()> = super::Showcase::new();
        let task = super::Task::new(
            "The answer to the ultimate question of life the universe and everything".to_string(),
            || Err(anyhow::anyhow!("Failed")),
        );
        showcase.display(task);
        std::thread::sleep(std::time::Duration::from_secs(1));
        showcase.poll();
        assert!(matches!(
            showcase.tasks[0].state,
            super::TaskState::Failed(_)
        ));
    }
}
