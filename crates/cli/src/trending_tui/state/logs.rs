//! Logs state

/// Logs state
#[derive(Debug)]
pub struct LogsState {
    pub messages: Vec<String>,
    pub scroll: usize,
}

impl LogsState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll: 0,
        }
    }

    /// Save all logs to a file. Returns the path to the saved file.
    pub fn save_to_file(&self) -> std::io::Result<String> {
        use std::io::Write;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("polymarket_logs_{}.txt", timestamp);

        let mut file = std::fs::File::create(&filename)?;

        writeln!(file, "Polymarket TUI Logs")?;
        writeln!(
            file,
            "Saved at: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        )?;
        writeln!(file, "Total entries: {}", self.messages.len())?;
        writeln!(file, "{}", "=".repeat(80))?;
        writeln!(file)?;

        for (i, msg) in self.messages.iter().enumerate() {
            writeln!(file, "[{}] {}", i + 1, msg)?;
        }

        Ok(filename)
    }
}
