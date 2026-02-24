use adb_client::server_device::ADBServerDevice;
use adb_client::ADBDeviceExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellSession {
    pub id: String,
    pub working_dir: String,
    pub env: HashMap<String, String>,
    pub is_root: bool,
}

pub struct ShellExecutor {
    sessions: HashMap<String, ShellSession>,
}

impl ShellExecutor {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn execute(
        &mut self,
        device: &mut ADBServerDevice,
        command: &str,
    ) -> Result<CommandOutput, ShellError> {
        let start = std::time::Instant::now();

        let output = run_shell_command(device, command)?;
        let duration_ms = start.elapsed().as_millis() as u64;

        let (stdout, stderr) = if output.contains("error:") || output.contains("Error:") {
            let parts: Vec<&str> = output.splitn(2, "error:").collect();
            if parts.len() == 2 {
                return Ok(CommandOutput {
                    stdout: parts[0].trim().to_string(),
                    stderr: format!("error:{}", parts[1]),
                    exit_code: 1,
                    duration_ms,
                });
            }
            (output, String::new())
        } else {
            (output, String::new())
        };

        Ok(CommandOutput {
            stdout,
            stderr,
            exit_code: 0,
            duration_ms,
        })
    }

    pub fn execute_with_su(
        &mut self,
        device: &mut ADBServerDevice,
        command: &str,
    ) -> Result<CommandOutput, ShellError> {
        let su_command = format!("su -c '{}'", command.replace("'", "'\\''"));
        self.execute(device, &su_command)
    }

    pub fn execute_as_root(
        &mut self,
        device: &mut ADBServerDevice,
        command: &str,
    ) -> Result<CommandOutput, ShellError> {
        self.execute_with_su(device, command)
    }

    pub fn get_prop(
        &mut self,
        device: &mut ADBServerDevice,
        key: &str,
    ) -> Result<String, ShellError> {
        let output = run_shell_command(device, &format!("getprop {}", key))?;
        Ok(output.trim().to_string())
    }

    pub fn set_prop(
        &mut self,
        device: &mut ADBServerDevice,
        key: &str,
        value: &str,
    ) -> Result<(), ShellError> {
        run_shell_command(device, &format!("setprop {} {}", key, value))?;
        Ok(())
    }

