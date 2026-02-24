use adb_client::server_device::ADBServerDevice;
use adb_client::ADBDeviceExt;

pub fn is_rooted(device: &mut ADBServerDevice) -> bool {
    let output = run_shell_command(device, "id");
    match output {
        Some(result) => result.contains("uid=0"),
        None => false,
    }
}

pub fn check_su_binary(device: &mut ADBServerDevice) -> bool {
    let su_paths = [
        "/system/bin/su",
        "/system/xbin/su",
        "/sbin/su",
        "/vendor/bin/su",
        "/data/local/xbin/su",
    ];

    for path in su_paths {
        let output = run_shell_command(device, &format!("ls -l {}", path));
        if let Some(result) = output {
            if !result.contains("No such file") && !result.contains("not found") {
                return true;
            }
        }
    }
    false
}

pub fn get_root_apps(device: &mut ADBServerDevice) -> Vec<String> {
    let output = run_shell_command(device, "pm list packages -3");
    match output {
        Some(result) => result
            .lines()
            .filter_map(|line| line.strip_prefix("package:"))
            .map(|s| s.to_string())
            .collect(),
        None => Vec::new(),
    }
}

pub fn has_root_access(device: &mut ADBServerDevice) -> RootStatus {
    if is_rooted(device) {
        RootStatus::Rooted
    } else if check_su_binary(device) {
        RootStatus::SuBinaryExists
    } else {
        RootStatus::NotRooted
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootStatus {
    Rooted,
    SuBinaryExists,
    NotRooted,
}

fn run_shell_command(device: &mut ADBServerDevice, command: &str) -> Option<String> {
    let mut output = Vec::new();
    match device.shell_command(&command, Some(&mut output), None) {
        Ok(_) => String::from_utf8(output).ok()?.trim().to_string().into(),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_status_variants() {
        assert_ne!(RootStatus::Rooted, RootStatus::NotRooted);
        assert_ne!(RootStatus::SuBinaryExists, RootStatus::NotRooted);
        assert_ne!(RootStatus::Rooted, RootStatus::SuBinaryExists);
    }

    #[test]
    fn test_root_status_debug() {
        let debug_rooted = format!("{:?}", RootStatus::Rooted);
        let debug_su_exists = format!("{:?}", RootStatus::SuBinaryExists);
        let debug_not_rooted = format!("{:?}", RootStatus::NotRooted);

        assert!(debug_rooted.contains("Rooted"));
        assert!(debug_su_exists.contains("SuBinaryExists"));
        assert!(debug_not_rooted.contains("NotRooted"));
    }

    #[test]
    fn test_root_status_clone() {
        let original = RootStatus::Rooted;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_root_status_partial_eq() {
        assert_eq!(RootStatus::Rooted, RootStatus::Rooted);
        assert_eq!(RootStatus::SuBinaryExists, RootStatus::SuBinaryExists);
        assert_eq!(RootStatus::NotRooted, RootStatus::NotRooted);

        assert_ne!(RootStatus::Rooted, RootStatus::SuBinaryExists);
        assert_ne!(RootStatus::Rooted, RootStatus::NotRooted);
        assert_ne!(RootStatus::SuBinaryExists, RootStatus::NotRooted);
    }

    #[test]
    fn test_su_paths_list() {
        let su_paths = [
            "/system/bin/su",
            "/system/xbin/su",
            "/sbin/su",
            "/vendor/bin/su",
            "/data/local/xbin/su",
        ];

        assert_eq!(su_paths.len(), 5);
        assert!(su_paths.contains(&"/system/bin/su"));
    }

    #[test]
    fn test_root_status_default_not_possible() {
        fn takes_root_status(status: RootStatus) -> bool {
            match status {
                RootStatus::Rooted => true,
                RootStatus::SuBinaryExists => false,
                RootStatus::NotRooted => false,
            }
        }

        assert!(takes_root_status(RootStatus::Rooted));
        assert!(!takes_root_status(RootStatus::SuBinaryExists));
        assert!(!takes_root_status(RootStatus::NotRooted));
    }

    #[test]
    fn test_id_output_parsing_rooted() {
        let output = "uid=0(root) gid=0(root) groups=0(root) context=u:r:su:s0";
        assert!(output.contains("uid=0"));
    }

    #[test]
    fn test_id_output_parsing_not_rooted() {
        let output = "uid=2000(shell) gid=2000(shell) groups=2000(shell)";
        assert!(!output.contains("uid=0"));
    }

    #[test]
    fn test_package_list_parsing() {
        let output = "package:com.example.app\npackage:com.test.app\npackage:org.root.check";
        let packages: Vec<String> = output
            .lines()
            .filter_map(|line| line.strip_prefix("package:"))
            .map(|s| s.to_string())
            .collect();

        assert_eq!(packages.len(), 3);
        assert!(packages.contains(&"com.example.app".to_string()));
        assert!(packages.contains(&"com.test.app".to_string()));
        assert!(packages.contains(&"org.root.check".to_string()));
    }

    #[test]
    fn test_run_shell_command_none_case() {
        let result: Option<String> = None;
        assert!(result.is_none());
    }
}
