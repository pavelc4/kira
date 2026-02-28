use adb_client::{ADBDeviceExt, server_device::ADBServerDevice};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformanceError {
    CommandFailed(String),
    ParseError(String),
}

impl std::fmt::Display for PerformanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerformanceError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            PerformanceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for PerformanceError {}

fn run_shell_command(
    device: &mut ADBServerDevice,
    command: &str,
) -> Result<String, PerformanceError> {
    let mut output = Vec::new();
    device
        .shell_command(&command, Some(&mut output), None)
        .map_err(|e| PerformanceError::CommandFailed(e.to_string()))?;

    String::from_utf8(output)
        .map_err(|e| PerformanceError::ParseError(e.to_string()))
        .map(|s| s.trim().to_string())
}

pub fn get_memory_info(device: &mut ADBServerDevice) -> Result<MemoryInfo, PerformanceError> {
    let output = run_shell_command(device, "cat /proc/meminfo")?;
    parse_meminfo(&output)
        .ok_or_else(|| PerformanceError::ParseError("Failed to parse meminfo".into()))
}

pub fn get_battery_info(device: &mut ADBServerDevice) -> Result<BatteryInfo, PerformanceError> {
    let output = run_shell_command(device, "dumpsys battery")?;
    parse_battery_info(&output)
        .ok_or_else(|| PerformanceError::ParseError("Failed to parse battery info".into()))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryInfo {
    pub total_kb: u64,
    pub free_kb: u64,
    pub available_kb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatteryInfo {
    pub level: u32,
    pub temperature: u32,
    pub voltage: u32,
}

pub fn parse_meminfo(output: &str) -> Option<MemoryInfo> {
    let mut total_kb = 0;
    let mut free_kb = 0;
    let mut available_kb = 0;

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let key = parts[0];
            let value = parts[1].parse::<u64>().unwrap_or(0);
            match key {
                "MemTotal:" => total_kb = value,
                "MemFree:" => free_kb = value,
                "MemAvailable:" => available_kb = value,
                _ => {}
            }
        }
    }

    if total_kb > 0 {
        Some(MemoryInfo {
            total_kb,
            free_kb,
            available_kb,
        })
    } else {
        None
    }
}

