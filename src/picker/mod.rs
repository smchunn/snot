pub mod fzf;

use std::path::PathBuf;
use anyhow::Result;

pub trait Picker {
    fn pick(&self, items: Vec<PickerItem>) -> Result<Option<PathBuf>>;
}

#[derive(Debug, Clone)]
pub struct PickerItem {
    pub title: String,
    pub path: PathBuf,
    pub preview: Option<String>,
}

impl PickerItem {
    pub fn new(title: String, path: PathBuf) -> Self {
        Self {
            title,
            path,
            preview: None,
        }
    }

    pub fn with_preview(mut self, preview: String) -> Self {
        self.preview = Some(preview);
        self
    }
}
