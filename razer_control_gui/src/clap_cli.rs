mod comms;
use clap::{command, value_parser, Arg, ArgAction, Command};
use std::{format};

fn print_help(reason: &str) -> ! {
    let mut ret_code = 0;

    if reason.len() > 1 {
        println!("ERROR: {}", reason);
        ret_code = 1;
    }

    let blurb: &'static str = "Help:
./razer-cli read <attr> <ac_state> <params>
./razer-cli write <attr> <ac_state> <params>
./razer-cli write standard_effect <effect name> <params>
./razer-cli write effect <effect name> <params>

Where 'attr':
- fan         -> Cooling fan RPM. 0 is automatic
- power       -> Power mode.
              0 = Balanced (Normal)
              1 = Gaming
              2 = Creator
              4 = Custom ->
                  0..3 = cpu boost
                  0..2 = gpu boost
- brightness  -> Keyboard brightness. 
                  0..100 percents
- logo        -> Logo mode.
                  0 = Off
                  1 = On
                  2 = Breathing
- sync        -> Sync light effects on battery and ac on/off
- standard_effect:
  -> 'off'
  -> 'wave' - PARAMS: <Direction>
  -> 'reactive' - PARAMS: <Speed> <Red> <Green> <Blue>
  -> 'breathing' - PARAMS: <Type> [Red] [Green] [Blue] [Red] [Green] [Blue]
  -> 'spectrum'
  -> 'static' - PARAMS: <Red> <Green> <Blue>
  -> 'starlight' - PARAMS: <Type> [Red] [Green] [Blue] [Red] [Green] [Blue]
- effect:
  -> 'static' - PARAMS: <Red> <Green> <Blue>
  -> 'static_gradient' - PARAMS: <Red1> <Green1> <Blue1> <Red2> <Green2> <Blue2>
  -> 'wave_gradient' - PARAMS: <Red1> <Green1> <Blue1> <Red2> <Green2> <Blue2>
  -> 'breathing_single' - PARAMS: <Red> <Green> <Blue> <Duration_ms/100>
- bho:
  -> 'on' - PARAMS: <threshold [50, 80] multiples of 5>
  -> 'off' 
Where 'ac_state':
- ac
- bat";

    println!("{}", blurb);

    std::process::exit(ret_code);
}

fn main() {
    let matches = Command::new("razer-cli")
        .author("phush0")
        .version("1.0.0")
        .about("An application to control hardware features of Razer notebooks.")
        .arg(
            Arg::new("action")
                .required(true)
                .value_parser(clap::builder::PossibleValuesParser::new(["get", "set"])),
        )
        .arg(Arg::new("attribute").required(true).value_parser(
            clap::builder::PossibleValuesParser::new([
                "fan",
                "power",
                "brightness",
                "logo",
                "sync",
                "effect",
                "color",
            ]),
        ))
        .arg(
            Arg::new("power_config").required(false).value_parser(
            clap::builder::PossibleValuesParser::new([
                "ac",
                "battery"
            ]),
        ))
        .arg(Arg::new("args").num_args(0..).trailing_var_arg(true))
        .get_matches();

    if std::fs::metadata(comms::SOCKET_PATH).is_err() {
        eprintln!("Error. Socket doesn't exit. Is daemon running?");
        std::process::exit(1);
    }

    let action: String = matches.get_one::<String>("action").expect("You must provide an action to perform. Valid options are get, set.").to_string();
    let attribute: String = matches.get_one::<String>("attribute").expect("You must provide an attribute to modify.").to_string();
    let power_config: Option<usize> = if let Some(config) = matches.get_one::<String>("power_config") {
        match config.as_str() {
            "ac" => Some(0x01),
            "battery" => Some(0x00),
            _ => todo!()
        }
    } else {
        None
    };
    let no_config_err = "This attribute requires a power config setting, either ac or battery.";

    let args = matches
        .get_many::<String>("args");

    match action.as_str() {

        "get" => match attribute.as_str() {
            "fan" => {
                read_fan_rpm(power_config.expect(no_config_err));
            }
            "ac" => {
                read_power_mode(power_config.expect(no_config_err))
            },
            "battery" => {
                read_power_mode(power_config.expect(no_config_err))
            },
            "brightness" => {
                read_brigtness(power_config.expect(no_config_err))
            },
            "logo" => {
                read_logo_mode(power_config.expect(no_config_err))
            },
            "sync" => read_sync(),
            "effect" => todo!(),
            "color" => todo!(),
            _ => todo!()
        },
        "set" => match attribute.as_str() {
            "fan" => {
                let mut all_args = args.expect("No fan setting provided!\n0 = Auto, anything else is RPM.").map(|s| s.as_str());
                let setting = all_args.next().unwrap().parse::<i32>().expect("Could not parse fan speed argument as an integer!");
                write_fan_speed(power_config.expect(no_config_err), setting);
            },
            "power" => {
                let all_args: Vec<String> = args.expect("No power setting provided!\n0 = Balanced, 1 = Gaming, 2 = Creator, 4 = Custom.\nIf set to 4, an additional two parameters are expected for CPU and GPU boost settings, respectively.\n0 = low power, 1 = normal, 2 = high, 3 = boost (only for CPU and only for Advanced 2020 model and Studio Edition)").map(String::from).collect();
                write_pwr_mode(power_config.expect(no_config_err), all_args);
            },
            "brightness" => {
                let mut all_args = args.expect("No brightness setting provided!").map(|s| s.as_str());
                let setting = all_args.next().unwrap().parse::<u8>().expect("Could not parse brightness argument as an integer!");
                write_brightness(power_config.expect(no_config_err), setting);
            },
            "logo" => {
                let mut all_args = args.expect("No logo mode provided!\n0 = off, 1 = on, 2 = breathing").map(|s| s.as_str());
                let setting = all_args.next().unwrap().parse::<u8>().expect("Could not parse logo argument as an integer!");
                write_logo_mode(power_config.expect(no_config_err), setting);
            },
            "sync" => todo!(),
            "effect" => todo!(),
            "color" => todo!(),
            _ => todo!()
        },
        _ => todo!()
    }
}

