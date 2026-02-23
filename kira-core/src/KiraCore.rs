use crate::device::{get_build_info, get_max_refresh_rate, get_storage, parse_battery, reboot, shell_cmd, DeviceInfo, RebootMode};
use adb_client::server::ADBServer;
use adb_client::server_device::ADBServerDevice;
use anyhow::Result;
use std::net::{Ipv4Addr, SocketAddrV4};

pub struct KiraCore {
    server: ADBServer,
}

impl KiraCore {
    pub fn new() -> Result<Self> {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037);
        let server = ADBServer::new(addr);
        Ok(Self { server })
    }

    pub fn refresh_device(&mut self, serial: &str) -> Result<DeviceInfo> {
        let devices = self.server.devices()?;
        let _ = devices
            .iter()
            .find(|d| d.identifier == serial)
            .ok_or(anyhow::anyhow!("Device {} not found", serial))?;

        let mut device = ADBServerDevice::new(serial.to_string(), None);

        let info = DeviceInfo {
            serial: serial.to_string(),
            model: shell_cmd(&mut device, "getprop ro.product.model"),
            manufacturer: shell_cmd(&mut device, "getprop ro.product.manufacturer"),
            android_version: shell_cmd(&mut device, "getprop ro.build.version.release"),
            abi: shell_cmd(&mut device, "getprop ro.product.cpu.abi"),
            slot: shell_cmd(&mut device, "getprop ro.boot.slot_suffix"),
            battery: parse_battery(
                &shell_cmd(&mut device, "dumpsys battery | grep level").unwrap_or_default(),
            ),
            storage: get_storage(&mut device),
            screen_resolution: shell_cmd(&mut device, "wm size"),
            refresh_rate: get_max_refresh_rate(&mut device),
            build: get_build_info(&mut device),
        };

        println!("KIRA: {:?}", info);
        Ok(info)
    }

    pub fn reboot(&mut self, serial: &str, mode: RebootMode) -> Result<()> {
        let devices = self.server.devices()?;
        let _ = devices
            .iter()
            .find(|d| d.identifier == serial)
            .ok_or(anyhow::anyhow!("Device {} not found", serial))?;

        let mut device = ADBServerDevice::new(serial.to_string(), None);
        reboot(&mut device, mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_refresh_device() {
        let mut core = KiraCore::new().expect("Failed to create KiraCore");
        
        let devices = core.server.devices().expect("Failed to get devices");
        
        if devices.is_empty() {
            println!("No devices connected. Skipping test.");
            return;
        }
        
        let serial = &devices[0].identifier;
        println!("Testing with device: {}", serial);
        
        let info = core.refresh_device(serial).expect("Failed to refresh device");
        
        println!("Device Info: {:?}", info);
        
        assert!(!info.serial.is_empty());
        assert!(info.model.is_some());
    }

    #[test]
    fn test_parse_battery() {
        assert_eq!(parse_battery("level:50"), Some(50));
        assert_eq!(parse_battery("level: 75"), Some(75));
        assert_eq!(parse_battery("level:100"), Some(100));
        assert_eq!(parse_battery("level:0"), Some(0));
        assert_eq!(parse_battery("other:50"), None);
        assert_eq!(parse_battery("level:"), None);
        assert_eq!(parse_battery(""), None);
    }

    #[tokio::test]
    #[ignore] // WARNING: This will reboot the device!
    async fn test_reboot_recovery() {
        let mut core = KiraCore::new().expect("Failed to create KiraCore");
        
        let devices = core.server.devices().expect("Failed to get devices");
        
        if devices.is_empty() {
            println!("No devices connected. Skipping test.");
            return;
        }
        
        let serial = &devices[0].identifier;
        println!("Testing reboot to recovery for device: {}", serial);
        
        core.reboot(serial, RebootMode::Recovery).expect("Failed to reboot to recovery");
        
        println!("Reboot to recovery command sent!");
    }

    #[tokio::test]
    #[ignore] // WARNING: This will reboot the device!
    async fn test_reboot_bootloader() {
        let mut core = KiraCore::new().expect("Failed to create KiraCore");
        
        let devices = core.server.devices().expect("Failed to get devices");
        
        if devices.is_empty() {
            println!("No devices connected. Skipping test.");
            return;
        }
        
        let serial = &devices[0].identifier;
        println!("Testing reboot to bootloader for device: {}", serial);
        
        core.reboot(serial, RebootMode::Bootloader).expect("Failed to reboot to bootloader");
        
        println!("Reboot to bootloader command sent!");
    }

    #[tokio::test]
    #[ignore] // WARNING: This will reboot the device!
    async fn test_reboot_system() {
        let mut core = KiraCore::new().expect("Failed to create KiraCore");
        
        let devices = core.server.devices().expect("Failed to get devices");
        
        if devices.is_empty() {
            println!("No devices connected. Skipping test.");
            return;
        }
        
        let serial = &devices[0].identifier;
        println!("Testing reboot to system for device: {}", serial);
        
        core.reboot(serial, RebootMode::Normal).expect("Failed to reboot to system");
        
        println!("Reboot to system command sent!");
    }

    #[tokio::test]
    #[ignore] // WARNING: This will reboot the device!
    async fn test_reboot_fastboot() {
        let mut core = KiraCore::new().expect("Failed to create KiraCore");
        
        let devices = core.server.devices().expect("Failed to get devices");
        
        if devices.is_empty() {
            println!("No devices connected. Skipping test.");
            return;
        }
        
        let serial = &devices[0].identifier;
        println!("Testing reboot to fastboot for device: {}", serial);
        
        core.reboot(serial, RebootMode::Fastboot).expect("Failed to reboot to fastboot");
        
        println!("Reboot to fastboot command sent!");
    }

    #[tokio::test]
    #[ignore] // WARNING: This will reboot the device!
    async fn test_reboot_sideload() {
        let mut core = KiraCore::new().expect("Failed to create KiraCore");
        
        let devices = core.server.devices().expect("Failed to get devices");
        
        if devices.is_empty() {
            println!("No devices connected. Skipping test.");
            return;
        }
        
        let serial = &devices[0].identifier;
        println!("Testing reboot to sideload for device: {}", serial);
        
        core.reboot(serial, RebootMode::Sideload).expect("Failed to reboot to sideload");
        
        println!("Reboot to sideload command sent!");
    }
}
