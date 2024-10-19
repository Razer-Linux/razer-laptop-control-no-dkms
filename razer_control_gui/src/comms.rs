use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};

/// Razer laptop control socket path
pub const SOCKET_PATH: &str = "/tmp/razercontrol-socket";

#[derive(Serialize, Deserialize, Debug)]
/// Represents data sent TO the daemon
pub enum DaemonCommand {
    SetFanSpeed { ac: usize, rpm: i32 },      // Fan speed
    GetFanSpeed { ac: usize },                 // Get (Fan speed)
    SetPowerMode { ac: usize, pwr: u8, cpu: u8, gpu: u8}, // Power mode
    GetPwrLevel { ac: usize },                 // Get (Power mode)
    GetCPUBoost { ac: usize },                 // Get (CPU boost)
    GetGPUBoost { ac: usize },                 // Get (GPU boost)
    SetLogoLedState{ ac:usize, logo_state: u8 },
    GetLogoLedState { ac: usize },
    GetKeyboardRGB { layer: i32 }, // Layer ID
    SetEffect { name: String, params: Vec<u8> }, // Set keyboard colour
    SetStandardEffect { name: String, params: Vec<u8> }, // Set keyboard colour
    SetBrightness { ac:usize, val: u8 },
    SetIdle {ac: usize, val: u32 },
    GetBrightness { ac: usize },
    SetSync { sync: bool },
    GetSync (),
    SetBatteryHealthOptimizer { is_on: bool, threshold: u8 },
    GetBatteryHealthOptimizer (),
    GetDeviceName 
}

#[derive(Serialize, Deserialize, Debug)]
/// Represents data sent back from Daemon after it receives
/// a command.
pub enum DaemonResponse {
    SetFanSpeed { result: bool },                    // Response
    GetFanSpeed { rpm: i32 },                        // Get (Fan speed)
    SetPowerMode { result: bool },                   // Response
    GetPwrLevel { pwr: u8 },                         // Get (Power mode)
    GetCPUBoost { cpu: u8 },                         // Get (CPU boost)
    GetGPUBoost { gpu: u8 },                         // Get (GPU boost)
    SetLogoLedState {result: bool },
    GetLogoLedState { logo_state: u8 },
    GetKeyboardRGB { layer: i32, rgbdata: Vec<u8> }, // Response (RGB) of 90 keys
    SetEffect { result: bool },                       // Set keyboard colour
    SetStandardEffect { result: bool },                       // Set keyboard colour
    SetBrightness { result: bool },
    SetIdle { result: bool },
    GetBrightness { result: u8 },
    SetSync { result: bool },
    GetSync { sync: bool },
    SetBatteryHealthOptimizer { result: bool },
    GetBatteryHealthOptimizer { is_on: bool, threshold: u8 },
    GetDeviceName { name: String }
}

#[allow(dead_code)]
pub fn bind() -> Option<UnixStream> {
    if let Ok(socket) = UnixStream::connect(SOCKET_PATH) {
        return Some(socket);
    } else {
        return None;
    }
}

#[allow(dead_code)]
/// We use this from the app, but it should replace bind
pub fn try_bind() -> std::io::Result<UnixStream> {
    UnixStream::connect(SOCKET_PATH)
}

#[allow(dead_code)]
pub fn create() -> Option<UnixListener> {
    if let Ok(_) = std::fs::metadata(SOCKET_PATH) {
        eprintln!("UNIX Socket already exists. Is another daemon running?");
        return None;
    }
    if let Ok(listener) = UnixListener::bind(SOCKET_PATH) {
        let mut perms = std::fs::metadata(SOCKET_PATH).unwrap().permissions();
        perms.set_readonly(false);
        if std::fs::set_permissions(SOCKET_PATH, perms).is_err() {
            eprintln!("Could not set socket permissions");
            return None;
        }
        return Some(listener);
    }
    return None;
}

#[allow(dead_code)]
pub fn send_to_daemon(command: DaemonCommand, mut sock: UnixStream) -> Option<DaemonResponse> {
    if let Ok(encoded) = bincode::serialize(&command) {
        if sock.write_all(&encoded).is_ok() {
            let mut buf = [0 as u8; 4096];
            return match sock.read(&mut buf) {
                Ok(readed) if readed > 0 => read_from_socked_resp(&buf[0..readed]),
                Ok(_) => {
                    eprintln!("No response from daemon");
                    None
                }
                Err(_) => {
                    eprintln!("Read failed!");
                    None
                }
            };
        } else {
            eprintln!("Socket write failed!");
        }
    }
    return None;
}

/// Deserializes incomming bytes in order to return
/// a `DaemonResponse`. None is returned if deserializing failed
fn read_from_socked_resp(bytes: &[u8]) -> Option<DaemonResponse> {
    match bincode::deserialize::<DaemonResponse>(bytes) {
        Ok(res) => {
            println!("RES: {:?}", res);
            return Some(res);
        }
        Err(e) => {
            println!("RES ERROR: {}", e);
            return None;
        }
    }
}

/// Deserializes incomming bytes in order to return
/// a `DaemonCommand`. None is returned if deserializing failed
#[allow(dead_code)]
pub fn read_from_socket_req(bytes: &[u8]) -> Option<DaemonCommand> {
    match bincode::deserialize::<DaemonCommand>(bytes) {
        Ok(res) => {
            println!("REQ: {:?}", res);
            return Some(res);
        }
        Err(e) => {
            println!("REQ ERROR: {}", e);
            return None;
        }
    }
}
