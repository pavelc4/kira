use crate::BuildInfo;
use crate::Storage;
use adb_client::ADBDeviceExt;
use adb_client::server_device::ADBServerDevice;

pub fn get_max_refresh_rate(device: &mut ADBServerDevice) -> Option<u32> {
    let output = shell_cmd(device, "dumpsys display")?;
    let mut max_rate = 0u32;

    for line in output.lines() {
        if line.contains("refreshRate") || line.contains("RefreshRate") {
            if let Some(rate) = extract_refresh_rate(line) {
                if rate > max_rate {
                    max_rate = rate;
                }
            }
        }
    }

    if max_rate > 0 { Some(max_rate) } else { None }
}

fn extract_refresh_rate(line: &str) -> Option<u32> {
    let line = line.to_lowercase();
    if let Some(pos) = line.find("refreshrate") {
        let rest = &line[pos + "refreshrate".len()..];
        let rest = rest
            .trim_start_matches(':')
            .trim_start_matches('=')
            .trim_start();
        let num: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        num.parse::<f64>().ok().map(|v| v as u32)
    } else {
        None
    }
}

pub fn get_storage(device: &mut ADBServerDevice) -> Option<Storage> {
    let df = shell_cmd(device, "df /data | tail -1")?;
    let parts: Vec<&str> = df.split_whitespace().collect();
    if parts.len() >= 4 {
        Some(Storage {
            total: parts.get(1)?.to_string(),
            used: parts.get(2)?.to_string(),
            free: parts.get(3)?.to_string(),
        })
    } else {
        None
    }
}

pub fn get_build_info(device: &mut ADBServerDevice) -> Option<BuildInfo> {
    Some(BuildInfo {
        security_patch: shell_cmd(device, "getprop ro.build.version.security_patch"),
        build_id: shell_cmd(device, "getprop ro.build.id"),
    })
}

pub fn parse_battery(raw: &str) -> Option<u8> {
    raw.lines()
        .find(|l| l.contains("level:"))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|s| s.trim().parse().ok())
}

pub fn shell_cmd(device: &mut ADBServerDevice, command: &str) -> Option<String> {
    let mut output = Vec::new();
    match device.shell_command(&command, Some(&mut output), None) {
        Ok(_) => {
            let result = String::from_utf8(output).ok()?.trim().to_string();
            if result.is_empty() {
                None
            } else {
                Some(result)
            }
        }
        Err(_) => None,
    }
}
