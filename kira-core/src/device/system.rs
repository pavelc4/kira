use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RebootMode {
    Normal,
    Recovery,
    Bootloader,
    Fastboot,
    Sideload,
    SideloadAutoReboot,
}

impl From<RebootMode> for adb_client::RebootType {
    fn from(mode: RebootMode) -> Self {
        match mode {
            RebootMode::Normal => adb_client::RebootType::System,
            RebootMode::Recovery => adb_client::RebootType::Recovery,
            RebootMode::Bootloader => adb_client::RebootType::Bootloader,
            RebootMode::Fastboot => adb_client::RebootType::Fastboot,
            RebootMode::Sideload => adb_client::RebootType::Sideload,
            RebootMode::SideloadAutoReboot => adb_client::RebootType::SideloadAutoReboot,
        }
    }
}

use adb_client::server_device::ADBServerDevice;
use adb_client::ADBDeviceExt;

pub fn reboot(device: &mut ADBServerDevice, mode: RebootMode) -> Result<(), anyhow::Error> {
    let reboot_type: adb_client::RebootType = mode.into();
    device.reboot(reboot_type)?;
    Ok(())
}
