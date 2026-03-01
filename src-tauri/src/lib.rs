use adb_client::server::ADBServer;
use adb_client::server_device::ADBServerDevice;
use kira_core::device::performance::{
    BatteryInfo, CpuInfo, FpsData, MemoryInfo, get_battery_info, get_cpu_info, get_flips_count,
    get_memory_info,
};
use kira_core::device::shell::{CommandOutput, ShellExecutor};
use kira_core::device::{
    self, AppInfo, InstallResult, PackageFilter, TopPackage, UninstallResult, get_app_info,
    install_app, list_installed_packages, uninstall_app,
};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddrV4};
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceListItem {
    pub serial: String,
    pub model: Option<String>,
}

#[command]
fn get_devices() -> Result<Vec<DeviceListItem>, String> {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);
    let mut server = ADBServer::new(addr);

    let devices = server.devices().map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for dev in devices {
        let serial = dev.identifier.clone();

        let mut device = ADBServerDevice::new(serial.clone(), None);
        let model = device::shell_cmd(&mut device, "getprop ro.product.model");

        result.push(DeviceListItem { serial, model });
    }

    Ok(result)
}

#[command]
fn get_device_info(serial: String) -> Result<device::DeviceInfo, String> {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);
    let mut server = ADBServer::new(addr);

    let devices = server.devices().map_err(|e| e.to_string())?;
    let _ = devices
        .iter()
        .find(|d| d.identifier == serial)
        .ok_or_else(|| format!("Device {} not found", serial))?;

    let mut device = ADBServerDevice::new(serial.clone(), None);

    let info = device::DeviceInfo {
        serial: serial.clone(),
        model: device::shell_cmd(&mut device, "getprop ro.product.model"),
        manufacturer: device::shell_cmd(&mut device, "getprop ro.product.manufacturer"),
        android_version: device::shell_cmd(&mut device, "getprop ro.build.version.release"),
        abi: device::shell_cmd(&mut device, "getprop ro.product.cpu.abi"),
        slot: device::shell_cmd(&mut device, "getprop ro.boot.slot_suffix"),
        battery: device::parse_battery(
            &device::shell_cmd(&mut device, "dumpsys battery | grep level").unwrap_or_default(),
        ),
        storage: device::get_storage(&mut device),
        screen_resolution: device::shell_cmd(&mut device, "wm size"),
        refresh_rate: device::get_max_refresh_rate(&mut device),
        build: device::get_build_info(&mut device),
    };

    Ok(info)
}

#[command]
fn list_packages(serial: String, filter: String) -> Result<Vec<String>, String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);

    let filter = match filter.as_str() {
        "system" => PackageFilter::System,
        "thirdparty" => PackageFilter::ThirdParty,
        "enabled" => PackageFilter::Enabled,
        "disabled" => PackageFilter::Disabled,
        _ => PackageFilter::All,
    };

    let mut device = ADBServerDevice::new(serial, None);
    list_installed_packages(&mut device, filter).map_err(|e| e.to_string())
}

#[command]
fn get_package_info(serial: String, package_name: String) -> Result<AppInfo, String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);

    let mut device = ADBServerDevice::new(serial, None);
    get_app_info(&mut device, &package_name).map_err(|e| e.to_string())
}

#[command]
fn uninstall_package(serial: String, package_name: String) -> Result<UninstallResult, String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);

    let mut device = ADBServerDevice::new(serial, None);
    uninstall_app(&mut device, &package_name).map_err(|e| e.to_string())
}

#[command]
fn install_package(serial: String, apk_path: String) -> Result<InstallResult, String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);

    let mut device = ADBServerDevice::new(serial, None);
    install_app(&mut device, &apk_path, true).map_err(|e| e.to_string())
}

#[command]
fn check_root(serial: String) -> Result<bool, String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);

    let mut device = ADBServerDevice::new(serial, None);
    Ok(device::is_rooted(&mut device))
}

#[command]
fn list_processes(serial: String) -> Result<Vec<device::ProcessInfo>, String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);

    let mut device = ADBServerDevice::new(serial, None);
    device::list_processes(&mut device).map_err(|e| e.to_string())
}

#[command]
fn kill_process(serial: String, pid: u32) -> Result<(), String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);

    let mut device = ADBServerDevice::new(serial, None);
    device::kill_process(&mut device, pid).map_err(|e| e.to_string())
}

#[command]
fn kill_package(serial: String, package_name: String) -> Result<(), String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);

    let mut device = ADBServerDevice::new(serial, None);
    device::kill_package(&mut device, &package_name).map_err(|e| e.to_string())
}

#[command]
fn reboot_device(serial: String, mode: String) -> Result<(), String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);

    let reboot_mode = match mode.as_str() {
        "recovery" => device::RebootMode::Recovery,
        "bootloader" => device::RebootMode::Bootloader,
        "fastboot" => device::RebootMode::Fastboot,
        "sideload" => device::RebootMode::Sideload,
        _ => device::RebootMode::Normal,
    };

    let mut device = ADBServerDevice::new(serial, None);
    device::reboot(&mut device, reboot_mode).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub memory: Result<MemoryInfo, String>,
    pub battery: Result<BatteryInfo, String>,
    pub cpu: Result<Vec<CpuInfo>, String>,
    pub fps: Result<FpsData, String>,
    pub uptime: Result<u64, String>,
}

#[command]
fn get_performance_profile(serial: String) -> Result<PerformanceProfile, String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);
    let mut device = ADBServerDevice::new(serial, None);

    let memory = get_memory_info(&mut device).map_err(|e| e.to_string());
    let battery = get_battery_info(&mut device).map_err(|e| e.to_string());
    let cpu = get_cpu_info(&mut device).map_err(|e| e.to_string());
    let fps = get_flips_count(&mut device).map_err(|e| e.to_string());
    let uptime = device::performance::get_uptime(&mut device).map_err(|e| e.to_string());

    Ok(PerformanceProfile {
        memory,
        battery,
        cpu,
        fps,
        uptime,
    })
}

#[command]
fn get_top_package(serial: String) -> Result<TopPackage, String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);
    let mut device = ADBServerDevice::new(serial, None);
    device::get_top_package(&mut device).map_err(|e| e.to_string())
}

#[command]
fn execute_shell_command(serial: String, command: String) -> Result<CommandOutput, String> {
    let _addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);
    let mut device = ADBServerDevice::new(serial, None);
    let mut executor = ShellExecutor::new();
    executor.execute(&mut device, &command).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_devices,
            get_device_info,
            list_packages,
            get_package_info,
            uninstall_package,
            install_package,
            check_root,
            list_processes,
            kill_process,
            kill_package,
            reboot_device,
            get_performance_profile,
            get_top_package,
            execute_shell_command,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
