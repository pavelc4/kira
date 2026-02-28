use crate::device::{
    DeviceInfo, RebootMode, get_build_info, get_max_refresh_rate, get_storage, parse_battery,
    reboot, shell_cmd,
};
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
