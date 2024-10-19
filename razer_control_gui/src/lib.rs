//! This is duplicated stuff for now, until we have a proper project structure

use serde::{Serialize, Deserialize};

pub const DEVICE_FILE: &str = "/usr/share/razercontrol/laptops.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedDevice {
    pub name: String,
    pub vid: String,
    pub pid: String,
    pub features: Vec<String>,
    pub fan: Vec<u16>,
}