fn read_bho() {
    send_data(comms::DaemonCommand::GetBatteryHealthOptimizer()).map_or_else(
        || eprintln!("Unknown error occured when getting bho"),
        |result| {
            if let comms::DaemonResponse::GetBatteryHealthOptimizer { is_on, threshold } = result {
                match is_on {
                    true => {
                        println!(
                            "Battery health optimization is on with a threshold of {}",
                            threshold
                        );
                    }
                    false => {
                        eprintln!("Battery health optimization is off");
                    }
                }
            }
        },
    );
}

fn write_bho(on: bool, threshold: u8) {
    if !on {
        bho_toggle_off();
        return;
    }

    bho_toggle_on(threshold);
}

fn bho_toggle_on(threshold: u8) {
    if !valid_bho_threshold(threshold) {
        eprintln!("Threshold value must be a multiple of five between 50 and 80");
        return;
    }

    send_data(comms::DaemonCommand::SetBatteryHealthOptimizer {
        is_on: true,
        threshold: threshold,
    })
    .map_or_else(
        || eprintln!("Unknown error occured when toggling bho"),
        |result| {
            if let comms::DaemonResponse::SetBatteryHealthOptimizer { result } = result {
                match result {
                    true => {
                        println!(
                            "Battery health optimization is on with a threshold of {}",
                            threshold
                        );
                    }
                    false => {
                        eprintln!("Failed to turn on bho with threshold of {}", threshold);
                    }
                }
            }
        },
    );
}

fn valid_bho_threshold(threshold: u8) -> bool {
    if threshold % 5 != 0 {
        return false;
    }

    if threshold < 50 || threshold > 80 {
        return false;
    }

    return true;
}

fn bho_toggle_off() {
    send_data(comms::DaemonCommand::SetBatteryHealthOptimizer {
        is_on: false,
        threshold: 80,
    })
    .map_or_else(
        || eprintln!("Unknown error occured when toggling bho"),
        |result| {
            if let comms::DaemonResponse::SetBatteryHealthOptimizer { result } = result {
                match result {
                    true => {
                        println!("Successfully turned off bho");
                    }
                    false => {
                        eprintln!("Failed to turn off bho");
                    }
                }
            }
        },
    );
}

