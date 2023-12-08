use log::{error, info};

use crate::bot::Bot;

#[derive(Debug, Clone)]
pub struct SavedMessage {
    pub author: String,
    pub content: String,
}

pub type History = Vec<SavedMessage>;

impl Bot {
    pub fn _get_history(&self) -> History {
        self.history.lock().unwrap().clone()
    }

    pub fn add_history(&self, author_id: &str, msg: &str) {
        if let Ok(mut history) = self.history.lock() {
            history.push(SavedMessage {
                author: author_id.to_string(),
                content: msg.to_string(),
            });
        } else {
            error!("Failed to acquire lock for history");
        }
    }

    pub fn _clear_history(&self) {
        info!("Clearing history");

        self.history.lock().unwrap().clear();
    }

    pub fn get_last_2_msgs(&self) -> Option<(SavedMessage, SavedMessage)> {
        if let Ok(history) = self.history.lock() {
            if history.len() > 1 {
                return Some((
                    history[history.len() - 2].clone(),
                    history[history.len() - 1].clone(),
                ));
            }
        }

        None
    }
}
