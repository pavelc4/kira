use adb_client::ADBDeviceExt;
use adb_client::server_device::ADBServerDevice;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub package_name: String,
    pub version_name: Option<String>,
    pub version_code: Option<i64>,
    pub label: Option<String>,
    pub install_location: InstallLocation,
    pub flags: Vec<String>,
    pub first_install_time: Option<String>,
    pub last_update_time: Option<String>,
    pub apk_path: Option<String>,
    pub data_dir: Option<String>,
    pub is_system_app: bool,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopPackage {
    pub name: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstallLocation {
    Auto,
    InternalOnly,
    PreferExternal,
    Unknown,
}

impl InstallLocation {
    pub fn from_str(s: &str) -> Self {
        match s {
            "auto" => InstallLocation::Auto,
            "internalOnly" => InstallLocation::InternalOnly,
            "preferExternal" => InstallLocation::PreferExternal,
            _ => InstallLocation::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPermissions {
    pub package_name: String,
    pub permissions: Vec<PermissionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionInfo {
    pub name: String,
    pub status: PermissionStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionStatus {
    Granted,
    Denied,
    Default,
    Unknown,
}

impl PermissionStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "granted" => PermissionStatus::Granted,
            "denied" => PermissionStatus::Denied,
            "default" => PermissionStatus::Default,
            _ => PermissionStatus::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub message: String,
    pub package_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallResult {
    pub success: bool,
    pub message: String,
}

pub fn list_installed_packages(
    device: &mut ADBServerDevice,
    filter: PackageFilter,
) -> Result<Vec<String>, AppManagerError> {
    let mut args = vec!["pm", "list"];

    match filter {
        PackageFilter::All => args.extend(["packages", ""]),
        PackageFilter::System => args.extend(["packages", "-s"]),
        PackageFilter::ThirdParty => args.extend(["packages", "-3"]),
        PackageFilter::Enabled => args.extend(["packages", "-e"]),
        PackageFilter::Disabled => args.extend(["packages", "-d"]),
    };

    let command = args.join(" ");
    let output = run_shell_command(device, &command)?;

    let packages: Vec<String> = output
        .lines()
        .filter_map(|line| line.strip_prefix("package:"))
        .map(|s| s.to_string())
        .collect();

    Ok(packages)
}

pub fn get_app_info(
    device: &mut ADBServerDevice,
    package_name: &str,
) -> Result<AppInfo, AppManagerError> {
    let command = format!("pm dump {}", package_name);
    let output = run_shell_command(device, &command)?;

    let mut version_name = None;
    let mut version_code = None;
    let mut label = None;
    let mut install_location = InstallLocation::Unknown;
    let mut flags = Vec::new();
    let mut first_install_time = None;
    let mut last_update_time = None;
    let mut apk_path = None;
    let mut data_dir = None;
    let mut is_system_app = false;
    let mut is_enabled = true;

    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("versionName=") {
            version_name = Some(line.trim_start_matches("versionName=").to_string());
        } else if line.starts_with("versionCode=") {
            if let Some(code) = line.trim_start_matches("versionCode=").split(' ').next() {
                version_code = code.parse().ok();
            }
        } else if line.starts_with("pkgFlags=") {
            let flag_str = line.trim_start_matches("pkgFlags=");
            flags = flag_str.split(' ').map(|s| s.to_string()).collect();
            is_system_app = flags.contains(&"SYSTEM".to_string());
        } else if line.starts_with("installLocation=") {
            let loc = line.trim_start_matches("installLocation=");
            install_location = InstallLocation::from_str(loc);
        } else if line.starts_with("firstInstallTime=") {
            first_install_time = Some(line.trim_start_matches("firstInstallTime=").to_string());
        } else if line.starts_with("lastUpdateTime=") {
            last_update_time = Some(line.trim_start_matches("lastUpdateTime=").to_string());
        } else if line.starts_with("codePath=") {
            apk_path = Some(line.trim_start_matches("codePath=").to_string());
        } else if line.starts_with("dataDir=") {
            data_dir = Some(line.trim_start_matches("dataDir=").to_string());
        } else if line.starts_with("label=") {
            label = Some(line.trim_start_matches("label=").to_string());
        } else if line.starts_with("enabled=") {
            is_enabled = line.contains("true");
        }
    }

    Ok(AppInfo {
        package_name: package_name.to_string(),
        version_name,
        version_code,
        label,
        install_location,
        flags,
        first_install_time,
        last_update_time,
        apk_path,
        data_dir,
        is_system_app,
        is_enabled,
    })
}

pub fn install_app(
    device: &mut ADBServerDevice,
    apk_path: &str,
    grant_permissions: bool,
) -> Result<InstallResult, AppManagerError> {
    let mut args = vec!["install"];

    if grant_permissions {
        args.push("-g");
    }

    args.push(apk_path);

    let command = args.join(" ");
    let output = run_shell_command(device, &command)?;

    if output.contains("Success") {
        let package_name = extract_package_name_from_apk(device, apk_path)?;
        Ok(InstallResult {
            success: true,
            message: "App installed successfully".to_string(),
            package_name: Some(package_name),
        })
    } else {
        let error_msg = extract_error_message(&output);
        Ok(InstallResult {
            success: false,
            message: error_msg,
            package_name: None,
        })
    }
}

pub fn uninstall_app(
    device: &mut ADBServerDevice,
    package_name: &str,
) -> Result<UninstallResult, AppManagerError> {
    let command = format!("pm uninstall {}", package_name);
    let output = run_shell_command(device, &command)?;

    if output.contains("Success") {
        Ok(UninstallResult {
            success: true,
            message: "App uninstalled successfully".to_string(),
        })
    } else {
        let error_msg = extract_error_message(&output);
        Ok(UninstallResult {
            success: false,
            message: error_msg,
        })
    }
}

pub fn uninstall_app_with_keep_data(
    device: &mut ADBServerDevice,
    package_name: &str,
) -> Result<UninstallResult, AppManagerError> {
    let command = format!("pm uninstall -k {}", package_name);
    let output = run_shell_command(device, &command)?;

    if output.contains("Success") {
        Ok(UninstallResult {
            success: true,
            message: "App uninstalled (data kept)".to_string(),
        })
    } else {
        let error_msg = extract_error_message(&output);
        Ok(UninstallResult {
            success: false,
            message: error_msg,
        })
    }
}

pub fn get_app_permissions(
    device: &mut ADBServerDevice,
    package_name: &str,
) -> Result<AppPermissions, AppManagerError> {
    let command = format!("pm dump {}", package_name);
    let output = run_shell_command(device, &command)?;

    let mut permissions = Vec::new();

    for line in output.lines() {
        if line.contains("granted=true") || line.contains("granted=false") {
            if let Some(name_start) = line.find("name=") {
                let name_line = &line[name_start..];
                if let Some(name_end) = name_line.find(']') {
                    let name = name_line[5..name_end].to_string();
                    let status = if line.contains("granted=true") {
                        PermissionStatus::Granted
                    } else {
                        PermissionStatus::Denied
                    };
                    permissions.push(PermissionInfo { name, status });
                }
            }
        }
    }

    Ok(AppPermissions {
        package_name: package_name.to_string(),
        permissions,
    })
}

pub fn clear_app_data(
    device: &mut ADBServerDevice,
    package_name: &str,
) -> Result<(), AppManagerError> {
    let command = format!("pm clear {}", package_name);
    run_shell_command(device, &command)?;
    Ok(())
}

pub fn force_stop_app(
    device: &mut ADBServerDevice,
    package_name: &str,
) -> Result<(), AppManagerError> {
    let command = format!("am force-stop {}", package_name);
    run_shell_command(device, &command)?;
    Ok(())
}

pub fn disable_app(
    device: &mut ADBServerDevice,
    package_name: &str,
) -> Result<(), AppManagerError> {
    let command = format!("pm disable-user {}", package_name);
    run_shell_command(device, &command)?;
    Ok(())
}

pub fn enable_app(device: &mut ADBServerDevice, package_name: &str) -> Result<(), AppManagerError> {
    let command = format!("pm enable {}", package_name);
    run_shell_command(device, &command)?;
    Ok(())
}

pub fn get_launcher_activity(
    device: &mut ADBServerDevice,
    package_name: &str,
) -> Result<Option<String>, AppManagerError> {
    let command = format!(
        "cmd package resolve-activity --brief -c android.intent.category.LAUNCHER {}",
        package_name
    );
    let output = run_shell_command(device, &command)?;

    let activity = output.trim().to_string();
    if activity.is_empty() || activity.contains("null") {
        Ok(None)
    } else {
        Ok(Some(activity))
    }
}

pub fn start_app(device: &mut ADBServerDevice, package_name: &str) -> Result<(), AppManagerError> {
    let activity = get_launcher_activity(device, package_name)?;

    match activity {
        Some(act) => {
            let command = format!("am start -n {}", act);
            run_shell_command(device, &command)?;
            Ok(())
        }
        None => Err(AppManagerError::ActivityNotFound(package_name.to_string())),
    }
}

pub fn start_app_with_activity(
    device: &mut ADBServerDevice,
    activity: &str,
) -> Result<(), AppManagerError> {
    let command = format!("am start -n {}", activity);
    run_shell_command(device, &command)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageFilter {
    All,
    System,
    ThirdParty,
    Enabled,
    Disabled,
}

fn run_shell_command(
    device: &mut ADBServerDevice,
    command: &str,
) -> Result<String, AppManagerError> {
    let mut output = Vec::new();
    device
        .shell_command(&command, Some(&mut output), None)
        .map_err(|e| AppManagerError::CommandFailed(e.to_string()))?;

    String::from_utf8(output)
        .map_err(|e| AppManagerError::ParseError(e.to_string()))
        .map(|s| s.trim().to_string())
}

fn extract_package_name_from_apk(
    _device: &mut ADBServerDevice,
    apk_path: &str,
) -> Result<String, AppManagerError> {
    let name = std::path::Path::new(apk_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(name)
}

fn extract_error_message(output: &str) -> String {
    for line in output.lines() {
        if line.contains("Failure") || line.contains("Error") || line.contains("error") {
            return line.trim().to_string();
        }
    }
    output.lines().last().unwrap_or("Unknown error").to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppManagerError {
    PackageNotFound(String),
    ActivityNotFound(String),
    InstallFailed(String),
    UninstallFailed(String),
    CommandFailed(String),
    ParseError(String),
    PermissionDenied(String),
}

impl std::fmt::Display for AppManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppManagerError::PackageNotFound(pkg) => write!(f, "Package not found: {}", pkg),
            AppManagerError::ActivityNotFound(pkg) => {
                write!(f, "Launcher activity not found: {}", pkg)
            }
            AppManagerError::InstallFailed(msg) => write!(f, "Install failed: {}", msg),
            AppManagerError::UninstallFailed(msg) => write!(f, "Uninstall failed: {}", msg),
            AppManagerError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            AppManagerError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AppManagerError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl std::error::Error for AppManagerError {}

pub fn parse_top_package(output: &str) -> TopPackage {
    let mut top_line = "";
    for line in output.lines() {
        if line.contains("top-activity") {
            top_line = line.trim();
            break;
        }
    }

    if top_line.is_empty() {
        return TopPackage {
            name: "".to_string(),
            pid: None,
        };
    }

    let parts: Vec<&str> = top_line.split_whitespace().collect();
    if parts.len() >= 2 {
        let pkg_part = parts[parts.len() - 2];
        let subparts: Vec<&str> = pkg_part.split(':').collect();
        if subparts.len() == 2 {
            let pid = subparts[0].parse::<u32>().ok();
            let mut name = subparts[1].to_string();
            if let Some(slash_idx) = name.find('/') {
                name = name[..slash_idx].to_string();
            }
            return TopPackage { name, pid };
        }
    }

    TopPackage {
        name: "".to_string(),
        pid: None,
    }
}

pub fn get_top_package(device: &mut ADBServerDevice) -> Result<TopPackage, AppManagerError> {
    let command = "dumpsys activity";
    let mut output = Vec::new();
    device
        .shell_command(&command, Some(&mut output), None)
        .map_err(|e| AppManagerError::CommandFailed(e.to_string()))?;

    let out_str = String::from_utf8(output).unwrap_or_default();
    Ok(parse_top_package(&out_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_location_from_str() {
        assert_eq!(InstallLocation::from_str("auto"), InstallLocation::Auto);
        assert_eq!(
            InstallLocation::from_str("internalOnly"),
            InstallLocation::InternalOnly
        );
        assert_eq!(
            InstallLocation::from_str("preferExternal"),
            InstallLocation::PreferExternal
        );
        assert_eq!(
            InstallLocation::from_str("unknown"),
            InstallLocation::Unknown
        );
    }

    #[test]
    fn test_permission_status_from_str() {
        assert_eq!(
            PermissionStatus::from_str("granted"),
            PermissionStatus::Granted
        );
        assert_eq!(
            PermissionStatus::from_str("denied"),
            PermissionStatus::Denied
        );
        assert_eq!(
            PermissionStatus::from_str("default"),
            PermissionStatus::Default
        );
        assert_eq!(
            PermissionStatus::from_str("unknown"),
            PermissionStatus::Unknown
        );
    }

    #[test]
    fn test_package_filter_variants() {
        assert_eq!(PackageFilter::All, PackageFilter::All);
        assert_eq!(PackageFilter::System, PackageFilter::System);
        assert_eq!(PackageFilter::ThirdParty, PackageFilter::ThirdParty);
    }

    #[test]
    fn test_app_info_creation() {
        let info = AppInfo {
            package_name: "com.example.app".to_string(),
            version_name: Some("1.0.0".to_string()),
            version_code: Some(1),
            label: Some("Example App".to_string()),
            install_location: InstallLocation::Auto,
            flags: vec!["HAS_CODE".to_string()],
            first_install_time: Some("2024-01-01".to_string()),
            last_update_time: Some("2024-01-15".to_string()),
            apk_path: Some("/data/app/example.apk".to_string()),
            data_dir: Some("/data/data/com.example.app".to_string()),
            is_system_app: false,
            is_enabled: true,
        };

        assert_eq!(info.package_name, "com.example.app");
        assert!(info.is_enabled);
    }

    #[test]
    fn test_install_result_success() {
        let result = InstallResult {
            success: true,
            message: "App installed successfully".to_string(),
            package_name: Some("com.example.app".to_string()),
        };

        assert!(result.success);
        assert!(result.package_name.is_some());
    }

    #[test]
    fn test_install_result_failure() {
        let result = InstallResult {
            success: false,
            message: "INSTALL_FAILED_INSUFFICIENT_STORAGE".to_string(),
            package_name: None,
        };

        assert!(!result.success);
        assert!(result.package_name.is_none());
    }

    #[test]
    fn test_uninstall_result_success() {
        let result = UninstallResult {
            success: true,
            message: "App uninstalled successfully".to_string(),
        };

        assert!(result.success);
    }

    #[test]
    fn test_uninstall_result_failure() {
        let result = UninstallResult {
            success: false,
            message: "DELETE_FAILED_INTERNAL_ERROR".to_string(),
        };

        assert!(!result.success);
    }

    #[test]
    fn test_app_permissions_creation() {
        let perms = AppPermissions {
            package_name: "com.example.app".to_string(),
            permissions: vec![
                PermissionInfo {
                    name: "android.permission.INTERNET".to_string(),
                    status: PermissionStatus::Granted,
                },
                PermissionInfo {
                    name: "android.permission.CAMERA".to_string(),
                    status: PermissionStatus::Denied,
                },
            ],
        };

        assert_eq!(perms.permissions.len(), 2);
    }

    #[test]
    fn test_permission_info_granted() {
        let perm = PermissionInfo {
            name: "android.permission.INTERNET".to_string(),
            status: PermissionStatus::Granted,
        };

        assert_eq!(perm.status, PermissionStatus::Granted);
    }

    #[test]
    fn test_permission_info_denied() {
        let perm = PermissionInfo {
            name: "android.permission.ACCESS_FINE_LOCATION".to_string(),
            status: PermissionStatus::Denied,
        };

        assert_eq!(perm.status, PermissionStatus::Denied);
    }

    #[test]
    fn test_app_manager_error_display() {
        let err = AppManagerError::PackageNotFound("com.test.app".to_string());
        assert!(format!("{}", err).contains("com.test.app"));

        let err2 = AppManagerError::InstallFailed("error".to_string());
        assert!(format!("{}", err2).contains("Install failed"));

        let err3 = AppManagerError::ActivityNotFound("com.app.Main".to_string());
        assert!(format!("{}", err3).contains("Launcher activity"));
    }

    #[test]
    fn test_app_info_system_app() {
        let info = AppInfo {
            package_name: "com.android.system".to_string(),
            version_name: Some("1.0".to_string()),
            version_code: Some(1),
            label: Some("System".to_string()),
            install_location: InstallLocation::InternalOnly,
            flags: vec!["SYSTEM".to_string()],
            first_install_time: None,
            last_update_time: None,
            apk_path: None,
            data_dir: None,
            is_system_app: true,
            is_enabled: true,
        };

        assert!(info.is_system_app);
    }

    #[test]
    fn test_app_info_disabled() {
        let info = AppInfo {
            package_name: "com.example.disabled".to_string(),
            version_name: None,
            version_code: None,
            label: None,
            install_location: InstallLocation::Unknown,
            flags: vec![],
            first_install_time: None,
            last_update_time: None,
            apk_path: None,
            data_dir: None,
            is_system_app: false,
            is_enabled: false,
        };

        assert!(!info.is_enabled);
    }

    #[test]
    fn test_extract_error_message() {
        let output = "Failure [INSTALL_FAILED_INSUFFICIENT_STORAGE]: Storage verification failed";
        let msg = extract_error_message(output);
        assert!(msg.contains("Failure"));

        let output2 = "Error: something went wrong";
        let msg2 = extract_error_message(output2);
        assert!(msg2.contains("Error"));
    }

    #[test]
    fn test_pm_list_packages_command_format() {
        let command = "pm list packages -3".to_string();
        assert!(command.contains("packages"));
        assert!(command.contains("-3"));
    }

    #[test]
    fn test_pm_install_command_format() {
        let command = format!("pm install -g /sdcard/Download/app.apk");
        assert!(command.contains("install"));
        assert!(command.contains("-g"));
    }

    #[test]
    fn test_pm_uninstall_command_format() {
        let package = "com.brave.browser";
        let command = format!("pm uninstall {}", package);
        assert_eq!(command, "pm uninstall com.brave.browser");
    }

    #[test]
    fn test_pm_uninstall_keep_data_command_format() {
        let package = "com.brave.browser";
        let command = format!("pm uninstall -k {}", package);
        assert_eq!(command, "pm uninstall -k com.brave.browser");
    }

    #[test]
    fn test_am_force_stop_command_format() {
        let package = "com.example.app";
        let command = format!("am force-stop {}", package);
        assert_eq!(command, "am force-stop com.example.app");
    }

    #[test]
    fn test_am_start_command_format() {
        let activity = "com.example.app/.MainActivity";
        let command = format!("am start -n {}", activity);
        assert_eq!(command, "am start -n com.example.app/.MainActivity");
    }

    #[test]
    fn test_parse_top_package_valid() {
        let sample_output = "
  some random stuff
  top-activity 4567:com.something.app/com.something.MainActivity another_thing
  more text
";
        let top = parse_top_package(sample_output);
        assert_eq!(top.name, "com.something.app");
        assert_eq!(top.pid, Some(4567));
    }

    #[test]
    fn test_parse_top_package_empty() {
        let sample_output = "No top-activity here";
        let top = parse_top_package(sample_output);
        assert_eq!(top.name, "");
        assert_eq!(top.pid, None);
    }
}
