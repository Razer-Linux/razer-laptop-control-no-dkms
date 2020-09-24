mod comms;
mod config;
// mod driver_sysfs;
mod kbd;
mod device;
use crate::kbd::Effect;
use lazy_static::lazy_static;
use signal_hook::{iterator::Signals, SIGINT, SIGTERM};
// use std::io::prelude::*;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use std::{thread, time};
use dbus::{blocking::Connection, arg};
use dbus::Message;
mod dbus_gnome_screensaver;
mod battery;
// use dbus;

lazy_static! {
    static ref EFFECT_MANAGER: Mutex<kbd::EffectManager> = Mutex::new(kbd::EffectManager::new());
    static ref CONFIG: Mutex<config::Configuration> = {
        match config::Configuration::read_from_config() {
            Ok(c) => Mutex::new(c),
            Err(_) => Mutex::new(config::Configuration::new()),
        }
    };
    static ref DEV_MANAGER: Mutex<device::DeviceManager> = {
        match device::DeviceManager::read_laptops_file() {
            Ok(c) => Mutex::new(c),
            Err(_) => Mutex::new(device::DeviceManager::new()),
        }
    };
}

fn print_refarg(value: &dyn arg::RefArg) {
    // We don't know what type the value is. We'll try a few and fall back to
    // debug printing if the value is more complex than that.
    if let Some(s) = value.as_str() { println!("as string {}", s); }
    else if let Some(i) = value.as_i64() { println!("as int {}", i); }
    else { println!("unknown {:?}", value); }
}

// Main function for daemon
fn main() {
    DEV_MANAGER.lock().unwrap().discover_devices();
    if let Some(laptop) = DEV_MANAGER.lock().unwrap().get_device() {
        println!("supported device: {:?}", laptop.get_name());
    } else {
        println!("no supported device found");
        std::process::exit(1);
    }

    // Start the keyboard animator thread,
    // This thread also periodically checks the machine power
    thread::spawn(move || {
        // let mut last_psu_status : driver_sysfs::PowerSupply = driver_sysfs::PowerSupply::UNK;
        loop {
            if let Some(laptop) = DEV_MANAGER.lock().unwrap().get_device() {
                EFFECT_MANAGER.lock().unwrap().update(laptop);
            }
            std::thread::sleep(std::time::Duration::from_millis(kbd::ANIMATION_SLEEP_MS));
            // let new_psu = driver_sysfs::read_power_source();
            // if last_psu_status != new_psu {
                // println!("Power source changed! Now {:?}", new_psu);
            // }
            // last_psu_status = new_psu;
        }
    });

    if let Ok(c) = CONFIG.lock() {
        if let Some(laptop) = DEV_MANAGER.lock().unwrap().get_device(){
            laptop.set_brightness(c.brightness);
            laptop.set_power_mode(c.power_mode, c.cpu_boost, c.gpu_boost);
            laptop.set_fan_rpm(c.fan_rpm as u16);
            laptop.set_logo_led_state(c.logo_state);
            if let Ok(json) = config::Configuration::read_effects_file() {
                EFFECT_MANAGER.lock().unwrap().load_from_save(json);
            } else {
                println!("No effects save, creating a new one");
                // No effects found, start with a green static layer, just like synapse
                EFFECT_MANAGER.lock().unwrap().push_effect(
                    kbd::effects::Static::new(vec![0, 255, 0]), 
                    [true; 90]
                    );
            }
        }
    }

    // Signal handler - cleanup if we are told to exit
    let signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
    let clean_thread = thread::spawn(move || {
        for _ in signals.forever() {
            println!("Received signal, cleaning up");
            if let Ok(mut c) = CONFIG.lock() {
                if let Err(e) = c.write_to_file() {
                    eprintln!("Error write config {:?}", e);
                }
            }
            let json = EFFECT_MANAGER.lock().unwrap().save();
            if let Err(e) = config::Configuration::write_effects_save(json) {
                eprintln!("Error write config {:?}", e);
            }
            if std::fs::metadata(comms::SOCKET_PATH).is_ok() {
                std::fs::remove_file(comms::SOCKET_PATH).unwrap();
            }
            std::process::exit(0);
        }
    });


    thread::spawn(move || {
        let dbus_session = Connection::new_session()
            .expect("failed to connect to D-Bus session bus");
        let  proxy = dbus_session.with_proxy("org.gnome.ScreenSaver", "/org/gnome/ScreenSaver", time::Duration::from_millis(5000));
        let _id = proxy.match_signal(|h: dbus_gnome_screensaver::OrgGnomeScreenSaverActiveChanged, _: &Connection, _: &Message| {
            if h.new_value {
                if let Some(laptop) = DEV_MANAGER.lock().unwrap().get_device() {
                    laptop.set_brightness(0);
                }
            } 
            else {
                if let Ok(c) = CONFIG.lock() {
                    if let Some(laptop) = DEV_MANAGER.lock().unwrap().get_device() {
                        laptop.set_brightness(c.brightness);
                    }
                }
            }
            true
        });

        loop { 
            dbus_session.process(time::Duration::from_millis(1000)).unwrap(); 
        }
    });

    thread::spawn(move || {
        let dbus_system = Connection::new_system()
            .expect("failed to connect to D-Bus system bus");
        let proxy_ac = dbus_system.with_proxy("org.freedesktop.UPower", "/org/freedesktop/UPower/devices/line_power_AC0", time::Duration::from_millis(5000));
        let _id = proxy_ac.match_signal(|h: battery::OrgFreedesktopDBusPropertiesPropertiesChanged, _: &Connection, _: &Message| {
            println!("interface name {:?}", h.interface_name);
            for (s, c) in h.changed_properties.iter() {
                println!("invalidated_property {:?}", s);
                print_refarg(c);
            }
            true
        });
        let proxy_battery = dbus_system.with_proxy("org.freedesktop.UPower", "/org/freedesktop/UPower/devices/battery_BAT0", time::Duration::from_millis(5000));
        let _id = proxy_battery.match_signal(|h: battery::OrgFreedesktopDBusPropertiesPropertiesChanged, _: &Connection, _: &Message| {
            println!("interface name {:?}", h.interface_name);
            for (s, c) in h.changed_properties.iter() {
                println!("invalidated_property {:?}", s);
                print_refarg(c);
            }
            true
        });
        loop { 
            dbus_system.process(time::Duration::from_millis(1000)).unwrap();
        }
    });


    if let Some(listener) = comms::create() {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => handle_data(stream),
                Err(_) => {} // Don't care about this
            }
        }
    } else {
        eprintln!("Could not create Unix socket!");
        std::process::exit(1);
    }
    clean_thread.join().unwrap();
}