fn write_standard_effect(opt: Vec<String>) {
    println!("Write standard effect: Args: {:?}", opt);
    let name = opt[0].clone();
    let mut params: Vec<u8> = vec![];
    for i in 1..opt.len() {
        if let Ok(x) = opt[i].parse::<u8>() {
            params.push(x);
        } else {
            print_help(
                format!(
                    "Option for effect is not valid (Must be 0-255): `{}`",
                    opt[i]
                )
                .as_str(),
            )
        }
    }
    println!("Params: {:?}", params);
    match name.to_ascii_lowercase().as_str() {
        "off" => {
            if params.len() != 0 {
                print_help("No parameters are required")
            }
            send_standard_effect(name.to_ascii_lowercase(), params);
        }
        "wave" => {
            if params.len() != 1 {
                print_help("Wave require 1 parameter - direction")
            }
            send_standard_effect(name.to_ascii_lowercase(), params);
        }
        "reactive" => {
            if params.len() != 4 {
                print_help("Reactive require 4 parameters - speed r g b")
            }
            send_standard_effect(name.to_ascii_lowercase(), params);
        }
        "breathing" => {
            if params[0] == 1 && params.len() != 4 {
                print_help("Breathing single require 4 parameters - type r g b")
            }
            if params[0] == 2 && params.len() != 7 {
                print_help("Breathing double require 7 parameters - type r1 g1 b1 r2 g2 b2")
            }
            if params[0] == 3 && params.len() != 1 {
                print_help("Breathing random require 1 parameter - type")
            }
            send_standard_effect(name.to_ascii_lowercase(), params);
        }
        "spectrum" => {
            if params.len() != 0 {
                print_help("No parameters are required")
            }
            send_standard_effect(name.to_ascii_lowercase(), params);
        }
        "static" => {
            if params.len() != 3 {
                print_help("Static require 3 parameters - r, g ,b")
            }
            send_standard_effect(name.to_ascii_lowercase(), params);
        }
        "starlight" => {
            if params[0] == 1 && params.len() != 5 {
                print_help("Starlight single require 5 parameters - type speed r g b")
            }
            if params[0] == 2 && params.len() != 8 {
                print_help("Starlight double require 8 parameters - type speed r1 g1 b1 r2 g2 b2")
            }
            if params[0] == 3 && params.len() != 2 {
                print_help("Starlight random require 2 parameter - type speed")
            }
            send_standard_effect(name.to_ascii_lowercase(), params);
        }
        _ => print_help(format!("Unrecognised effect name: `{}`", name).as_str()),
    }
}

fn send_standard_effect(name: String, params: Vec<u8>) {
    if let Some(r) = send_data(comms::DaemonCommand::SetStandardEffect { name, params }) {
        if let comms::DaemonResponse::SetStandardEffect { result } = r {
            match result {
                true => println!("Effect set OK!"),
                _ => eprintln!("Effect set FAIL!"),
            }
        }
    } else {
        eprintln!("Unknown daemon error!");
    }
}

fn write_effect(opt: Vec<String>) {
    println!("Write effect: Args: {:?}", opt);
    let name = opt[0].clone();
    let mut params: Vec<u8> = vec![];
    for i in 1..opt.len() {
        if let Ok(x) = opt[i].parse::<u8>() {
            params.push(x)
        } else {
            print_help(
                format!(
                    "Option for effect is not valid (Must be 0-255): `{}`",
                    opt[i]
                )
                .as_str(),
            )
        }
    }
    println!("Params: {:?}", params);
    match name.to_ascii_lowercase().as_str() {
        "static" => {
            if params.len() != 3 {
                print_help("Static effect requires 3 args")
            }
            send_effect(name.to_ascii_lowercase(), params)
        }
        "static_gradient" => {
            if params.len() != 6 {
                print_help("Static gradient requires 6 args")
            }
            params.push(0); // Until implimented direction
            send_effect(name.to_ascii_lowercase(), params)
        }
        "wave_gradient" => {
            if params.len() != 6 {
                print_help("Wave gradient requires 6 args")
            }
            params.push(0); // Until implimented direction
            send_effect(name.to_ascii_lowercase(), params)
        }
        "breathing_single" => {
            if params.len() != 4 {
                print_help("Breathing single requires 4 args")
            }
            send_effect(name.to_ascii_lowercase(), params)
        }
        _ => print_help(format!("Unrecognised effect name: `{}`", name).as_str()),
    }
}

