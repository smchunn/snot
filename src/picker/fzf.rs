use super::{Picker, PickerItem};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;
use anyhow::{Result, Context};

pub struct FzfPicker {
    fzf_options: Vec<String>,
}

impl FzfPicker {
    pub fn new() -> Self {
        Self {
            fzf_options: vec![
                "--preview".to_string(),
                "cat {}".to_string(),
                "--preview-window".to_string(),
                "right:60%:wrap".to_string(),
            ],
        }
    }

    pub fn with_options(mut self, options: Vec<String>) -> Self {
        self.fzf_options = options;
        self
    }
}

impl Default for FzfPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl Picker for FzfPicker {
    fn pick(&self, items: Vec<PickerItem>) -> Result<Option<PathBuf>> {
        if items.is_empty() {
            return Ok(None);
        }

        let mut child = Command::new("fzf")
            .args(&self.fzf_options)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to spawn fzf. Make sure fzf is installed.")?;

        {
            let stdin = child.stdin.as_mut()
                .context("Failed to open fzf stdin")?;

            for item in &items {
                writeln!(stdin, "{}", item.path.display())
                    .context("Failed to write to fzf stdin")?;
            }
        }

        let output = child.wait_with_output()
            .context("Failed to read fzf output")?;

        if !output.status.success() {
            // User cancelled or error
            return Ok(None);
        }

        let selected = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in fzf output")?
            .trim()
            .to_string();

        if selected.is_empty() {
            return Ok(None);
        }

        Ok(Some(PathBuf::from(selected)))
    }
}
