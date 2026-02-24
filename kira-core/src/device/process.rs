use adb_client::server_device::ADBServerDevice;
use adb_client::ADBDeviceExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: String,
    pub free: String,
    pub used: String,
    pub threshold: String,
    pub low_memory: bool,
}

pub fn list_processes(device: &mut ADBServerDevice) -> Result<Vec<ProcessInfo>, ProcessError> {
    let output = run_shell_command(device, "ps")?;

    let processes: Vec<ProcessInfo> = output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let user = parts[0].to_string();
                let pid = parts[1].parse::<u32>().ok()?;
                let name = parts.last()?.to_string();
                Some(ProcessInfo { pid, name, user })
            } else {
                None
            }
        })
        .collect();

    Ok(processes)
}

pub fn kill_process(device: &mut ADBServerDevice, pid: u32) -> Result<(), ProcessError> {
    let output = run_shell_command(device, &format!("kill {}", pid))?;

    if output.contains("Operation not permitted") || output.contains("Permission denied") {
        return Err(ProcessError::PermissionDenied);
    }

    Ok(())
}

pub fn kill_package(device: &mut ADBServerDevice, package_name: &str) -> Result<(), ProcessError> {
    let output = run_shell_command(device, &format!("am force-stop {}", package_name))?;

    if output.contains("Error") || output.contains("failed") {
        return Err(ProcessError::PackageNotFound(package_name.to_string()));
    }

    Ok(())
}

pub fn get_process_memory(
    device: &mut ADBServerDevice,
    pid: u32,
) -> Result<MemoryInfo, ProcessError> {
    let output = run_shell_command(device, &format!("cat /proc/{}/status", pid))?;

    let mut mem_total = String::new();
    let mut mem_free = String::new();
    let mut mem_used = String::new();
    let mut threshold = String::new();
    let mut low_memory = false;

    for line in output.lines() {
        if line.starts_with("VmRSS:") {
            mem_used = line.split_whitespace().nth(1).unwrap_or("0").to_string();
        } else if line.starts_with("VmSize:") {
            mem_total = line.split_whitespace().nth(1).unwrap_or("0").to_string();
        }
    }

    let meminfo = run_shell_command(device, "cat /proc/meminfo")?;
    for line in meminfo.lines() {
        if line.starts_with("MemFree:") {
            mem_free = line.split_whitespace().nth(1).unwrap_or("0").to_string();
        } else if line.starts_with("MemAvailable:") {
            threshold = line.split_whitespace().nth(1).unwrap_or("0").to_string();
        } else if line.starts_with("LowMemory:") {
            low_memory = line.contains("yes") || line.contains("1");
        }
    }

    Ok(MemoryInfo {
        total: mem_total,
        free: mem_free,
        used: mem_used,
        threshold,
        low_memory,
    })
}