pub fn parse_battery_info(output: &str) -> Option<BatteryInfo> {
    let mut level = 0;
    let mut temperature = 0;
    let mut voltage = 0;
    let mut found = false;

    for line in output.lines() {
        let line = line.trim();
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            if let Ok(value) = parts[1].trim().parse::<u32>() {
                match key {
                    "level" => {
                        level = value;
                        found = true;
                    }
                    "temperature" => temperature = value,
                    "voltage" => voltage = value,
                    _ => {}
                }
            }
        }
    }

    if found {
        Some(BatteryInfo {
            level,
            temperature,
            voltage,
        })
    } else {
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpuTimes {
    pub user: u64,
    pub nice: u64,
    pub sys: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpuInfo {
    pub times: CpuTimes,
    pub speed_mhz: Option<u32>,
}

pub fn parse_cpu_stat(output: &str) -> Vec<CpuInfo> {
    let mut cpus = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.starts_with("cpu") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts[0] == "cpu" {
            // Aggregate stat, skip
            continue;
        }

        if parts.len() >= 8 {
            let times = CpuTimes {
                user: parts[1].parse().unwrap_or(0),
                nice: parts[2].parse().unwrap_or(0),
                sys: parts[3].parse().unwrap_or(0),
                idle: parts[4].parse().unwrap_or(0),
                iowait: parts[5].parse().unwrap_or(0),
                irq: parts[6].parse().unwrap_or(0),
                softirq: parts[7].parse().unwrap_or(0),
            };
            cpus.push(CpuInfo {
                times,
                speed_mhz: None,
            });
        }
    }
    cpus
}

pub fn get_cpu_info(device: &mut ADBServerDevice) -> Result<Vec<CpuInfo>, PerformanceError> {
    let output = run_shell_command(device, "cat /proc/stat")?;
    let mut cpus = parse_cpu_stat(&output);

    // Optional: Fetch cpu speeds
    let cmd_speeds = "cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq";
    if let Ok(speeds_out) = run_shell_command(device, cmd_speeds) {
        let speeds: Vec<u32> = speeds_out
            .lines()
            .filter_map(|line| line.trim().parse::<u32>().ok())
            .map(|speed_khz| speed_khz / 1000)
            .collect();

        for (i, cpu) in cpus.iter_mut().enumerate() {
            if i < speeds.len() {
                cpu.speed_mhz = Some(speeds[i]);
            }
        }
    }

    Ok(cpus)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FpsData {
    pub flips: u64,
    pub timestamp_ms: u64,
}

pub fn parse_flips_count(output: &str) -> Option<u64> {
    for line in output.lines() {
        let line = line.trim();
        if let Some(idx) = line.find("flips=") {
            let remain = &line[idx + 6..];
            let digits: String = remain.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(flips) = digits.parse::<u64>() {
                return Some(flips);
            }
        }
    }
    None
}

pub fn get_flips_count(device: &mut ADBServerDevice) -> Result<FpsData, PerformanceError> {
    let output = run_shell_command(device, "dumpsys SurfaceFlinger")?;
    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    if let Some(flips) = parse_flips_count(&output) {
        Ok(FpsData {
            flips,
            timestamp_ms,
        })
    } else {
        Err(PerformanceError::ParseError(
            "Could not find flips count".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flips_count() {
        let sample_output = "Build: android
flips=123456
OtherSurface=888";
        assert_eq!(parse_flips_count(sample_output), Some(123456));

        let sample_output_inline = "Some state information flips=9992 ";
        assert_eq!(parse_flips_count(sample_output_inline), Some(9992));

        let sample_no_flips = "Build: android\nOtherSurface=888";
        assert_eq!(parse_flips_count(sample_no_flips), None);
    }

    #[test]
    fn test_parse_cpu_stat() {
        let sample_output = "cpu  416629 8243 277259 3448405 13745 66736 12224 0 0 0
        cpu0 102570 1978 72152 865261 4872 13580 3280 0 0 0
        cpu1 113337 2038 71830 855219 4697 22004 3302 0 0 0
        intr 14197288 38 693 0 0 0 0 0 0 2 0 0 0 0 0
        ctxt 23793740
        btime 1740713783
        processes 201178
        procs_running 5
        procs_blocked 0
        softirq 16010072 2 4639912 37 131849 53 0 29323 0 0 11208896
        ";

        let result = parse_cpu_stat(sample_output);
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].times.user, 102570);
        assert_eq!(result[0].times.idle, 865261);

        assert_eq!(result[1].times.nice, 2038);
        assert_eq!(result[1].times.softirq, 3302);
    }

    #[test]
    fn test_parse_meminfo() {
        let sample_output = "
        MemTotal:       11432996 kB
        MemFree:          197724 kB
        MemAvailable:    1680480 kB
        Buffers:            2796 kB
        Cached:          1639720 kB
        ";
        let expected = MemoryInfo {
            total_kb: 11432996,
            free_kb: 197724,
            available_kb: 1680480,
        };

        assert_eq!(parse_meminfo(sample_output), Some(expected));
    }

    #[test]
    fn test_parse_battery_info() {
        let sample_output = "Current Battery Service state:
        AC powered: false
        USB powered: true
        Wireless powered: false
        Max charging current: 500000
        Max charging voltage: 5000000
        Charge counter: 2000000
        status: 2
        health: 2
        present: true
        level: 85
        scale: 100
        voltage: 4123
        temperature: 320
        technology: Li-poly
        ";
        let expected = BatteryInfo {
            level: 85,
            temperature: 320, // 32.0 C
            voltage: 4123,    // 4.123 V
        };

        assert_eq!(parse_battery_info(sample_output), Some(expected));
    }
}
