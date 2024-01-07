use log::{error, info};

use crate::bot::Bot;

#[derive(Debug, Clone)]
pub struct SavedMessage {
    pub author: String,
    pub content: String,
}

impl SavedMessage {
    pub fn get(&self) -> String {
        format!("{}: {}", self.author, self.content)
    }
}

pub type History = Vec<SavedMessage>;

impl Bot {
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

    #[allow(dead_code)]
    pub fn clear_history(&self) {
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

    pub fn get_history_text(&self, n: usize) -> String {
        let mut msg = String::new();

        if let Ok(history) = self.history.lock() {
            for i in 0..(n.min(history.len())) {
                msg.push_str(&history[i].get());
            }
        }

        msg
    }
}
