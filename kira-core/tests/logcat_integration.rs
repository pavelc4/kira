use kira_core::device::{parse_logcat_line, LogLevel, LogcatBuffer, LogcatEntry, LogcatFilter};

#[test]
fn test_parse_real_logcat_line_threadtime() {
    let line =
        "01-15 10:30:45.123  1234  5678 I ActivityManager: Starting activity com.example.app";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.pid, 1234);
    assert_eq!(entry.tid, 5678);
    assert_eq!(entry.level, LogLevel::Info);
    assert_eq!(entry.tag, "ActivityManager");
    assert!(entry.message.contains("Starting activity"));
}

#[test]
fn test_parse_real_logcat_line_error() {
    let line = "01-15 10:30:45.456  2345  6789 E System: Error occurred while starting service";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.level, LogLevel::Error);
    assert_eq!(entry.tag, "System");
    assert!(entry.message.contains("Error"));
}

#[test]
fn test_parse_real_logcat_line_warning() {
    let line = "01-15 10:30:45.789  3456  7890 W WindowManager: Low memory warning";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.level, LogLevel::Warning);
    assert_eq!(entry.tag, "WindowManager");
    assert!(entry.message.contains("Low memory"));
}

#[test]
fn test_parse_real_logcat_line_debug() {
    let line = "01-15 10:30:46.012  4567  8901 D MyApp: Debug information here";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.level, LogLevel::Debug);
    assert_eq!(entry.tag, "MyApp");
    assert!(entry.message.contains("Debug"));
}

#[test]
fn test_parse_real_logcat_line_verbose() {
    let line = "01-15 10:30:46.123  5678  9012 V MyApp: Verbose logging with detail=123";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.level, LogLevel::Verbose);
    assert_eq!(entry.tag, "MyApp");
    assert!(entry.message.contains("Verbose"));
}

#[test]
fn test_parse_real_logcat_line_brief_format() {
    let line = "[ActivityManager] I Process started";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.level, LogLevel::Info);
    assert_eq!(entry.tag, "ActivityManager");
    assert!(entry.message.contains("Process started"));
}

#[test]
fn test_parse_real_logcat_line_kotlin_style() {
    let line = "2024-01-15 10:30:47.000  1234  5678 I ActivityTaskManager: START u0 {act=android.intent.action.MAIN cat=[android.intent.category.LAUNCHER] pkg=com.example.app}";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.pid, 1234);
    assert_eq!(entry.tid, 5678);
    assert_eq!(entry.level, LogLevel::Info);
    assert!(entry.tag.contains("ActivityTaskManager") || entry.tag.contains("ActivityManager"));
}

#[test]
fn test_parse_real_logcat_line_with_colon_in_message() {
    let line = "01-15 10:30:48.000  1111  2222 E MyApp: Error: something failed: null pointer";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.level, LogLevel::Error);
    assert!(entry.message.contains("Error:"));
    assert!(entry.message.contains("null pointer"));
}

#[test]
fn test_parse_real_logcat_line_crash() {
    let line = "01-15 10:30:49.000  3333  4444 F libc: Fatal signal 11 (SIGSEGV), code 1";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.level, LogLevel::Fatal);
    assert!(entry.message.contains("Fatal signal") || entry.message.contains("SIGSEGV"));
}

#[test]
fn test_filter_by_specific_tag() {
    let filter = LogcatFilter {
        tag: Some("ActivityManager".to_string()),
        level: None,
        message_contains: None,
    };

    let entry = LogcatEntry {
        timestamp: "01-15 10:30:00".to_string(),
        pid: 1234,
        tid: 5678,
        level: LogLevel::Info,
        tag: "ActivityManager".to_string(),
        message: "Process started".to_string(),
        raw: String::new(),
    };

    assert!(filter.matches(&entry));

    let entry2 = LogcatEntry {
        tag: "WindowManager".to_string(),
        ..entry.clone()
    };

    assert!(!filter.matches(&entry2));
}

#[test]
fn test_filter_by_minimum_level() {
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

    let info_entry = LogcatEntry {
        level: LogLevel::Info,
        ..Default::default()
    };

    let debug_entry = LogcatEntry {
        level: LogLevel::Debug,
        ..Default::default()
    };

    assert!(filter.matches(&warning_entry));
    assert!(filter.matches(&error_entry));
    assert!(!filter.matches(&info_entry));
    assert!(!filter.matches(&debug_entry));
}

