mod comms;
use std::{env, format};

fn print_help(reason: &str) -> ! {
    let mut ret_code = 0;
    if reason.len() > 1 {
        println!("ERROR: {}", reason);
        ret_code = 1;
    }
    println!("Help:");
    println!("./razer-cli read <attr> <ac_state>");
    println!("./razer-cli write <attr> <ac_stete>");
    println!("./razer-cli write standard_effect <effect name> <params>");
    println!("./razer-cli write effect <effect name> <params>");
    println!("");
    println!("Where 'attr':");
    println!("- fan         -> Cooling fan RPM. 0 is automatic");
    println!("");
    println!("- power       -> Power mode.");
    println!("              0 = Balanced (Normal)");
    println!("              1 = Gaming");
    println!("              2 = Creator");
    println!("              4 = Custom ->");
    println!("                  0..3 = cpu boost");
    println!("                  0..2 = gpu boost");
    println!("");
    println!("- brightness  -> Logo mode.");
    println!("                  0..100 percents");
    println!("");
    println!("- logo        -> Logo mode.");
    println!("                  0 = Off");
    println!("                  1 = On");
    println!("                  2 = Breathing");
    println!("");
    println!("- sync        -> Sync light effects on battery and ac on/off");
    println!("");
    println!("- standard_effect:");
    println!("  -> 'off'");
    println!("  -> 'wave' - PARAMS: <Direction>");
    println!("  -> 'reactive' - PARAMS: <Speed> <Red> <Green> <Blue>");
    println!("  -> 'breathing' - PARAMS: <Type> [Red] [Green] [Blue] [Red] [Green] [Blue]");
    println!("  -> 'spectrum'");
    println!("  -> 'static' - PARAMS: <Red> <Green> <Blue>");
    println!("  -> 'starlight' - PARAMS: <Type> [Red] [Green] [Blue] [Red] [Green] [Blue]");
    println!("");
    println!("- effect:");
    println!("  -> 'static' - PARAMS: <Red> <Green> <Blue>");
    println!("  -> 'static_gradient' - PARAMS: <Red1> <Green1> <Blue1> <Red2> <Green2> <Blue2>");
    println!("  -> 'wave_gradient' - PARAMS: <Red1> <Green1> <Blue1> <Red2> <Green2> <Blue2>");
    println!("  -> 'breathing_single' - PARAMS: <Red> <Green> <Blue> <Duration_ms/100>");
    std::process::exit(ret_code);
}