pub fn list_running_services(device: &mut ADBServerDevice) -> Result<Vec<String>, ProcessError> {
    let output = run_shell_command(device, "dumpsys activity services")?;

    let services: Vec<String> = output
        .lines()
        .filter_map(|line| {
            if line.contains("Service[") {
                let start = line.find("Service[")? + 8;
                let end = line.find("]")?;
                Some(line[start..end].to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(services)
}

pub fn find_process_by_package(
    device: &mut ADBServerDevice,
    package_name: &str,
) -> Result<Vec<ProcessInfo>, ProcessError> {
    let processes = list_processes(device)?;

    let matching: Vec<ProcessInfo> = processes
        .into_iter()
        .filter(|p| p.name.contains(package_name) || p.name.starts_with(package_name))
        .collect();

    Ok(matching)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessError {
    ProcessNotFound(u32),
    PackageNotFound(String),
    PermissionDenied,
    CommandFailed(String),
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::ProcessNotFound(pid) => write!(f, "Process with PID {} not found", pid),
            ProcessError::PackageNotFound(pkg) => write!(f, "Package {} not found", pkg),
            ProcessError::PermissionDenied => write!(f, "Permission denied to kill process"),
            ProcessError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
        }
    }
}

impl std::error::Error for ProcessError {}

fn run_shell_command(device: &mut ADBServerDevice, command: &str) -> Result<String, ProcessError> {
    let mut output = Vec::new();
    device
        .shell_command(&command, Some(&mut output), None)
        .map_err(|e| ProcessError::CommandFailed(e.to_string()))?;

    String::from_utf8(output)
        .map_err(|e| ProcessError::CommandFailed(e.to_string()))
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_info_creation() {
        let process = ProcessInfo {
            pid: 1234,
            name: "com.example.app".to_string(),
            user: "u0_a123".to_string(),
        };

        assert_eq!(process.pid, 1234);
        assert_eq!(process.name, "com.example.app");
        assert_eq!(process.user, "u0_a123");
    }

    #[test]
    fn test_process_info_clone() {
        let original = ProcessInfo {
            pid: 1234,
            name: "com.example.app".to_string(),
            user: "u0_a123".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_memory_info_creation() {
        let memory = MemoryInfo {
            total: "1024000".to_string(),
            free: "512000".to_string(),
            used: "512000".to_string(),
            threshold: "102400".to_string(),
            low_memory: false,
        };

        assert_eq!(memory.total, "1024000");
        assert!(!memory.low_memory);
    }

    #[test]
    fn test_memory_info_low_memory() {
        let memory = MemoryInfo {
            total: "1024000".to_string(),
            free: "512000".to_string(),
            used: "512000".to_string(),
            threshold: "102400".to_string(),
            low_memory: true,
        };

        assert!(memory.low_memory);
    }

    #[test]
    fn test_process_error_display() {
        let err_not_found = ProcessError::ProcessNotFound(1234);
        assert!(format!("{}", err_not_found).contains("1234"));

        let err_package = ProcessError::PackageNotFound("com.test.app".to_string());
        assert!(format!("{}", err_package).contains("com.test.app"));

        let err_permission = ProcessError::PermissionDenied;
        assert!(format!("{}", err_permission).contains("Permission"));

        let err_command = ProcessError::CommandFailed("test error".to_string());
        assert!(format!("{}", err_command).contains("test error"));
    }

    #[test]
    fn test_process_error_partial_eq() {
        assert_eq!(
            ProcessError::ProcessNotFound(1),
            ProcessError::ProcessNotFound(1)
        );
        assert_ne!(
            ProcessError::ProcessNotFound(1),
            ProcessError::ProcessNotFound(2)
        );

        assert_eq!(
            ProcessError::PackageNotFound("com.app".to_string()),
            ProcessError::PackageNotFound("com.app".to_string())
        );
        assert_ne!(
            ProcessError::PackageNotFound("com.app1".to_string()),
            ProcessError::PackageNotFound("com.app2".to_string())
        );

        assert_eq!(
            ProcessError::PermissionDenied,
            ProcessError::PermissionDenied
        );
    }

    #[test]
    fn test_ps_output_parsing() {
        let ps_output = "USER           PID  PPID     VSZ    RSS WCHAN            ADDR S NAME\n\
                         u0_a123        1234     1   10240   5120 0                   0 S com.example.app\n\
                         u0_a456        5678     1   20480   8192 0                   0 S system_server";

        let processes: Vec<ProcessInfo> = ps_output
            .lines()
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 {
                    let user = parts[0].to_string();
                    let pid = parts[1].parse::<u32>().ok()?;
                    let name = parts.last()?.to_string();
                    Some(ProcessInfo { pid, name, user })
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(processes.len(), 2);
        assert_eq!(processes[0].pid, 1234);
        assert_eq!(processes[0].name, "com.example.app");
        assert_eq!(processes[1].pid, 5678);
        assert_eq!(processes[1].name, "system_server");
    }

    #[test]
    fn test_package_name_filter() {
        let processes = vec![
            ProcessInfo {
                pid: 1,
                name: "com.paget96.batteryguru".to_string(),
                user: "u0_a123".to_string(),
            },
            ProcessInfo {
                pid: 2,
                name: "com.android.phone".to_string(),
                user: "u0_a456".to_string(),
            },
            ProcessInfo {
                pid: 3,
                name: "batteryguru_helper".to_string(),
                user: "u0_a789".to_string(),
            },
        ];

        let matching: Vec<ProcessInfo> = processes
            .into_iter()
            .filter(|p| p.name.contains("batteryguru"))
            .collect();

        assert_eq!(matching.len(), 2);
    }

    #[test]
    fn test_kill_command_format() {
        let pid = 12345;
        let command = format!("kill {}", pid);
        assert_eq!(command, "kill 12345");
    }

    #[test]
    fn test_am_force_stop_command_format() {
        let package = "com.paget96.batteryguru";
        let command = format!("am force-stop {}", package);
        assert_eq!(command, "am force-stop com.paget96.batteryguru");
    }

    #[test]
    fn test_proc_status_parsing() {
        let status_output = "Name:\tcom.example.app\n\
                            State:\tS (sleeping)\n\
                            Pid:\t1234\n\
                            VmRSS:\t512000 kB\n\
                            VmSize:\t1024000 kB";

        let mut mem_used = String::new();

        for line in status_output.lines() {
            if line.starts_with("VmRSS:") {
                mem_used = line.split_whitespace().nth(1).unwrap_or("0").to_string();
            }
        }

        assert_eq!(mem_used, "512000");
    }

    #[test]
    fn test_process_error_debug() {
        let err = ProcessError::ProcessNotFound(999);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ProcessNotFound"));
        assert!(debug_str.contains("999"));
    }
}
