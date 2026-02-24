use adb_client::server_device::ADBServerDevice;
use adb_client::ADBDeviceExt;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub permissions: String,
    pub is_directory: bool,
    pub is_symlink: bool,
    pub modified: Option<u64>,
    pub owner: Option<String>,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub path: String,
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_size: u64,
    pub files: Vec<FileInfo>,
    pub parent_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub path: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub percentage_used: f64,
    pub filesystem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_directory: bool,
    pub matched_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileType {
    pub extension: Option<String>,
    pub mime_type: String,
    pub category: FileCategory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileCategory {
    Directory,
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Apk,
    Code,
    Other,
}

pub fn list_directory(
    device: &mut ADBServerDevice,
    path: &str,
) -> Result<DirectoryListing, FileManagerError> {
    let command = format!("ls -la --time-style=+%s {}", path);
    let output = run_shell_command(device, &command)?;

    let mut files = Vec::new();
    let mut total_files = 0;
    let mut total_dirs = 0;
    let mut total_size = 0u64;

    for line in output.lines().skip(1) {
        if let Some(file_info) = parse_ls_line(line, path) {
            if file_info.is_directory {
                total_dirs += 1;
            } else {
                total_files += 1;
                total_size += file_info.size;
            }
            files.push(file_info);
        }
    }

    let parent_path = get_parent_path(path);

    Ok(DirectoryListing {
        path: path.to_string(),
        total_files,
        total_dirs,
        total_size,
        files,
        parent_path,
    })
}

pub fn get_file_info(
    device: &mut ADBServerDevice,
    path: &str,
) -> Result<FileInfo, FileManagerError> {
    let command = format!("ls -la --time-style=+%s -d {}", path);
    let output = run_shell_command(device, &command)?;

    let line = output
        .lines()
        .next()
        .ok_or_else(|| FileManagerError::FileNotFound(path.to_string()))?;

    parse_ls_line(line, path).ok_or_else(|| FileManagerError::FileNotFound(path.to_string()))
}

pub fn get_storage_info(
    device: &mut ADBServerDevice,
    path: &str,
) -> Result<StorageInfo, FileManagerError> {
    let command = format!("df -k {}", path);
    let output = run_shell_command(device, &command)?;

    for line in output.lines() {
        if line.contains(path) || line.ends_with(path.trim_start_matches('/')) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let filesystem = parts[0].to_string();
                let total_kb: u64 = parts[1].parse().unwrap_or(0);
                let used_kb: u64 = parts[2].parse().unwrap_or(0);
                let free_kb: u64 = parts[3].parse().unwrap_or(0);

                let total_bytes = total_kb * 1024;
                let used_bytes = used_kb * 1024;
                let free_bytes = free_kb * 1024;
                let percentage_used = if total_bytes > 0 {
                    (used_bytes as f64 / total_bytes as f64) * 100.0
                } else {
                    0.0
                };

                return Ok(StorageInfo {
                    path: path.to_string(),
                    total_bytes,
                    used_bytes,
                    free_bytes,
                    percentage_used,
                    filesystem,
                });
            }
        }
    }

    Err(FileManagerError::PathNotFound(path.to_string()))
}

pub fn search_files(
    device: &mut ADBServerDevice,
    base_path: &str,
    pattern: &str,
    max_depth: u32,
) -> Result<Vec<FileSearchResult>, FileManagerError> {
    let command = format!(
        "find {} -maxdepth {} -name '{}' 2>/dev/null",
        base_path, max_depth, pattern
    );
    let output = run_shell_command(device, &command)?;

    let mut results = Vec::new();

    for line in output.lines() {
        let path = line.trim();
        if path.is_empty() {
            continue;
        }

        let name = std::path::Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let is_dir = run_shell_command(device, &format!("test -d {} && echo 1 || echo 0", path))?
            .trim()
            == "1";

        let size = if is_dir {
            0
        } else {
            run_shell_command(device, &format!("stat -c %s {}", path))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0)
        };

        results.push(FileSearchResult {
            name,
            path: path.to_string(),
            size,
            is_directory: is_dir,
            matched_line: None,
        });
    }

    Ok(results)
}