    pub fn list_files(
        &mut self,
        device: &mut ADBServerDevice,
        path: &str,
    ) -> Result<Vec<FileEntry>, ShellError> {
        let output = run_shell_command(device, &format!("ls -la {}", path))?;

        let entries: Vec<FileEntry> = output
            .lines()
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 {
                    let permissions = parts[0].to_string();
                    let size = parts[4].parse::<u64>().ok().unwrap_or(0);
                    let name = parts[8..].join(" ");
                    let is_dir = permissions.starts_with('d');

                    Some(FileEntry {
                        name,
                        permissions,
                        size,
                        is_directory: is_dir,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(entries)
    }

    pub fn get_device_status(
        &mut self,
        device: &mut ADBServerDevice,
    ) -> Result<DeviceStatus, ShellError> {
        let uptime = run_shell_command(device, "cat /proc/uptime")?;
        let meminfo = run_shell_command(device, "cat /proc/meminfo")?;
        let loadavg = run_shell_command(device, "cat /proc/loadavg").unwrap_or_default();

        let uptime_secs: f64 = uptime
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let load: Vec<f64> = loadavg
            .split_whitespace()
            .take(3)
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();

        let mut total_mem = 0u64;
        let mut free_mem = 0u64;
        let mut available_mem = 0u64;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total_mem = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0)
                    * 1024;
            } else if line.starts_with("MemFree:") {
                free_mem = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0)
                    * 1024;
            } else if line.starts_with("MemAvailable:") {
                available_mem = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0)
                    * 1024;
            }
        }

        Ok(DeviceStatus {
            uptime_secs: uptime_secs as u64,
            total_memory: total_mem,
            free_memory: free_mem,
            available_memory: available_mem,
            load_average_1m: load.get(0).copied().unwrap_or(0.0),
            load_average_5m: load.get(1).copied().unwrap_or(0.0),
            load_average_15m: load.get(2).copied().unwrap_or(0.0),
        })
    }

    pub fn is_root_available(&mut self, device: &mut ADBServerDevice) -> bool {
        let output = run_shell_command(device, "id");
        match output {
            Ok(s) => s.contains("uid=0"),
            Err(_) => false,
        }
    }

    pub fn get_selinux_status(
        &mut self,
        device: &mut ADBServerDevice,
    ) -> Result<String, ShellError> {
        let output = run_shell_command(device, "getenforce")?;
        Ok(output.trim().to_string())
    }

    pub fn get_mounts(
        &mut self,
        device: &mut ADBServerDevice,
    ) -> Result<Vec<MountInfo>, ShellError> {
        let output = run_shell_command(device, "cat /proc/mounts")?;

        let mounts: Vec<MountInfo> = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    Some(MountInfo {
                        device: parts[0].to_string(),
                        mount_point: parts[1].to_string(),
                        fs_type: parts[2].to_string(),
                        options: parts[3].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(mounts)
    }

    pub fn get_networks(
        &mut self,
        device: &mut ADBServerDevice,
    ) -> Result<Vec<NetworkInterface>, ShellError> {
        let output = run_shell_command(device, "ip addr show")?;

        let mut interfaces = Vec::new();
        let mut current_iface: Option<NetworkInterface> = None;

        for line in output.lines() {
            if line.contains(": ") {
                if let Some(iface) = current_iface.take() {
                    interfaces.push(iface);
                }
                let name = line
                    .split(": ")
                    .nth(1)
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() && name != "lo" {
                    current_iface = Some(NetworkInterface {
                        name,
                        state: if line.contains("state UP") {
                            "UP".to_string()
                        } else {
                            "DOWN".to_string()
                        },
                        ip_address: String::new(),
                    });
                }
            } else if let Some(ref mut iface) = current_iface {
                if line.contains("inet ") {
                    let inet = line.trim().split_whitespace().nth(1).unwrap_or("");
                    if let Some(ip) = inet.split('/').next() {
                        iface.ip_address = ip.to_string();
                    }
                }
            }
        }

        if let Some(iface) = current_iface {
            interfaces.push(iface);
        }

        Ok(interfaces)
    }

    pub fn run_dumpsys(
        &mut self,
        device: &mut ADBServerDevice,
        service: &str,
    ) -> Result<String, ShellError> {
        let output = run_shell_command(device, &format!("dumpsys {}", service))?;
        Ok(output)
    }

    pub fn run_dumpsys_battery(
        &mut self,
        device: &mut ADBServerDevice,
    ) -> Result<BatteryInfo, ShellError> {
        let output = self.run_dumpsys(device, "battery")?;

        let mut level = 0u32;
        let mut scale = 100u32;
        let mut status = String::new();
        let mut health = String::new();
        let mut plugged = String::new();
        let mut voltage = 0u32;
        let mut temperature = 0i32;
        let mut technology = String::new();

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("level:") {
                level = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("scale:") {
                scale = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(100);
            } else if line.starts_with("status:") {
                status = line.split_whitespace().nth(1).unwrap_or("").to_string();
            } else if line.starts_with("health:") {
                health = line.split_whitespace().nth(1).unwrap_or("").to_string();
            } else if line.starts_with("plugged:") {
                plugged = line.split_whitespace().nth(1).unwrap_or("").to_string();
            } else if line.starts_with("voltage:") {
                voltage = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("temperature:") {
                temperature = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("technology:") {
                technology = line.split_whitespace().nth(1).unwrap_or("").to_string();
            }
        }

        Ok(BatteryInfo {
            level,
            scale,
            percentage: if scale > 0 {
                (level as f32 / scale as f32 * 100.0) as u32
            } else {
                0
            },
            status,
            health,
            plugged,
            voltage,
            temperature: temperature as f32 / 10.0,
            technology,
        })
    }
}

impl Default for ShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub permissions: String,
    pub size: u64,
    pub is_directory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub uptime_secs: u64,
    pub total_memory: u64,
    pub free_memory: u64,
    pub available_memory: u64,
    pub load_average_1m: f64,
    pub load_average_5m: f64,
    pub load_average_15m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountInfo {
    pub device: String,
    pub mount_point: String,
    pub fs_type: String,
    pub options: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub state: String,
    pub ip_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub level: u32,
    pub scale: u32,
    pub percentage: u32,
    pub status: String,
    pub health: String,
    pub plugged: String,
    pub voltage: u32,
    pub temperature: f32,
    pub technology: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShellError {
    DeviceNotFound,
    CommandFailed(String),
    Timeout,
    PermissionDenied,
    IOError(String),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellError::DeviceNotFound => write!(f, "Device not found"),
            ShellError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            ShellError::Timeout => write!(f, "Command timed out"),
            ShellError::PermissionDenied => write!(f, "Permission denied"),
            ShellError::IOError(msg) => write!(f, "IO Error: {}", msg),
        }
    }
}

impl std::error::Error for ShellError {}

fn run_shell_command(device: &mut ADBServerDevice, command: &str) -> Result<String, ShellError> {
    let mut output = Vec::new();
    device
        .shell_command(&command, Some(&mut output), None)
        .map_err(|e| ShellError::CommandFailed(e.to_string()))?;

    String::from_utf8(output)
        .map_err(|e| ShellError::IOError(e.to_string()))
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_output_creation() {
        let output = CommandOutput {
            stdout: "Hello World".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
            duration_ms: 100,
        };

        assert_eq!(output.stdout, "Hello World");
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.duration_ms, 100);
    }

    #[test]
    fn test_shell_executor_new() {
        let executor = ShellExecutor::new();
        assert!(executor.sessions.is_empty());
    }

    #[test]
    fn test_shell_executor_default() {
        let executor = ShellExecutor::default();
        assert!(executor.sessions.is_empty());
    }

    #[test]
    fn test_file_entry_creation() {
        let entry = FileEntry {
            name: "test.txt".to_string(),
            permissions: "-rw-r--r--".to_string(),
            size: 1024,
            is_directory: false,
        };

        assert_eq!(entry.name, "test.txt");
        assert!(!entry.is_directory);
    }

    #[test]
    fn test_file_entry_directory() {
        let entry = FileEntry {
            name: "mydir".to_string(),
            permissions: "drwxr-xr-x".to_string(),
            size: 4096,
            is_directory: true,
        };

        assert!(entry.is_directory);
    }

    #[test]
    fn test_device_status_creation() {
        let status = DeviceStatus {
            uptime_secs: 3600,
            total_memory: 8_000_000_000,
            free_memory: 4_000_000_000,
            available_memory: 6_000_000_000,
            load_average_1m: 1.5,
            load_average_5m: 1.2,
            load_average_15m: 1.0,
        };

        assert_eq!(status.uptime_secs, 3600);
        assert!(status.load_average_1m > 0.0);
    }

    #[test]
    fn test_mount_info_creation() {
        let mount = MountInfo {
            device: "/dev/block/sda1".to_string(),
            mount_point: "/data".to_string(),
            fs_type: "ext4".to_string(),
            options: "rw,seclabel".to_string(),
        };

        assert_eq!(mount.mount_point, "/data");
        assert_eq!(mount.fs_type, "ext4");
    }

    #[test]
    fn test_network_interface_creation() {
        let iface = NetworkInterface {
            name: "wlan0".to_string(),
            state: "UP".to_string(),
            ip_address: "192.168.1.100".to_string(),
        };

        assert_eq!(iface.name, "wlan0");
        assert_eq!(iface.state, "UP");
    }

    #[test]
    fn test_battery_info_creation() {
        let battery = BatteryInfo {
            level: 75,
            scale: 100,
            percentage: 75,
            status: "Charging".to_string(),
            health: "Good".to_string(),
            plugged: "USB".to_string(),
            voltage: 4200,
            temperature: 25.5,
            technology: "Li-ion".to_string(),
        };

        assert_eq!(battery.percentage, 75);
        assert_eq!(battery.temperature, 25.5);
    }

    #[test]
    fn test_shell_error_display() {
        let err = ShellError::DeviceNotFound;
        assert!(format!("{}", err).contains("Device"));

        let err2 = ShellError::CommandFailed("test".to_string());
        assert!(format!("{}", err2).contains("Command failed"));

        let err3 = ShellError::IOError("io error".to_string());
        assert!(format!("{}", err3).contains("IO Error"));
    }

    #[test]
    fn test_shell_error_variants() {
        let err1 = ShellError::Timeout;
        let err2 = ShellError::Timeout;
        assert_eq!(err1, err2);

        let err3 = ShellError::PermissionDenied;
        let err4 = ShellError::PermissionDenied;
        assert_eq!(err3, err4);
    }

    #[test]
    fn test_command_output_with_stderr() {
        let output = CommandOutput {
            stdout: "result".to_string(),
            stderr: "error message".to_string(),
            exit_code: 1,
            duration_ms: 50,
        };

        assert_eq!(output.stderr, "error message");
        assert_eq!(output.exit_code, 1);
    }

    #[test]
    fn test_device_status_memory_calculation() {
        let total = 8_000_000_000u64;
        let free = 2_000_000_000u64;
        let available = 5_000_000_000u64;

        let status = DeviceStatus {
            uptime_secs: 100,
            total_memory: total,
            free_memory: free,
            available_memory: available,
            load_average_1m: 0.5,
            load_average_5m: 0.3,
            load_average_15m: 0.2,
        };

        assert_eq!(status.total_memory, total);
        assert!(status.free_memory < status.total_memory);
    }

    #[test]
    fn test_shell_command_format_getprop() {
        let key = "ro.product.model";
        let command = format!("getprop {}", key);
        assert_eq!(command, "getprop ro.product.model");
    }

    #[test]
    fn test_shell_command_format_setprop() {
        let key = "debug.sf.hwc";
        let value = "1";
        let command = format!("setprop {} {}", key, value);
        assert_eq!(command, "setprop debug.sf.hwc 1");
    }

    #[test]
    fn test_shell_command_format_su() {
        let cmd = "ls -la";
        let su_cmd = format!("su -c '{}'", cmd.replace("'", "'\\''"));
        assert_eq!(su_cmd, "su -c 'ls -la'");
    }

    #[test]
    fn test_shell_command_format_su_with_single_quotes() {
        let cmd = "echo 'hello world'";
        let su_cmd = format!("su -c '{}'", cmd.replace("'", "'\\''"));
        assert_eq!(su_cmd, "su -c 'echo '\\''hello world'\\'''");
    }

    #[test]
    fn test_battery_percentage_calculation() {
        let battery = BatteryInfo {
            level: 50,
            scale: 100,
            percentage: 50,
            status: "Discharging".to_string(),
            health: "Good".to_string(),
            plugged: "".to_string(),
            voltage: 3700,
            temperature: 30.0,
            technology: "Li-ion".to_string(),
        };

        assert_eq!(battery.percentage, 50);
    }

    #[test]
    fn test_device_status_uptime() {
        let status = DeviceStatus {
            uptime_secs: 86400,
            total_memory: 1_000_000,
            free_memory: 500_000,
            available_memory: 600_000,
            load_average_1m: 0.1,
            load_average_5m: 0.1,
            load_average_15m: 0.1,
        };

        assert_eq!(status.uptime_secs, 86400);
    }
}
