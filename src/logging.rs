//! Chat logging persistence layer
//!
//! Provides file-based logging of IRC messages organized by network and channel.
//! Logs are stored in XDG_DATA_HOME/slirc-client/logs/ with the structure:
//! logs/network/channel/YYYY-MM-DD.log

use chrono::Local;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::thread;
use crossbeam_channel::{unbounded, Receiver, Sender};

/// A log entry to be written to disk
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub network: String,
    pub channel: String,
    pub timestamp: String,
    pub nick: String,
    pub message: String,
}

/// Logger manages file-based chat logging without blocking the UI thread
pub struct Logger {
    /// Channel to send log entries to the background thread
    tx: Sender<LogEntry>,
}

impl Logger {
    /// Create a new logger and spawn background thread for async I/O
    pub fn new() -> Result<Self, String> {
        let log_dir = get_log_directory()?;

        // Create log directory if it doesn't exist
        fs::create_dir_all(&log_dir)
            .map_err(|e| format!("Failed to create log directory: {}", e))?;

        let (tx, rx) = unbounded::<LogEntry>();

        // Spawn background thread for non-blocking I/O
        let log_dir_clone = log_dir.clone();
        thread::spawn(move || {
            run_logger_thread(rx, log_dir_clone);
        });

        Ok(Self { tx })
    }

    /// Log a message (non-blocking, queued for background writing)
    pub fn log(&self, entry: LogEntry) {
        // If send fails, the logger thread has stopped - silently ignore
        let _ = self.tx.send(entry);
    }
}

/// Background thread that handles all file I/O
fn run_logger_thread(rx: Receiver<LogEntry>, log_dir: PathBuf) {
    // Cache of open file handles to avoid reopening files constantly
    let mut file_cache: HashMap<String, BufWriter<File>> = HashMap::new();

    // Process log entries as they arrive
    while let Ok(entry) = rx.recv() {
        if let Err(e) = write_log_entry(&mut file_cache, &log_dir, &entry) {
            eprintln!("Logger error: {}", e);
        }
    }

    // Flush all cached files on shutdown
    for (_, mut writer) in file_cache.drain() {
        let _ = writer.flush();
    }
}

/// Write a single log entry to the appropriate file
fn write_log_entry(
    file_cache: &mut HashMap<String, BufWriter<File>>,
    log_dir: &std::path::Path,
    entry: &LogEntry,
) -> Result<(), String> {
    // Build path: logs/network/channel/YYYY-MM-DD.log
    let date = Local::now().format("%Y-%m-%d").to_string();
    let sanitized_network = sanitize_filename(&entry.network);
    let sanitized_channel = sanitize_filename(&entry.channel);

    let channel_dir = log_dir.join(&sanitized_network).join(&sanitized_channel);
    fs::create_dir_all(&channel_dir)
        .map_err(|e| format!("Failed to create channel directory: {}", e))?;

    let log_file_path = channel_dir.join(format!("{}.log", date));
    let cache_key = format!("{}/{}/{}", sanitized_network, sanitized_channel, date);

    // Get or create buffered writer for this file
    let writer = if let Some(w) = file_cache.get_mut(&cache_key) {
        w
    } else {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        file_cache.insert(cache_key.clone(), BufWriter::new(file));
        file_cache.get_mut(&cache_key).unwrap()
    };

    // Format: [HH:MM:SS] <Nick> Message
    writeln!(writer, "[{}] <{}> {}", entry.timestamp, entry.nick, entry.message)
        .map_err(|e| format!("Failed to write log entry: {}", e))?;

    // Flush periodically to ensure logs are written
    writer.flush()
        .map_err(|e| format!("Failed to flush log: {}", e))?;

    Ok(())
}

/// Get the platform-specific log directory using XDG conventions
fn get_log_directory() -> Result<PathBuf, String> {
    let base = directories::BaseDirs::new()
        .ok_or("Failed to determine home directory")?;

    // Use XDG_DATA_HOME on Linux, equivalent on other platforms
    let data_dir = base.data_dir();
    Ok(data_dir.join("slirc-client").join("logs"))
}

/// Sanitize a filename to be filesystem-safe
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            '#' => '_', // Remove # prefix from channels for cleaner paths
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("#rust"), "_rust");
        assert_eq!(sanitize_filename("irc.libera.chat"), "irc.libera.chat");
        assert_eq!(sanitize_filename("test/path"), "test_path");
    }

    #[test]
    fn test_log_directory_exists() {
        let result = get_log_directory();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("slirc-client"));
    }
}