pub fn search_content(
    device: &mut ADBServerDevice,
    base_path: &str,
    pattern: &str,
    file_types: Option<&[&str]>,
) -> Result<Vec<FileSearchResult>, FileManagerError> {
    let extensions = file_types.map(|t| t.join(" -o -name ")).unwrap_or_default();

    let find_cmd = if extensions.is_empty() {
        format!(
            "grep -r -l '{}' {} 2>/dev/null | head -50",
            pattern, base_path
        )
    } else {
        format!(
            "grep -r -l -E '{}' --include='*.{}' {} 2>/dev/null | head -50",
            pattern, extensions, base_path
        )
    };

    let output = run_shell_command(device, &find_cmd)?;

    let mut results = Vec::new();

    for line in output.lines() {
        let path = line.trim();
        if path.is_empty() {
            continue;
        }

        let name = std::path::Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let matched_line = get_matched_line(device, path, pattern);

        results.push(FileSearchResult {
            name,
            path: path.to_string(),
            size: 0,
            is_directory: false,
            matched_line,
        });
    }

    Ok(results)
}

pub fn get_file_type(path: &str) -> FileType {
    let extension = std::path::Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase());

    let ext = extension.as_deref();

    let (mime, category) = match ext {
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") | Some("webp")
        | Some("heic") => ("image/jpeg", FileCategory::Image),
        Some("mp4") | Some("mkv") | Some("avi") | Some("mov") | Some("webm") | Some("3gp") => {
            ("video/mp4", FileCategory::Video)
        }
        Some("mp3") | Some("wav") | Some("ogg") | Some("flac") | Some("aac") | Some("m4a") => {
            ("audio/mpeg", FileCategory::Audio)
        }
        Some("pdf") | Some("doc") | Some("docx") | Some("xls") | Some("xlsx") | Some("ppt")
        | Some("pptx") | Some("txt") => ("application/pdf", FileCategory::Document),
        Some("zip") | Some("rar") | Some("7z") | Some("tar") | Some("gz") | Some("bz2") => {
            ("application/zip", FileCategory::Archive)
        }
        Some("apk") => ("application/vnd.android.package-archive", FileCategory::Apk),
        Some("rs") | Some("js") | Some("ts") | Some("py") | Some("java") | Some("kt")
        | Some("cpp") | Some("c") | Some("h") | Some("html") | Some("css") | Some("json")
        | Some("xml") | Some("yaml") | Some("yml") | Some("toml") => {
            ("text/plain", FileCategory::Code)
        }
        _ => ("application/octet-stream", FileCategory::Other),
    };

    FileType {
        extension: ext.map(String::from),
        mime_type: mime.to_string(),
        category,
    }
}

pub fn get_quick_storage_info(
    device: &mut ADBServerDevice,
) -> Result<Vec<StorageInfo>, FileManagerError> {
    let output = run_shell_command(device, "df")?;
    let mut storages = Vec::new();

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let path = parts.last().unwrap_or(&"");
            if path.starts_with('/') && !path.contains(":/") {
                let filesystem = parts[0].to_string();
                let total_kb: u64 = parts[1].parse().unwrap_or(0);
                let used_kb: u64 = parts[2].parse().unwrap_or(0);
                let free_kb: u64 = parts[3].parse().unwrap_or(0);

                let total_bytes = total_kb * 1024;
                let used_bytes = used_kb * 1024;
                let free_bytes = free_kb * 1024;
                let percentage_used = if total_bytes > 0 {
                    (used_bytes as f64 / total_bytes as f64) * 100.0
                } else {
                    0.0
                };

                storages.push(StorageInfo {
                    path: path.to_string(),
                    total_bytes,
                    used_bytes,
                    free_bytes,
                    percentage_used,
                    filesystem,
                });
            }
        }
    }

    Ok(storages)
}

pub fn get_common_directories() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Internal Storage", "/sdcard"),
        ("Download", "/sdcard/Download"),
        ("Pictures", "/sdcard/Pictures"),
        ("DCIM", "/sdcard/DCIM"),
        ("Movies", "/sdcard/Movies"),
        ("Music", "/sdcard/Music"),
        ("Documents", "/sdcard/Documents"),
        ("Android Data", "/data/data"),
        ("APK Files", "/data/app"),
        ("Root", "/"),
    ]
}