fn main() {
    if std::fs::metadata(comms::SOCKET_PATH).is_err() {
        eprintln!("Error. Socket doesn't exit. Is daemon running?");
        std::process::exit(1);
    }
    let mut args : Vec<_> = env::args().collect();
    if args.len() < 3 {
        print_help("Not enough args supplied");
    }
    match args[1].to_ascii_lowercase().as_str() {
        "read" => {
            if args.len() != 4 {
                print_help("Invalid number of args supplied");
            }
            let ac: usize;
            match args[3].to_ascii_lowercase().as_str() {
                "ac" => ac = 0x01,
                "bat" => ac = 0x00,
                _ => print_help("Unknown power mode"),
            }
            match args[2].to_ascii_lowercase().as_str() {
                "fan" => read_fan_rpm(ac),
                "power" => read_power_mode(ac),
                "logo" => read_logo_mode(ac),
                "brightness" => read_brigtness(ac),
                _ => print_help(format!("Unrecognised option to read: `{}`", args[2]).as_str())
            }
        },
        "write" => {
            
            // Special case for setting effect - lots of params
            if args[2].to_ascii_lowercase().as_str() == "effect" {
                args.drain(0..3);
                write_effect(args);
                return;
            }
            if args[2].to_ascii_lowercase().as_str() == "standard_effect" {
                args.drain(0..3);
                write_standard_effect(args);
                return;
            }
            if args[2].to_ascii_lowercase().as_str() == "power" {
                let ac: usize;
                match args[3].to_ascii_lowercase().as_str() {
                    "ac" => ac = 0x01,
                    "bat" => ac = 0x00,
                    _ => print_help("Unknown power mode"),
                }

                args.drain(0..4);
                write_pwr_mode(ac, args);
                return;
            }
            if args[2].to_ascii_lowercase().as_str() == "sync" {
                if args.len() != 4 {
                    print_help("Invalid number of args supplied");
                }
                let sync: bool;
                match args[3].to_ascii_lowercase().as_str() {
                    "on" => sync = true,
                    "off" => sync = false,
                    _ => print_help("unkown parameter"),
                }

                write_sync(sync);
                return;
            }
            if args.len() != 5 {
                print_help("Invalid number of args supplied");
            }
            if let Ok(processed) = args[4].parse::<i32>() {
                let ac: usize;
                match args[3].to_ascii_lowercase().as_str() {
                    "ac" => ac = 0x01,
                    "bat" => ac = 0x00,
                    _ => print_help("Unknown power mode"),
                }
                match args[2].to_ascii_lowercase().as_str() {
                    "fan" => write_fan_speed(ac, processed),
                    "logo" => write_logo_mode(ac, processed as u8),
                    "brightness" => write_brightness(ac, processed as u8),
                    "idle" => write_idle(ac, processed as u32),
                    _ => print_help(format!("Unrecognised option to read: `{}`", args[2]).as_str())
                }
            } else {
                print_help(format!("`{}` is not a valid number", args[4]).as_str())
            }
        },
        _ => print_help(format!("Unrecognised argument: `{}`", args[1]).as_str())
    }
}
fn write_standard_effect(opt: Vec<String>) {
    println!("Write standard effect: Args: {:?}", opt);
    let name = opt[0].clone();
    let mut params : Vec<u8> = vec![];
    for i in 1..opt.len() {
        if let Ok(x) = opt[i].parse::<u8>() {
            params.push(x);
        } else {
            print_help(format!("Option for effect is not valid (Must be 0-255): `{}`", opt[i]).as_str())
        }
    }
    println!("Params: {:?}", params);
    match name.to_ascii_lowercase().as_str() {
        "off" => {
            if params.len() != 0 { print_help("No parameters are required") }
            send_standard_effect(name.to_ascii_lowercase(), params);
        },
        "wave" => {
            if params.len() != 1 { print_help("Wave require 1 parameter - direction") }
            send_standard_effect(name.to_ascii_lowercase(), params);
        },
        "reactive" => {
            if params.len() != 4 { print_help("Reactive require 4 parameters - speed r g b") }
            send_standard_effect(name.to_ascii_lowercase(), params);
        },
        "breathing" => {
            if params[0] == 1 && params.len() != 4 { print_help("Breathing single require 4 parameters - type r g b") }
            if params[0] == 2 && params.len() != 7 { print_help("Breathing double require 7 parameters - type r1 g1 b1 r2 g2 b2") }
            if params[0] == 3 && params.len() != 1 { print_help("Breathing random require 1 parameter - type") }
            send_standard_effect(name.to_ascii_lowercase(), params);
        },
        "spectrum" => {
            if params.len() != 0 { print_help("No parameters are required") }
            send_standard_effect(name.to_ascii_lowercase(), params);
        },
        "static" => {
            if params.len() != 3 { print_help("Static require 3 parameters - r, g ,b") }
            send_standard_effect(name.to_ascii_lowercase(), params);
        },
        "starlight" => {
            if params[0] == 1 && params.len() != 5 { print_help("Starlight single require 5 parameters - type speed r g b") }
            if params[0] == 2 && params.len() != 8 { print_help("Starlight double require 8 parameters - type speed r1 g1 b1 r2 g2 b2") }
            if params[0] == 3 && params.len() != 2 { print_help("Starlight random require 2 parameter - type speed") }
            send_standard_effect(name.to_ascii_lowercase(), params);
        }, 
        _ => print_help(format!("Unrecognised effect name: `{}`", name).as_str())
    }
}

fn send_standard_effect(name: String, params: Vec<u8>) {
    if let Some(r) = send_data(comms::DaemonCommand::SetStandardEffect { name, params }) {
        if let comms::DaemonResponse::SetStandardEffect { result } = r {
            match result {
                true => println!("Effect set OK!"),
                _ => eprintln!("Effect set FAIL!")
            }
        }
    } else {
        eprintln!("Unknown daemon error!");
    }
}