fn handle_data(mut stream: UnixStream) {
    let mut buffer = [0 as u8; 4096];
    if stream.read(&mut buffer).is_err() {
        return;
    }

    if let Some(cmd) = comms::read_from_socket_req(&buffer) {
        if let Some(s) = process_client_request(cmd) {
            if let Ok(x) = bincode::serialize(&s) {
                stream.write_all(&x).unwrap();
            }
        }
    }
}

pub fn process_client_request(cmd: comms::DaemonCommand) -> Option<comms::DaemonResponse> {
    if let Some(laptop) = DEV_MANAGER.lock().unwrap().get_device() {
        return match cmd {
            comms::DaemonCommand::GetCfg() => {
                let fan_rpm = CONFIG.lock().unwrap().fan_rpm;
                let pwr = CONFIG.lock().unwrap().power_mode;
                Some(comms::DaemonResponse::GetCfg { fan_rpm, pwr })
            }
            comms::DaemonCommand::SetPowerMode { pwr, cpu, gpu } => {
                if let  Ok(mut x) = CONFIG.lock() {
                    x.power_mode = pwr;
                    x.cpu_boost = cpu;
                    x.gpu_boost = gpu;
                    x.write_to_file().unwrap();
                }

                Some(comms::DaemonResponse::SetPowerMode { result: laptop.set_power_mode(pwr, cpu, gpu) })
            },
            comms::DaemonCommand::SetFanSpeed { rpm } => {
                if let  Ok(mut x) = CONFIG.lock() {
                    x.fan_rpm = rpm;
                    x.write_to_file().unwrap();
                }

                Some(comms::DaemonResponse::SetFanSpeed { result: laptop.set_fan_rpm(rpm as u16) })
            },
            comms::DaemonCommand::SetLogoLedState{ logo_state } => {
                if let Ok (mut x) = CONFIG.lock() {
                    x.logo_state = logo_state;
                    x.write_to_file().unwrap();
                }

                Some(comms::DaemonResponse::SetLogoLedState { result: laptop.set_logo_led_state(logo_state) })
            },
            comms::DaemonCommand::SetBrightness { val } => {
                let _val = val as u16  * 255 / 100;
                if let Ok (mut x) = CONFIG.lock() {
                    x.brightness = _val as u8;
                    x.write_to_file().unwrap();
                }

                Some(comms::DaemonResponse::SetBrightness {result: laptop.set_brightness(_val as u8) })
            }
            comms::DaemonCommand::GetBrightness() =>  {
                let val = laptop.get_brightness() as u32;
                let mut perc = val * 100 * 100/ 255;
                perc += 50;
                perc /= 100;
                Some(comms::DaemonResponse::GetBrightness { result: perc as u8})
            },
            comms::DaemonCommand::GetLogoLedState() => Some(comms::DaemonResponse::GetLogoLedState {logo_state: laptop.get_logo_led_state() }),
            comms::DaemonCommand::GetKeyboardRGB { layer } => {
                let map = EFFECT_MANAGER.lock().unwrap().get_map(layer);
                Some(comms::DaemonResponse::GetKeyboardRGB {
                    layer,
                    rgbdata: map,
                })
            }
            comms::DaemonCommand::GetFanSpeed() => Some(comms::DaemonResponse::GetFanSpeed { rpm: laptop.get_fan_rpm() as i32 }),
            comms::DaemonCommand::GetPwrLevel() => Some(comms::DaemonResponse::GetPwrLevel { pwr: laptop.get_power_mode(0x01) }),
            comms::DaemonCommand::GetCPUBoost() => Some(comms::DaemonResponse::GetCPUBoost { cpu: laptop.get_cpu_boost() }),
            comms::DaemonCommand::GetGPUBoost() => Some(comms::DaemonResponse::GetGPUBoost { gpu: laptop.get_gpu_boost() }),
            comms::DaemonCommand::SetEffect{ name, params } => {
                let mut res = false;
                if let Ok(mut k) = EFFECT_MANAGER.lock() {
                    res = true;
                    let effect = match name.as_str() {
                        "static" => Some(kbd::effects::Static::new(params)),
                        "static_gradient" => Some(kbd::effects::StaticGradient::new(params)),
                        "wave_gradient" => Some(kbd::effects::WaveGradient::new(params)),
                        "breathing_single" => Some(kbd::effects::BreathSingle::new(params)),
                        _ => None
                    };

                    if let Some(e) = effect {
                        k.pop_effect(laptop); // Remove old layer
                        k.push_effect(
                            e,
                            [true; 90]
                            );
                    } else {
                        res = false
                    }
                }
                Some(comms::DaemonResponse::SetEffect{result: res})
            }

            comms::DaemonCommand::SetStandardEffect{ name, params } => {
                // TODO save standart effect may be struct ?
                let mut res = false;
                if let Ok(mut k) = EFFECT_MANAGER.lock() {
                    k.pop_effect(laptop); // Remove old layer
                    let _res = match name.as_str() {
                        "off" => laptop.set_standard_effect(device::RazerLaptop::OFF, params),
                        "wave" => laptop.set_standard_effect(device::RazerLaptop::WAVE, params),
                        "reactive" => laptop.set_standard_effect(device::RazerLaptop::REACTIVE, params),
                        "breathing" => laptop.set_standard_effect(device::RazerLaptop::BREATHING, params),
                        "spectrum" => laptop.set_standard_effect(device::RazerLaptop::SPECTRUM, params),
                        "static" => laptop.set_standard_effect(device::RazerLaptop::STATIC, params),
                        "starlight" => laptop.set_standard_effect(device::RazerLaptop::STARLIGHT, params), 
                        _ => false,
                    };
                    res = _res;

                }
                Some(comms::DaemonResponse::SetStandardEffect{result: res})
            }

        };
    } else {
        return None;
    }
}