fn parse_ls_line(line: &str, base_path: &str) -> Option<FileInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }

    let permissions = parts[0].to_string();
    let is_directory = permissions.starts_with('d');
    let is_symlink = permissions.starts_with('l');
    let owner = parts.get(2).map(|s| s.to_string());
    let group = parts.get(3).map(|s| s.to_string());
    let size: u64 = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
    let modified: Option<u64> = parts.get(5).and_then(|s| s.parse().ok());

    let name = parts[8..].join(" ");
    let name = name.trim_matches('\n').to_string();

    if name == "." || name == ".." {
        return None;
    }

    let path = if base_path.ends_with('/') {
        format!("{}{}", base_path, name)
    } else {
        format!("{}/{}", base_path, name)
    };

    Some(FileInfo {
        name,
        path,
        size,
        permissions,
        is_directory,
        is_symlink,
        modified,
        owner,
        group,
    })
}

fn get_parent_path(path: &str) -> Option<String> {
    let p = std::path::Path::new(path);
    p.parent().map(|p| p.to_string_lossy().to_string())
}

fn get_matched_line(device: &mut ADBServerDevice, path: &str, pattern: &str) -> Option<String> {
    let command = format!("grep -n '{}' {} 2>/dev/null | head -1", pattern, path);
    run_shell_command(device, &command).ok()
}

fn run_shell_command(
    device: &mut ADBServerDevice,
    command: &str,
) -> Result<String, FileManagerError> {
    let mut output = Vec::new();
    device
        .shell_command(&command, Some(&mut output), None)
        .map_err(|e| FileManagerError::CommandFailed(e.to_string()))?;

    String::from_utf8(output)
        .map_err(|e| FileManagerError::ParseError(e.to_string()))
        .map(|s| s.trim().to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileManagerError {
    PathNotFound(String),
    FileNotFound(String),
    PermissionDenied(String),
    CommandFailed(String),
    ParseError(String),
    NotADirectory(String),
}

impl std::fmt::Display for FileManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileManagerError::PathNotFound(p) => write!(f, "Path not found: {}", p),
            FileManagerError::FileNotFound(p) => write!(f, "File not found: {}", p),
            FileManagerError::PermissionDenied(p) => write!(f, "Permission denied: {}", p),
            FileManagerError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            FileManagerError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            FileManagerError::NotADirectory(p) => write!(f, "Not a directory: {}", p),
        }
    }
}

impl std::error::Error for FileManagerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_info_creation() {
        let info = FileInfo {
            name: "test.txt".to_string(),
            path: "/sdcard/test.txt".to_string(),
            size: 1024,
            permissions: "-rw-r--r--".to_string(),
            is_directory: false,
            is_symlink: false,
            modified: Some(1640000000),
            owner: Some("root".to_string()),
            group: Some("root".to_string()),
        };

