use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceInfo {
    pub serial: String,
    pub model: Option<String>,
    pub manufacturer: Option<String>,
    pub android_version: Option<String>,
    pub abi: Option<String>,
    pub slot: Option<String>,
    pub battery: Option<u8>,
    pub storage: Option<Storage>,
    pub screen_resolution: Option<String>,
    pub refresh_rate: Option<u32>,
    pub build: Option<BuildInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Storage {
    pub total: String,
    pub used: String,
    pub free: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildInfo {
    pub security_patch: Option<String>,
    pub build_id: Option<String>,
}
