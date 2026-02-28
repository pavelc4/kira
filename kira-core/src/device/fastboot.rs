use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastbootDeviceInfo {
    pub serial: String,
    pub product: Option<String>,
    pub model: Option<String>,
    pub device: Option<String>,
    pub bootloader: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlashPartition {
    Boot,
    System,
    Recovery,
    Vendor,
    Custom(String),
}

impl FlashPartition {
    pub fn as_str(&self) -> &str {
        match self {
            FlashPartition::Boot => "boot",
            FlashPartition::System => "system",
            FlashPartition::Recovery => "recovery",
            FlashPartition::Vendor => "vendor",
            FlashPartition::Custom(name) => name,
        }
    }
}

#[derive(Debug, Error)]
pub enum FastbootError {
    #[error("No device found")]
    NoDevice,
    #[error("Multiple devices found")]
    MultipleDevices,
    #[error("Fastboot error: {0}")]
    CommandError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Fastboot protocol error: {0}")]
    ProtocolError(String),
}

pub struct FastbootCore {
    device: Option<fastboot_protocol::nusb::NusbFastBoot>,
}

impl FastbootCore {
    pub fn new() -> Result<Self, FastbootError> {
        Ok(Self { device: None })
    }

    pub async fn list_devices() -> Result<Vec<FastbootDeviceInfo>, FastbootError> {
        let mut devices = Vec::new();

        let fb_devices = fastboot_protocol::nusb::devices()
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        for info in fb_devices {
            let serial = info
                .serial_number()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            devices.push(FastbootDeviceInfo {
                serial,
                product: None,
                model: None,
                device: None,
                bootloader: None,
                version: None,
            });
        }

        Ok(devices)
    }

    pub async fn connect(&mut self, serial: Option<&str>) -> Result<(), FastbootError> {
        let mut fb_devices = fastboot_protocol::nusb::devices()
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        let info = match serial {
            Some(s) => fb_devices
                .find(|info| info.serial_number().map(|sn| sn == s).unwrap_or(false))
                .ok_or(FastbootError::NoDevice)?,
            None => fb_devices.next().ok_or(FastbootError::NoDevice)?,
        };

        let fb = fastboot_protocol::nusb::NusbFastBoot::from_info(&info)
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        self.device = Some(fb);
        Ok(())
    }

    pub async fn get_var(&mut self, var: &str) -> Result<String, FastbootError> {
        let device = self.device.as_mut().ok_or(FastbootError::NoDevice)?;

        let value = device
            .get_var(var)
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        Ok(value)
    }

    pub async fn get_all_vars(&mut self) -> Result<FastbootDeviceInfo, FastbootError> {
        let device = self.device.as_mut().ok_or(FastbootError::NoDevice)?;

        let serial = device
            .get_var("serialno")
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        let product = device.get_var("product").await.ok();
        let model = device.get_var("model").await.ok();
        let dev = device.get_var("device").await.ok();
        let bootloader = device.get_var("bootloader").await.ok();
        let version = device.get_var("version").await.ok();

        Ok(FastbootDeviceInfo {
            serial,
            product,
            model,
            device: dev,
            bootloader,
            version,
        })
    }

    pub async fn flash(
        &mut self,
        partition: FlashPartition,
        image_path: &str,
    ) -> Result<(), FastbootError> {
        let device = self.device.as_mut().ok_or(FastbootError::NoDevice)?;

        let data = std::fs::read(image_path).map_err(|e| FastbootError::IoError(e))?;

        let size = data.len() as u32;

        let mut downloader = device
            .download(size)
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        downloader
            .extend_from_slice(&data)
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        device
            .flash(partition.as_str())
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        Ok(())
    }

    pub async fn erase(&mut self, partition: FlashPartition) -> Result<(), FastbootError> {
        let device = self.device.as_mut().ok_or(FastbootError::NoDevice)?;

        device
            .erase(partition.as_str())
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        Ok(())
    }

    pub async fn reboot(&mut self) -> Result<(), FastbootError> {
        let device = self.device.as_mut().ok_or(FastbootError::NoDevice)?;

        device
            .reboot()
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        Ok(())
    }

    pub async fn continue_boot(&mut self) -> Result<(), FastbootError> {
        let device = self.device.as_mut().ok_or(FastbootError::NoDevice)?;

        device
            .continue_boot()
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_var_partition_type(
        &mut self,
        partition: &str,
    ) -> Result<String, FastbootError> {
        let device = self.device.as_mut().ok_or(FastbootError::NoDevice)?;

        let var_name = format!("partition-type:{}", partition);
        device
            .get_var(&var_name)
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))
    }

    pub async fn is_slot_supported(&mut self, slot: &str) -> Result<bool, FastbootError> {
        let var_name = format!("has-slot:{}", slot);
        match self.get_var(&var_name).await {
            Ok(val) => Ok(val == "yes"),
            Err(_) => Ok(false),
        }
    }

    pub async fn reboot_bootloader(&mut self) -> Result<(), FastbootError> {
        let device = self.device.as_mut().ok_or(FastbootError::NoDevice)?;

        device
            .reboot_bootloader()
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        Ok(())
    }

    pub async fn powerdown(&mut self) -> Result<(), FastbootError> {
        let device = self.device.as_mut().ok_or(FastbootError::NoDevice)?;

        // Use get_var to send raw command
        let _ = device
            .get_var("powerdown")
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()));

        Ok(())
    }

    pub async fn wipe_userdata(&mut self) -> Result<(), FastbootError> {
        let device = self.device.as_mut().ok_or(FastbootError::NoDevice)?;

        // Erase userdata and cache
        device
            .erase("userdata")
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()))?;

        let _ = device
            .erase("cache")
            .await
            .map_err(|e| FastbootError::ProtocolError(e.to_string()));

        Ok(())
    }
}

impl Default for FastbootCore {
    fn default() -> Self {
        Self::new().expect("Failed to create FastbootCore")
    }
}