#[test]
fn test_filter_by_message_contains() {
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
        message: "Success: operation completed".to_string(),
        ..Default::default()
    };

    assert!(filter.matches(&entry1));
    assert!(!filter.matches(&entry2));
}

#[test]
fn test_combined_filter() {
    let filter = LogcatFilter {
        tag: Some("MyApp".to_string()),
        level: Some(LogLevel::Error),
        message_contains: Some("crash".to_string()),
    };

    let entry1 = LogcatEntry {
        tag: "MyApp".to_string(),
        level: LogLevel::Error,
        message: "App crash detected".to_string(),
        ..Default::default()
    };

    let entry2 = LogcatEntry {
        tag: "MyApp".to_string(),
        level: LogLevel::Error,
        message: "App started successfully".to_string(),
        ..Default::default()
    };

    let entry3 = LogcatEntry {
        tag: "MyApp".to_string(),
        level: LogLevel::Info,
        message: "App crash detected".to_string(),
        ..Default::default()
    };

    let entry4 = LogcatEntry {
        tag: "OtherApp".to_string(),
        level: LogLevel::Error,
        message: "App crash detected".to_string(),
        ..Default::default()
    };

    assert!(filter.matches(&entry1));
    assert!(!filter.matches(&entry2));
    assert!(!filter.matches(&entry3));
    assert!(!filter.matches(&entry4));
}

#[test]
fn test_logcat_buffer_variants() {
    assert_eq!(LogcatBuffer::Main.as_str(), "main");
    assert_eq!(LogcatBuffer::System.as_str(), "system");
    assert_eq!(LogcatBuffer::Radio.as_str(), "radio");
    assert_eq!(LogcatBuffer::Events.as_str(), "events");
    assert_eq!(LogcatBuffer::Crash.as_str(), "crash");
}

#[test]
fn test_log_level_ordering() {
    assert!(LogLevel::Verbose < LogLevel::Debug);
    assert!(LogLevel::Debug < LogLevel::Info);
    assert!(LogLevel::Info < LogLevel::Warning);
    assert!(LogLevel::Warning < LogLevel::Error);
    assert!(LogLevel::Error < LogLevel::Fatal);
    assert!(LogLevel::Fatal < LogLevel::Silent);
}

#[test]
fn test_parse_android_system_logs() {
    let lines = vec![
        "01-15 12:00:00.001  1000  1000 I ActivityManager: START u0 {act=android.intent.action.MAIN cat=[android.intent.category.LAUNCHER] pkg=com.android.launcher}",
        "01-15 12:00:00.050  1000  1000 I ActivityManager: START u0 {act=android.intent.action.MAIN cat=[android.intent.category.LAUNCHER] pkg=com.paget96.batteryguru}",
        "01-15 12:00:01.000  1234  5678 W ActivityManager: Activity pause timeout for ActivityRecord",
        "01-15 12:00:02.000  2345  6789 E ActivityManager: Failure starting process com.paget96.batteryguru",
    ];

    for line in &lines {
        let entry = parse_logcat_line(line);
        assert!(entry.is_some(), "Failed to parse: {}", line);
    }
}

#[test]
fn test_parse_kotlin_null_pointer_exception() {
    let line = "01-15 12:05:30.123  5678  9012 E AndroidRuntime: java.lang.NullPointerException: Attempt to invoke virtual method 'java.lang.String com.example.MyClass.getName()' on a null object reference";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.level, LogLevel::Error);
    assert!(entry.message.contains("NullPointerException"));
}

#[test]
fn test_parse_anr_application_not_responding() {
    let line = "01-15 12:10:00.000  1000  2345 E ActivityManager: ANR in com.paget96.batteryguru (com.paget96.batteryguru/.MainActivity)";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.level, LogLevel::Error);
    assert!(entry.message.contains("ANR"));
    assert!(entry.message.contains("batteryguru"));
}

#[test]
fn test_parse_wifi_connection_event() {
    let line = "01-15 12:15:00.000  1000  1234 I WifiService: WiFi connected to network SSID";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.level, LogLevel::Info);
    assert!(entry.message.contains("WiFi"));
}

#[test]
fn test_parse_battery_service_event() {
    let line = "01-15 12:20:00.000  1000  1000 I BatteryService: level=75, scale=100, status=2, health=2, present=true, technology=Li-ion";

    let entry = parse_logcat_line(line).expect("Should parse successfully");

    assert_eq!(entry.level, LogLevel::Info);
    assert!(entry.message.contains("level="));
    assert!(entry.message.contains("75"));
}