        assert_eq!(info.name, "test.txt");
        assert!(!info.is_directory);
    }

    #[test]
    fn test_directory_listing_creation() {
        let listing = DirectoryListing {
            path: "/sdcard".to_string(),
            total_files: 10,
            total_dirs: 5,
            total_size: 1024000,
            files: Vec::new(),
            parent_path: Some("/".to_string()),
        };

        assert_eq!(listing.total_files, 10);
        assert_eq!(listing.total_dirs, 5);
    }

    #[test]
    fn test_storage_info_creation() {
        let storage = StorageInfo {
            path: "/data".to_string(),
            total_bytes: 64_000_000_000,
            used_bytes: 32_000_000_000,
            free_bytes: 32_000_000_000,
            percentage_used: 50.0,
            filesystem: "/dev/block/sda11".to_string(),
        };

        assert_eq!(storage.percentage_used, 50.0);
    }

    #[test]
    fn test_file_search_result_creation() {
        let result = FileSearchResult {
            name: "app.apk".to_string(),
            path: "/sdcard/Download/app.apk".to_string(),
            size: 50_000_000,
            is_directory: false,
            matched_line: None,
        };

        assert_eq!(result.size, 50_000_000);
    }

    #[test]
    fn test_get_file_type_image() {
        let file_type = get_file_type("/sdcard/photo.jpg");
        assert_eq!(file_type.category, FileCategory::Image);
        assert_eq!(file_type.extension, Some("jpg".to_string()));
    }

    #[test]
    fn test_get_file_type_apk() {
        let file_type = get_file_type("/sdcard/app.apk");
        assert_eq!(file_type.category, FileCategory::Apk);
        assert_eq!(
            file_type.mime_type,
            "application/vnd.android.package-archive"
        );
    }

    #[test]
    fn test_get_file_type_video() {
        let file_type = get_file_type("/sdcard/movie.mp4");
        assert_eq!(file_type.category, FileCategory::Video);
    }

    #[test]
    fn test_get_file_type_audio() {
        let file_type = get_file_type("/sdcard/music.mp3");
        assert_eq!(file_type.category, FileCategory::Audio);
    }

    #[test]
    fn test_get_file_type_document() {
        let file_type = get_file_type("/sdcard/document.pdf");
        assert_eq!(file_type.category, FileCategory::Document);
    }

    #[test]
    fn test_get_file_type_archive() {
        let file_type = get_file_type("/sdcard/archive.zip");
        assert_eq!(file_type.category, FileCategory::Archive);
    }

    #[test]
    fn test_get_file_type_code() {
        let file_type = get_file_type("/sdcard/main.rs");
        assert_eq!(file_type.category, FileCategory::Code);
    }

    #[test]
    fn test_get_file_type_directory() {
        let file_type = get_file_type("/sdcard/Pictures/");
        assert_eq!(file_type.category, FileCategory::Other);
    }

    #[test]
    fn test_get_file_type_unknown() {
        let file_type = get_file_type("/sdcard/weirdfile.xyz");
        assert_eq!(file_type.category, FileCategory::Other);
    }

    #[test]
    fn test_common_directories() {
        let dirs = get_common_directories();
        assert!(!dirs.is_empty());
        assert!(dirs.iter().any(|(name, _)| *name == "Internal Storage"));
        assert!(dirs.iter().any(|(name, _)| *name == "Download"));
        assert!(dirs.iter().any(|(name, _)| *name == "APK Files"));
    }

    #[test]
    fn test_get_parent_path() {
        assert_eq!(
            get_parent_path("/sdcard/Download/file.txt"),
            Some("/sdcard/Download".to_string())
        );
        assert_eq!(get_parent_path("/sdcard"), Some("/".to_string()));
        assert_eq!(get_parent_path("/"), None);
    }

    #[test]
    fn test_file_manager_error_display() {
        let err = FileManagerError::PathNotFound("/test".to_string());
        assert!(format!("{}", err).contains("/test"));

        let err2 = FileManagerError::PermissionDenied("/data".to_string());
        assert!(format!("{}", err2).contains("Permission"));

        let err3 = FileManagerError::CommandFailed("error".to_string());
        assert!(format!("{}", err3).contains("Command"));
    }

    #[test]
    fn test_storage_percentage_calculation() {
        let storage = StorageInfo {
            path: "/data".to_string(),
            total_bytes: 100_000_000_000,
            used_bytes: 75_000_000_000,
            free_bytes: 25_000_000_000,
            percentage_used: 75.0,
            filesystem: "/dev/block".to_string(),
        };

        assert!((storage.percentage_used - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_file_info_symlink() {
        let info = FileInfo {
            name: "link".to_string(),
            path: "/sdcard/link".to_string(),
            size: 0,
            permissions: "lrwxrwxrwx".to_string(),
            is_directory: false,
            is_symlink: true,
            modified: None,
            owner: None,
            group: None,
        };

        assert!(info.is_symlink);
    }

    #[test]
    fn test_file_category_variants() {
        assert_eq!(FileCategory::Directory, FileCategory::Directory);
        assert_ne!(FileCategory::Image, FileCategory::Video);
    }
}