fn send_effect(name: String, params: Vec<u8>) {
    if let Some(r) = send_data(comms::DaemonCommand::SetEffect { name, params }) {
        if let comms::DaemonResponse::SetEffect { result } = r {
            match result {
                true => println!("Effect set OK!"),
                _ => eprintln!("Effect set FAIL!"),
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
    if let Some(resp) = send_data(comms::DaemonCommand::GetFanSpeed { ac }) {
        if let comms::DaemonResponse::GetFanSpeed { rpm } = resp {
            let rpm_desc: String = match rpm {
                f if f < 0 => String::from("Unknown"),
                0 => String::from("Auto (0)"),
                _ => format!("{} RPM", rpm),
            };
            println!("Current fan setting: {}", rpm_desc);
        } else {
            eprintln!("Daemon responded with invalid data!");
        }
    }
}

fn read_logo_mode(ac: usize) {
    if let Some(resp) = send_data(comms::DaemonCommand::GetLogoLedState { ac }) {
        if let comms::DaemonResponse::GetLogoLedState { logo_state } = resp {
            let logo_state_desc: &str = match logo_state {
                0 => "Off",
                1 => "On",
                2 => "Breathing",
                _ => "Unknown",
            };
            println!("Current logo setting: {}", logo_state_desc);
        } else {
            eprintln!("Daemon responded with invalid data!");
        }
    }
}

fn read_power_mode(ac: usize) {
    if let Some(resp) = send_data(comms::DaemonCommand::GetPwrLevel { ac }) {
        if let comms::DaemonResponse::GetPwrLevel { pwr } = resp {
            let power_desc: &str = match pwr {
                0 => "Balanced",
                1 => "Gaming",
                2 => "Creator",
                4 => "Custom",
                _ => "Unknown",
            };
            println!("Current power setting: {}", power_desc);
            if pwr == 4 {
                if let Some(resp) = send_data(comms::DaemonCommand::GetCPUBoost { ac }) {
                    if let comms::DaemonResponse::GetCPUBoost { cpu } = resp {
                        let cpu_boost_desc: &str = match cpu {
                            0 => "Low",
                            1 => "Medium",
                            2 => "High",
                            3 => "Boost",
                            _ => "Unknown",
                        };
                        println!("Current CPU setting: {}", cpu_boost_desc);
                    };
                }
                if let Some(resp) = send_data(comms::DaemonCommand::GetGPUBoost { ac }) {
                    if let comms::DaemonResponse::GetGPUBoost { gpu } = resp {
                        let gpu_boost_desc: &str = match gpu {
                            0 => "Low",
                            1 => "Medium",
                            2 => "High",
                            _ => "Unknown",
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

fn write_pwr_mode(ac: usize, opt: Vec<String>) {
    println!("Write power: Args: {:?}", opt);
    if let Ok(mut x) = opt[0].parse::<i8>() {
        if (x >= 0 && x <= 2) || (x == 4) {
            if ac == 0 && x != 0 {
                eprintln!("Only balanced mode can be set on battery power!!!");
                x = 0;
            }
            if x == 4 {
                if opt.len() != 3 {
                    print_help("Invalid number of args supplied");
                } else {
                    if let Ok(cpu_boost) = opt[1].parse::<i8>() {
                        if cpu_boost >= 0 && cpu_boost <= 3 {
                            if let Ok(gpu_boost) = opt[2].parse::<i8>() {
                                if gpu_boost >= 0 && gpu_boost <= 2 {
                                    if let Some(_) = send_data(comms::DaemonCommand::SetPowerMode {
                                        ac,
                                        pwr: x as u8,
                                        cpu: cpu_boost as u8,
                                        gpu: gpu_boost as u8,
                                    }) {
                                        read_power_mode(ac)
                                    } else {
                                        eprintln!("Unknown error!");
                                    }
                                } else {
                                    print_help("CPU boost must be between 0 and 2");
                                }
                            }
                        } else {
                            print_help("CPU boost must be between 0 and 3");
                        }
                    }
                }
            } else {
                if let Some(_) = send_data(comms::DaemonCommand::SetPowerMode {
                    ac,
                    pwr: x as u8,
                    cpu: 0,
                    gpu: 0,
                }) {
                    read_power_mode(ac)
                } else {
                    eprintln!("Unknown error!");
                }
            }
        } else {
            print_help("Power mode must be 0, 1, 2 or 4");
        }
    } else {
        eprintln!("Unknown error!");
    }
}

fn read_brigtness(ac: usize) {
    if let Some(resp) = send_data(comms::DaemonCommand::GetBrightness { ac }) {
        if let comms::DaemonResponse::GetBrightness { result } = resp {
            println!("Current brightness: {}", result);
        } else {
            eprintln!("Daemon responded with invalid data!");
        }
    }
}

fn read_sync() {
    if let Some(resp) = send_data(comms::DaemonCommand::GetSync()) {
        if let comms::DaemonResponse::GetSync { sync } = resp {
            println!("Current sync: {:?}", sync);
        } else {
            eprintln!("Daemon responded with invalid data!");
        }
    }
}

fn write_brightness(ac: usize, val: u8) {
    if let Some(_) = send_data(comms::DaemonCommand::SetBrightness { ac, val }) {
        read_brigtness(ac)
    } else {
        eprintln!("Unknown error!");
    }
}

fn write_idle(ac: usize, val: u32) {
    if let Some(_) = send_data(comms::DaemonCommand::SetIdle { ac, val }) {
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
