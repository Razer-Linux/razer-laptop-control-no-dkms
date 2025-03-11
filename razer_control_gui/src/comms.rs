use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};

/// Razer laptop control socket path
pub const SOCKET_PATH: &str = "/tmp/razercontrol-socket";

#[derive(Serialize, Deserialize, Debug)]
/// Represents data sent TO the daemon
pub enum DaemonCommand {
    SetFanSpeed {
        ac: usize,
        rpm: i32,
    }, // Fan speed
    GetFanSpeed {
        ac: usize,
    }, // Get (Fan speed)
    SetPowerMode {
        ac: usize,
        pwr: u8,
        cpu: u8,
        gpu: u8,
    }, // Power mode
    GetPwrLevel {
        ac: usize,
    }, // Get (Power mode)
    GetCPUBoost {
        ac: usize,
    }, // Get (CPU boost)
    GetGPUBoost {
        ac: usize,
    }, // Get (GPU boost)
    SetLogoLedState {
        ac: usize,
        logo_state: u8,
    },
    GetLogoLedState {
        ac: usize,
    },
    GetKeyboardRGB {
        layer: i32,
    }, // Layer ID
    SetEffect {
        name: String,
        params: Vec<u8>,
    }, // Set keyboard colour
    SetStandardEffect {
        name: String,
        params: Vec<u8>,
    }, // Set keyboard colour
    SetBrightness {
        ac: usize,
        val: u8,
    },
    SetIdle {
        ac: usize,
        val: u32,
    },
    GetBrightness {
        ac: usize,
    },
    SetSync {
        sync: bool,
    },
    GetSync(),
    SetBatteryHealthOptimizer {
        is_on: bool,
        threshold: u8,
    },
    GetBatteryHealthOptimizer(),
    GetDeviceName,
}

#[derive(Serialize, Deserialize, Debug)]
/// Represents data sent back from Daemon after it receives
/// a command.
pub enum DaemonResponse {
    SetFanSpeed { result: bool },  // Response
    GetFanSpeed { rpm: i32 },      // Get (Fan speed)
    SetPowerMode { result: bool }, // Response
    GetPwrLevel { pwr: u8 },       // Get (Power mode)
    GetCPUBoost { cpu: u8 },       // Get (CPU boost)
    GetGPUBoost { gpu: u8 },       // Get (GPU boost)
    SetLogoLedState { result: bool },
    GetLogoLedState { logo_state: u8 },
    GetKeyboardRGB { layer: i32, rgbdata: Vec<u8> }, // Response (RGB) of 90 keys
    SetEffect { result: bool },                      // Set keyboard colour
    SetStandardEffect { result: bool },              // Set keyboard colour
    SetBrightness { result: bool },
    SetIdle { result: bool },
    GetBrightness { result: u8 },
    SetSync { result: bool },
    GetSync { sync: bool },
    SetBatteryHealthOptimizer { result: bool },
    GetBatteryHealthOptimizer { is_on: bool, threshold: u8 },
    GetDeviceName { name: String },
}

#[allow(dead_code)]
pub fn bind() -> Option<UnixStream> {
    UnixStream::connect(SOCKET_PATH).ok()
}

#[allow(dead_code)]
/// We use this from the app, but it should replace bind
pub fn try_bind() -> std::io::Result<UnixStream> {
    UnixStream::connect(SOCKET_PATH)
}

#[allow(dead_code)]
pub fn create() -> Option<UnixListener> {
    if std::fs::metadata(SOCKET_PATH).is_ok() {
        eprintln!("UNIX Socket already exists. Is another daemon running?");
        return None;
    }
    UnixListener::bind(SOCKET_PATH).ok().and_then(|listener| {
        let mut perms = std::fs::metadata(SOCKET_PATH).unwrap().permissions();
        // TODO(phush0): previously, set_readonly makes the permission a+w. I'm assuming this
        // is intentional. Unsure what you think the permission bits should be.
        perms.set_mode(0o777);

        std::fs::set_permissions(SOCKET_PATH, perms)
            .inspect_err(|_| {
                eprintln!("Could not set socket permissions");
            })
            .is_ok()
            .then_some(listener)
    })
}

#[allow(dead_code)]
pub fn send_to_daemon(command: DaemonCommand, mut sock: UnixStream) -> Option<DaemonResponse> {
    bincode::serialize(&command).ok().and_then(|encoded| {
        sock.write_all(&encoded)
            .inspect_err(|_| {
                eprintln!("Socket write failed!");
            })
            .ok()
            .and_then(|_| {
                let mut buf = [0u8; 4096];
                match sock.read(&mut buf) {
                    Ok(readed) if readed > 0 => read_from_socked_resp(&buf[0..readed]),
                    Ok(_) => {
                        eprintln!("No response from daemon");
                        None
                    }
                    Err(_) => {
                        eprintln!("Read failed!");
                        None
                    }
                }
            })
    })
}

/// Deserializes incomming bytes in order to return
/// a `DaemonResponse`. None is returned if deserializing failed
fn read_from_socked_resp(bytes: &[u8]) -> Option<DaemonResponse> {
    match bincode::deserialize::<DaemonResponse>(bytes) {
        Ok(res) => {
            println!("RES: {:?}", res);
            Some(res)
        }
        Err(e) => {
            println!("RES ERROR: {}", e);
            None
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
            Some(res)
        }
        Err(e) => {
            println!("REQ ERROR: {}", e);
            None
        }
    }
}