fn write_effect(opt: Vec<String>) {
    println!("Write effect: Args: {:?}", opt);
    let name = opt[0].clone();
    let mut params : Vec<u8> = vec![];
    for i in 1..opt.len() {
        if let Ok(x) = opt[i].parse::<u8>() {
            params.push(x)
        } else {
            print_help(format!("Option for effect is not valid (Must be 0-255): `{}`", opt[i]).as_str())
        }
    }
    println!("Params: {:?}", params);
    match name.to_ascii_lowercase().as_str() {
        "static" => {
            if params.len() != 3 { print_help("Static effect requires 3 args") }
            send_effect(name.to_ascii_lowercase(), params)
        }
        "static_gradient" => {
            if params.len() != 6 { print_help("Static gradient requires 6 args") }
            params.push(0); // Until implimented direction
            send_effect(name.to_ascii_lowercase(), params)
        }
        "wave_gradient" => {
            if params.len() != 6 { print_help("Wave gradient requires 6 args") }
            params.push(0); // Until implimented direction
            send_effect(name.to_ascii_lowercase(), params)
        }
        "breathing_single" => {
            if params.len() != 4 { print_help("Breathing single requires 4 args") }
            send_effect(name.to_ascii_lowercase(), params)
        }
        _ => print_help(format!("Unrecognised effect name: `{}`", name).as_str())
    }
}

fn send_effect(name: String, params: Vec<u8>) {
    if let Some(r) = send_data(comms::DaemonCommand::SetEffect { name, params }) {
        if let comms::DaemonResponse::SetEffect { result } = r {
            match result {
                true => println!("Effect set OK!"),
                _ => eprintln!("Effect set FAIL!")
            }
        }
    } else {
        eprintln!("Unknown daemon error!");
    }
}


fn send_data(opt: comms::DaemonCommand) -> Option<comms::DaemonResponse> {
    if let Some(socket) = comms::bind() {
        return comms::send_to_daemon(opt, socket);
    } else {
        eprintln!("Error. Cannot bind to socket");
        return None;
    }
}

fn read_fan_rpm(ac: usize) {
    if let Some(resp) = send_data(comms::DaemonCommand::GetFanSpeed {ac}) {
        if let comms::DaemonResponse::GetFanSpeed { rpm } = resp {
            let rpm_desc : String = match rpm {
                f if f < 0 => String::from("Unknown"),
                0 => String::from("Auto (0)"),
                _ => format!("{} RPM", rpm)
            };
            println!("Current fan setting: {}", rpm_desc);
        } else {
            eprintln!("Daemon responded with invalid data!");
        }
    }
}

fn read_logo_mode(ac:usize) {
    if let Some(resp) = send_data(comms::DaemonCommand::GetLogoLedState {ac}) {
        if let comms::DaemonResponse::GetLogoLedState {logo_state } = resp {
            let logo_state_desc : &str = match logo_state {
                0 => "Off",
                1 => "On",
                2 => "Breathing",
                _ => "Unknown"
            };
            println!("Current logo setting: {}", logo_state_desc);
        } else {
            eprintln!("Daemon responded with invalid data!");
        }
    }
}

fn read_power_mode(ac:usize) {
    if let Some(resp) = send_data(comms::DaemonCommand::GetPwrLevel{ac}) {
        if let comms::DaemonResponse::GetPwrLevel {pwr } = resp {
            let power_desc : &str = match pwr {
                0 => "Balanced",
                1 => "Gaming",
                2 => "Creator",
                4 => "Custom",
                _ => "Unknown"
            };
            println!("Current power setting: {}", power_desc);
            if pwr == 4 {
                if let Some(resp) = send_data(comms::DaemonCommand::GetCPUBoost{ac}) {
                    if let comms::DaemonResponse::GetCPUBoost { cpu } = resp {
                        let cpu_boost_desc : &str = match cpu {
                            0 => "Low",
                            1 => "Medium",
                            2 => "High",
                            3 => "Boost",
                            _ => "Unknown"
                        };
                        println!("Current CPU setting: {}", cpu_boost_desc);
                    };
                }
                if let Some(resp) = send_data(comms::DaemonCommand::GetGPUBoost{ac}) {
                    if let comms::DaemonResponse::GetGPUBoost {gpu } = resp {
                        let gpu_boost_desc : &str = match gpu {
                            0 => "Low",
                            1 => "Medium",
                            2 => "High",
                            _ => "Unknown"
                        };
                        println!("Current GPU setting: {}", gpu_boost_desc);
                    };
                }
            }
        } else {
            eprintln!("Daemon responded with invalid data!");
        }
    }
}

