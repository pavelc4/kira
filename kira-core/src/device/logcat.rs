use adb_client::server_device::ADBServerDevice;
use adb_client::ADBDeviceExt;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogcatEntry {
    pub timestamp: String,
    pub pid: u32,
    pub tid: u32,
    pub level: LogLevel,
    pub tag: String,
    pub message: String,
    pub raw: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Verbose,
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
    Silent,
}

impl From<char> for LogLevel {
    fn from(c: char) -> Self {
        match c {
            'V' => LogLevel::Verbose,
            'D' => LogLevel::Debug,
            'I' => LogLevel::Info,
            'W' => LogLevel::Warning,
            'E' => LogLevel::Error,
            'F' => LogLevel::Fatal,
            'S' => LogLevel::Silent,
            _ => LogLevel::Debug,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::Verbose => "V",
            LogLevel::Debug => "D",
            LogLevel::Info => "I",
            LogLevel::Warning => "W",
            LogLevel::Error => "E",
            LogLevel::Fatal => "F",
            LogLevel::Silent => "S",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogcatBuffer {
    Main,
    System,
    Radio,
    Events,
    Crash,
    Default,
}

impl LogcatBuffer {
    pub fn as_str(&self) -> &str {
        match self {
            LogcatBuffer::Main => "main",
            LogcatBuffer::System => "system",
            LogcatBuffer::Radio => "radio",
            LogcatBuffer::Events => "events",
            LogcatBuffer::Crash => "crash",
            LogcatBuffer::Default => "main",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogcatFilter {
    pub tag: Option<String>,
    pub level: Option<LogLevel>,
    pub message_contains: Option<String>,
}

impl LogcatFilter {
    pub fn matches(&self, entry: &LogcatEntry) -> bool {
        if let Some(ref tag) = self.tag {
            if !entry.tag.contains(tag) {
                return false;
            }
        }
        if let Some(level) = self.level {
            if entry.level < level {
                return false;
            }
        }
        if let Some(ref msg) = self.message_contains {
            if !entry.message.contains(msg) {
                return false;
            }
        }
        true
    }
}

impl Default for LogcatFilter {
    fn default() -> Self {
        Self {
            tag: None,
            level: Some(LogLevel::Info),
            message_contains: None,
        }
    }
}

pub fn parse_logcat_line(line: &str) -> Option<LogcatEntry> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    if let Some(entry) = parse_threadtime_format(line) {
        return Some(entry);
    }

    if let Some(entry) = parse_brief_format(line) {
        return Some(entry);
    }

    Some(LogcatEntry {
        timestamp: String::new(),
        pid: 0,
        tid: 0,
        level: LogLevel::Debug,
        tag: String::new(),
        message: line.to_string(),
        raw: line.to_string(),
    })
}

fn parse_threadtime_format(line: &str) -> Option<LogcatEntry> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 7 {
        let timestamp = parts.get(0).map(|s| s.to_string())?;
        let pid = parts.get(2)?.parse::<u32>().ok()?;
        let tid = parts.get(3)?.parse::<u32>().ok()?;
        let level_char = parts.get(4)?.chars().next()?;
        let level = LogLevel::from(level_char);
        let tag = parts.get(5)?.trim_matches(':').to_string();
        let message = parts[6..].join(" ");

        return Some(LogcatEntry {
            timestamp,
            pid,
            tid,
            level,
            tag,
            message,
            raw: line.to_string(),
        });
    }
    None
}

fn parse_brief_format(line: &str) -> Option<LogcatEntry> {
    if let Some(bracket_start) = line.find('[') {
        if let Some(bracket_end) = line.find(']') {
            let tag = line[bracket_start + 1..bracket_end].to_string();
            let rest = line[bracket_end + 1..].trim();
            let level_char = rest.chars().next().unwrap_or('I');
            let level = LogLevel::from(level_char);
            let message = rest[2..].trim().to_string();

            return Some(LogcatEntry {
                timestamp: String::new(),
                pid: 0,
                tid: 0,
                level,
                tag,
                message,
                raw: line.to_string(),
            });
        }
    }
    None
}

pub fn read_logcat(
    device: &mut ADBServerDevice,
    buffer: LogcatBuffer,
    lines: usize,
) -> Result<Vec<LogcatEntry>, LogcatError> {
    let command = format!("logcat -d -b {} -t {}", buffer.as_str(), lines);
    let output = run_shell_command(device, &command)?;

    let entries: Vec<LogcatEntry> = output.lines().filter_map(parse_logcat_line).collect();

    Ok(entries)
}

pub fn clear_logcat(device: &mut ADBServerDevice, buffer: LogcatBuffer) -> Result<(), LogcatError> {
    let command = format!("logcat -c -b {}", buffer.as_str());
    run_shell_command(device, &command)?;
    Ok(())
}

pub fn stream_logcat(
    device: &mut ADBServerDevice,
    buffer: LogcatBuffer,
    filter: LogcatFilter,
) -> Result<mpsc::Receiver<LogcatEntry>, LogcatError> {
    let (tx, rx) = mpsc::channel();
    let command = format!("logcat -v threadtime -b {}", buffer.as_str());

    let serial = device
        .identifier
        .as_ref()
        .ok_or_else(|| LogcatError::DeviceNotFound)?;

    let mut child = std::process::Command::new("adb")
        .args(["-s", serial, "shell", &command])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| LogcatError::IOError(e.to_string()))?;

    let stdout = child
        .stdout
        .take()
        .ok_or(LogcatError::IOError("Failed to capture stdout".to_string()))?;
    let reader = BufReader::new(stdout);

    thread::spawn(move || {
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if let Some(entry) = parse_logcat_line(&line) {
                        if filter.matches(&entry) {
                            if tx.send(entry).is_err() {
                                break;
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
        let _ = child.kill();
    });

    Ok(rx)
}

pub fn get_logcat_buffers(device: &mut ADBServerDevice) -> Result<Vec<String>, LogcatError> {
    let output = run_shell_command(device, "logcat -g")?;

    let buffers: Vec<String> = output
        .lines()
        .filter_map(|line| {
            if line.contains("ring buffer") {
                let parts: Vec<&str> = line.split(':').collect();
                parts
                    .first()
                    .map(|s| s.split_whitespace().last().unwrap_or("").to_string())
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    Ok(buffers)
}

pub fn filter_entries(entries: Vec<LogcatEntry>, filter: LogcatFilter) -> Vec<LogcatEntry> {
    entries.into_iter().filter(|e| filter.matches(e)).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogcatError {
    DeviceNotFound,
    IOError(String),
    ParseError(String),
    StreamClosed,
}

impl std::fmt::Display for LogcatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogcatError::DeviceNotFound => write!(f, "Device not found"),
            LogcatError::IOError(msg) => write!(f, "IO Error: {}", msg),
            LogcatError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            LogcatError::StreamClosed => write!(f, "Logcat stream closed"),
        }
    }
}

impl std::error::Error for LogcatError {}

fn run_shell_command(device: &mut ADBServerDevice, command: &str) -> Result<String, LogcatError> {
    let mut output = Vec::new();
    device
        .shell_command(&command, Some(&mut output), None)
        .map_err(|e| LogcatError::IOError(e.to_string()))?;

    String::from_utf8(output)
        .map_err(|e| LogcatError::ParseError(e.to_string()))
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_char() {
        assert_eq!(LogLevel::from('V'), LogLevel::Verbose);
        assert_eq!(LogLevel::from('D'), LogLevel::Debug);
        assert_eq!(LogLevel::from('I'), LogLevel::Info);
        assert_eq!(LogLevel::from('W'), LogLevel::Warning);
        assert_eq!(LogLevel::from('E'), LogLevel::Error);
        assert_eq!(LogLevel::from('F'), LogLevel::Fatal);
        assert_eq!(LogLevel::from('S'), LogLevel::Silent);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::Verbose), "V");
        assert_eq!(format!("{}", LogLevel::Info), "I");
        assert_eq!(format!("{}", LogLevel::Error), "E");
    }

    #[test]
    fn test_logcat_buffer_as_str() {
        assert_eq!(LogcatBuffer::Main.as_str(), "main");
        assert_eq!(LogcatBuffer::System.as_str(), "system");
        assert_eq!(LogcatBuffer::Radio.as_str(), "radio");
    }

    #[test]
    fn test_logcat_filter_default() {
        let filter = LogcatFilter::default();
        assert!(filter.level.is_some());
    }

    #[test]
    fn test_logcat_filter_by_tag() {
        let filter = LogcatFilter {
            tag: Some("ActivityManager".to_string()),
            level: None,
            message_contains: None,
        };

        let entry = LogcatEntry {
            timestamp: "01-15 12:00:00.000".to_string(),
            pid: 1234,
            tid: 1234,
            level: LogLevel::Info,
            tag: "ActivityManager".to_string(),
            message: "Process started".to_string(),
            raw: String::new(),
        };

        assert!(filter.matches(&entry));

        let entry2 = LogcatEntry {
            tag: "OtherTag".to_string(),
            ..entry.clone()
        };

        assert!(!filter.matches(&entry2));
    }

    #[test]
    fn test_logcat_filter_by_level() {
        let filter = LogcatFilter {
            tag: None,
            level: Some(LogLevel::Warning),
            message_contains: None,
        };

        let warning_entry = LogcatEntry {
            level: LogLevel::Warning,
            ..Default::default()
        };

        let error_entry = LogcatEntry {
            level: LogLevel::Error,
            ..Default::default()
        };

        let debug_entry = LogcatEntry {
            level: LogLevel::Debug,
            ..Default::default()
        };

        assert!(filter.matches(&warning_entry));
        assert!(filter.matches(&error_entry));
        assert!(!filter.matches(&debug_entry));
    }

    #[test]
    fn test_logcat_filter_by_message() {
        let filter = LogcatFilter {
            tag: None,
            level: None,
            message_contains: Some("error".to_string()),
        };

        let entry1 = LogcatEntry {
            message: "An error occurred".to_string(),
            ..Default::default()
        };

        let entry2 = LogcatEntry {
            message: "Everything is fine".to_string(),
            ..Default::default()
        };

        assert!(filter.matches(&entry1));
        assert!(!filter.matches(&entry2));
    }

    #[test]
    fn test_parse_logcat_threadtime_format() {
        let line = "01-15 12:00:00.123  1234  5678 I ActivityManager: Starting activity";

        let entry = parse_logcat_line(line).unwrap();

        assert_eq!(entry.pid, 1234);
        assert_eq!(entry.tid, 5678);
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.tag, "ActivityManager");
        assert!(entry.message.contains("Starting activity"));
    }

    #[test]
    fn test_parse_logcat_brief_format() {
        let line = "[ActivityManager] I Starting activity";

        let entry = parse_logcat_line(line).unwrap();

        assert_eq!(entry.tag, "ActivityManager");
        assert_eq!(entry.level, LogLevel::Info);
        assert!(entry.message.contains("Starting activity"));
    }

    #[test]
    fn test_parse_logcat_empty_line() {
        let entry = parse_logcat_line("");
        assert!(entry.is_none());

        let entry2 = parse_logcat_line("   ");
        assert!(entry2.is_none());
    }

    #[test]
    fn test_parse_logcat_error_level() {
        let line = "01-15 12:00:00.123  1234  5678 E System: Error occurred";

        let entry = parse_logcat_line(line).unwrap();

        assert_eq!(entry.level, LogLevel::Error);
    }

    #[test]
    fn test_parse_logcat_debug_level() {
        let line = "01-15 12:00:00.123  1234  5678 D MyApp: Debug message";

        let entry = parse_logcat_line(line).unwrap();

        assert_eq!(entry.level, LogLevel::Debug);
    }

    #[test]
    fn test_filter_entries_function() {
        let entries = vec![
            LogcatEntry {
                tag: "ActivityManager".to_string(),
                level: LogLevel::Info,
                message: "Starting".to_string(),
                ..Default::default()
            },
            LogcatEntry {
                tag: "MyApp".to_string(),
                level: LogLevel::Debug,
                message: "Debug info".to_string(),
                ..Default::default()
            },
        ];

        let filter = LogcatFilter {
            tag: Some("ActivityManager".to_string()),
            level: None,
            message_contains: None,
        };

        let filtered = filter_entries(entries, filter);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].tag, "ActivityManager");
    }

    #[test]
    fn test_logcat_entry_default() {
        let entry = LogcatEntry::default();

        assert_eq!(entry.pid, 0);
        assert_eq!(entry.level, LogLevel::Debug);
        assert!(entry.raw.is_empty());
    }

    #[test]
    fn test_logcat_error_display() {
        let err = LogcatError::DeviceNotFound;
        assert!(format!("{}", err).contains("Device"));

        let err2 = LogcatError::IOError("test".to_string());
        assert!(format!("{}", err2).contains("IO Error"));

        let err3 = LogcatError::ParseError("parse failed".to_string());
        assert!(format!("{}", err3).contains("Parse Error"));

        let err4 = LogcatError::StreamClosed;
        assert!(format!("{}", err4).contains("closed"));
    }

    #[test]
    fn test_multiple_tags_filter() {
        let filter = LogcatFilter {
            tag: Some("Activity".to_string()),
            level: None,
            message_contains: None,
        };

        let entry1 = LogcatEntry {
            tag: "ActivityManager".to_string(),
            level: LogLevel::Info,
            ..Default::default()
        };

        let entry2 = LogcatEntry {
            tag: "ActivityTaskManager".to_string(),
            level: LogLevel::Info,
            ..Default::default()
        };

        let entry3 = LogcatEntry {
            tag: "WindowManager".to_string(),
            level: LogLevel::Info,
            ..Default::default()
        };

        assert!(filter.matches(&entry1));
        assert!(filter.matches(&entry2));
        assert!(!filter.matches(&entry3));
    }

    #[test]
    fn test_case_sensitive_tag_filter() {
        let filter = LogcatFilter {
            tag: Some("activity".to_string()),
            level: None,
            message_contains: None,
        };

        let entry = LogcatEntry {
            tag: "ActivityManager".to_string(),
            ..Default::default()
        };

        assert!(!filter.matches(&entry));
    }

    #[test]
    fn test_logcat_entry_clone() {
        let entry = LogcatEntry {
            timestamp: "01-15 12:00:00".to_string(),
            pid: 1234,
            tid: 5678,
            level: LogLevel::Error,
            tag: "MyApp".to_string(),
            message: "Error message".to_string(),
            raw: "raw line".to_string(),
        };

        let cloned = entry.clone();

        assert_eq!(entry.timestamp, cloned.timestamp);
        assert_eq!(entry.pid, cloned.pid);
        assert_eq!(entry.tag, cloned.tag);
    }
}

impl Default for LogcatEntry {
    fn default() -> Self {
        Self {
            timestamp: String::new(),
            pid: 0,
            tid: 0,
            level: LogLevel::Debug,
            tag: String::new(),
            message: String::new(),
            raw: String::new(),
        }
    }
}