fn write_pwr_mode(ac:usize, opt: Vec<String>) {
    println!("Write power: Args: {:?}", opt);
    if let Ok(mut x) = opt[0].parse::<i8>() {
        if (x >= 0 && x <= 2) || (x == 4) {
            if ac == 0 && x != 0 {
                eprintln!("Only balanced mode can be set on battery power!!!");
                x = 0;
            }
            if x == 4
            {
                if opt.len() != 3 {
                    print_help("Invalid number of args supplied");
                }
                else {
                    if let Ok(cpu_boost) = opt[1].parse::<i8>() {
                        if cpu_boost >= 0 && cpu_boost <= 3 {
                            if let Ok(gpu_boost) = opt[2].parse::<i8>() {
                                if gpu_boost >= 0 && gpu_boost <= 2 {
                                    if let Some(_) = send_data(comms::DaemonCommand::SetPowerMode { ac, pwr: x as u8, cpu: cpu_boost as u8, gpu: gpu_boost as u8}) {
                                        read_power_mode(ac)
                                    } else {
                                        eprintln!("Unknown error!");
                                    }
                                }
                                else
                                {
                                    print_help("CPU boost must be between 0 and 2");
                                }
                            }
                        }
                        else
                        {
                            print_help("CPU boost must be between 0 and 3");
                        }
                    }
                }
            }
            else
            {
                if let Some(_) = send_data(comms::DaemonCommand::SetPowerMode { ac, pwr: x as u8, cpu: 0, gpu: 0}) {
                    read_power_mode(ac)
                } else {
                    eprintln!("Unknown error!");
                }
            }
        }
        else
        {
            print_help("Power mode must be 0, 1, 2 or 4");
        }
    } else {
        eprintln!("Unknown error!");
    }
    
}

fn read_brigtness (ac:usize) {
    if let Some(resp) = send_data(comms::DaemonCommand::GetBrightness{ac}) {
        if let comms::DaemonResponse::GetBrightness { result } = resp {
            println!("Current brightness: {}", result);
        } else {
            eprintln!("Daemon responded with invalid data!");
        }
    }
}

fn read_sync () {
    if let Some(resp) = send_data(comms::DaemonCommand::GetSync()) {
        if let comms::DaemonResponse::GetSync { sync } = resp {
            println!("Current sync: {:?}", sync);
        } else {
            eprintln!("Daemon responded with invalid data!");
        }
    }
}

fn write_brightness(ac:usize, val: u8)
{
    if let Some(_) = send_data(comms::DaemonCommand::SetBrightness { ac, val } ) {
        read_brigtness(ac)
    } else {
        eprintln!("Unknown error!");
    }
}

fn write_idle(ac: usize, val: u32)
{
    if let Some(_) = send_data(comms::DaemonCommand::SetIdle { ac, val } ) {
    } else {
        eprintln!("Unknown error!");
    }
}

fn write_fan_speed(ac: usize, x: i32) {
    if let Some(_) = send_data(comms::DaemonCommand::SetFanSpeed { ac, rpm: x }) {
        read_fan_rpm(ac)
    } else {
        eprintln!("Unknown error!");
    }
}

fn write_logo_mode(ac: usize, x: u8) {
    if let Some(_) = send_data(comms::DaemonCommand::SetLogoLedState { ac, logo_state: x }) {
        read_logo_mode(ac)
    } else {
        eprintln!("Unknown error!");
    }
}

fn write_sync(sync: bool) {
    if let Some(_) = send_data(comms::DaemonCommand::SetSync { sync }) {
        read_sync()
    } else {
        eprintln!("Unknown error!");
    }
}

/*
fn write_colour(r: u8, g: u8, b: u8) {
    if let Some(_) = send_data(comms::DaemonCommand::SetColour { r, g, b }) {
        read_fan_rpm()
    } else {
        eprintln!("Unknown error!");
    }
}
*/
