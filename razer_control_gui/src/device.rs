// mod kbd;
use serde::{Deserialize, Serialize};
use serde_big_array::big_array;
use std::{thread, time, io, fs, env};
use hidapi::HidApi;

#[derive(Serialize, Deserialize, Debug)]
pub struct SupportedDevice {
    pub name: String,
    pub vid: String,
    pub pid: String,
    pub features: Vec<String>,
    pub fan: Vec<u16>,
}

big_array! { 
    BigArray; 
    +80
}

#[derive(Serialize, Deserialize)]
pub struct RazerPacket {
    report: u8,
    status: u8,
    id: u8,
    remaining_packets: u16,
    protocol_type: u8,
    data_size: u8,
    command_class: u8,
    command_id: u8,
    #[serde(with = "BigArray")]
    args: [u8; 80],
    crc: u8,
    reserved: u8,
}

impl RazerPacket {
// Command status
    const RAZER_CMD_NEW:u8 = 0x00;
    // const RAZER_CMD_BUSY:u8 = 0x01;
    const RAZER_CMD_SUCCESSFUL:u8 = 0x02;
    // const RAZER_CMD_FAILURE:u8 = 0x03;
    // const RAZER_CMD_TIMEOUT:u8 =0x04;
    const RAZER_CMD_NOT_SUPPORTED:u8 = 0x05;

    fn new(command_class: u8, command_id: u8, data_size: u8) -> RazerPacket {
        return RazerPacket {
            report: 0x00,
            status: RazerPacket::RAZER_CMD_NEW,
            id: 0x1F,
            remaining_packets: 0x0000,
            protocol_type: 0x00,
            data_size,
            command_class,
            command_id,
            args: [0x00; 80],
            crc: 0x00,
            reserved: 0x00,
        };
    }

    fn calc_crc(&mut self) -> Vec<u8>{
        let mut res: u8 = 0x00;
        let buf: Vec<u8> = bincode::serialize(self).unwrap();
        for i in 2..88 {
            res ^= buf[i];
        }

        self.crc = res;
        return buf;
    }
}

const DEVICE_FILE: &str = "/.local/share/razercontrol/data/devices/laptops.json";
pub struct DeviceManager {
    pub device: Option <RazerLaptop>,
    supported_devices: Vec<SupportedDevice>,
}

impl DeviceManager {
    pub fn new () -> DeviceManager {
        return DeviceManager {
            device: None,
            supported_devices: vec![],
        };
    }

    pub fn read_laptops_file() -> io::Result<DeviceManager > {
        let str: Vec<u8> = fs::read(env::var("HOME").unwrap() + DEVICE_FILE)?;
        let mut res: DeviceManager = DeviceManager::new();
        res.supported_devices = serde_json::from_slice(str.as_slice())?;
        println!("suported devices len: {:?}", res.supported_devices.len());
        Ok(res)
    }

    pub fn get_device(&mut self) -> Option<&mut RazerLaptop> {
        return self.device.as_mut();
    }

    // pub fn set_device(&mut self, device: RazerLaptop) {
        // self.device = Some(device);
    // }

    pub fn discover_devices(&mut self) {
        // Check if socket is OK
        match HidApi::new() {
            Ok(api) => {
                for device in api.device_list() {
                    if device.vendor_id() == 0x1532 {
                        if device.interface_number() != 0 {
                        } else {
                            for sdevice in self.supported_devices.iter_mut() {
                                let pid = u16::from_str_radix(&sdevice.pid, 16).unwrap();
                                if device.product_id() == pid {
                                    match api.open_path(device.path()) {
                                        Ok(dev) => {
                                            self.device = Some(RazerLaptop::new(sdevice.name.clone(), sdevice.features.clone(), sdevice.fan.clone(), dev));
                                            // if let Some(laptop) = self.get_device() {
                                                break;
                                            // }
                                        },
                                        Err(e) => {
                                            eprintln!("Error: {}", e);
                                        }
                                    };
                                }
                            }
                        }

                    }
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            },
        }
    }
}

pub struct RazerLaptop {
    name: String,
    features: Vec<String>,
    fan: Vec<u16>,
    device: hidapi::HidDevice,
    power: u8, // need for fan
    fan_rpm: u8, // need for power
}
//
impl RazerLaptop {
// LED STORAGE Options
    const NOSTORE:u8 = 0x00;
    const VARSTORE:u8 = 0x01;
// LED definitions
    const LOGO_LED:u8 = 0x04;
    const BACKLIGHT_LED:u8 = 0x05;
// effects
    pub const OFF:u8 = 0x00;
    pub const WAVE:u8 = 0x01;
    pub const REACTIVE:u8 = 0x02; // Afterglo
    #[allow(dead_code)]
    pub const BREATHING:u8 = 0x03;
    pub const SPECTRUM:u8 = 0x04;
    pub const CUSTOMFRAME:u8 = 0x05;
    pub const STATIC:u8 = 0x06;
    #[allow(dead_code)]
    pub const STARLIGHT:u8 = 0x19;

    pub fn new(name: String, features: Vec<String>, fan: Vec<u16>, device: hidapi::HidDevice) -> RazerLaptop {
        return RazerLaptop{
            name,
            features,
            fan,
            device,
            power: 0,
            fan_rpm: 0,
        };
    }

    pub fn get_name(&mut self) -> String {
        return self.name.clone();
    }

    pub fn have_feature(&mut self, fch: String) -> bool {
        return self.features.contains(&fch);
    }

    fn clamp_fan(&mut self, rpm: u16) -> u8 {
        if rpm > self.fan[1] {
            return (self.fan[1] / 100) as u8;
        }
        if rpm < self.fan[0] {
            return (self.fan[0] / 100) as u8;
        }

        return (rpm / 100) as u8;
    }

    fn clamp_u8(&mut self, value: u8, min: u8, max: u8) ->u8 {
        if value > max {
            return max;
        }
        if value < min {
            return min;
        }

        return value;
    }

    pub fn set_standard_effect(&mut self, effect_id: u8, params: Vec<u8>) -> bool {
        let mut report: RazerPacket = RazerPacket::new(0x03, 0x0a, 80);
        report.args[0] = effect_id; // effect id
        if params.len() > 0 {
            for idx in 0..params.len() {
                report.args[idx+1] = params[idx];
            }
        }
        if let Some(_) = self.send_report(report) {
            return true;
        }

        return false;
    }

    pub fn set_custom_frame_data(&mut self, row: u8, data: Vec<u8>) {
        // if data.len() == kbd::board::KEYS_PER_ROW {
        if data.len() == 45 {
            let mut report: RazerPacket = RazerPacket::new(0x03, 0x0b, 0x34);
            report.args[0] = 0xff;
            report.args[1] = row;
            report.args[2] = 0x00; // start col
            report.args[3] = 0x0f; // end col
            for idx in 0..data.len() {
                report.args[idx + 7] = data[idx];
            }
            self.send_report(report);
        }
    }

    pub fn set_custom_frame(&mut self) -> bool {
        let mut report: RazerPacket = RazerPacket::new(0x03, 0x0a, 0x02);
        report.args[0] = RazerLaptop::CUSTOMFRAME; // effect id
        report.args[1] = RazerLaptop::NOSTORE;
        if let Some(_) = self.send_report(report) {
            return true;
        }

        return false;
    }

    pub fn get_power_mode(&mut self, zone: u8) -> u8 {
        let mut report: RazerPacket = RazerPacket::new(0x0d, 0x82, 0x04);
        report.args[0] = 0x00;
        report.args[1] = zone;
        report.args[2] = 0x00;
        report.args[3] = 0x00;
        if let Some(response) = self.send_report(report) {
            return response.args[2];
        }
        return 0;
    }

    fn set_power(&mut self, zone: u8) -> bool {
        let mut report: RazerPacket = RazerPacket::new(0x0d, 0x02, 0x04);
        report.args[0] = 0x00;
        report.args[1] = zone;
        report.args[2] = self.power;
        match self.fan_rpm {
            0 => report.args[3] = 0x00,
            _ => report.args[3] = 0x01
        }
        if let Some(_) = self.send_report(report) {
            return  true;
        }

        return false;
    }

    pub fn get_cpu_boost(&mut self) -> u8 {
        let mut report: RazerPacket = RazerPacket::new(0x0d, 0x87, 0x03);
        report.args[0] = 0x00;
        report.args[1] = 0x01;
        report.args[2] = 0x00;
        if let Some(response) = self.send_report(report) {
            return response.args[2];
        }
        return 0;
    }

    pub fn set_cpu_boost(&mut self, mut boost: u8) -> bool {
        let mut report: RazerPacket = RazerPacket::new(0x0d, 0x07, 0x03);
        if boost == 3 && self.have_feature("boost".to_string()) == false {
            boost = 2;
        }
        report.args[0] = 0x00;
        report.args[1] = 0x01;
        report.args[2] = boost;
        if let Some(_)= self.send_report(report) {
            return true;
        }

        return false;
    }

    pub fn get_gpu_boost(&mut self) -> u8 {
        let mut report: RazerPacket = RazerPacket::new(0x0d, 0x87, 0x03);
        report.args[0] = 0x00;
        report.args[1] = 0x02;
        report.args[2] = 0x00;
        if let Some(response) = self.send_report(report){
            return response.args[2];
        }
        return 0;
    }

    pub fn set_gpu_boost(&mut self, boost: u8) -> bool {
        let mut report: RazerPacket = RazerPacket::new(0x0d, 0x07, 0x03);
        report.args[0] = 0x00;
        report.args[1] = 0x02;
        report.args[2] = boost;
        if let Some(_) = self.send_report(report) {
            return true;
        }
        return false;
    }

    pub fn set_power_mode(&mut self, mut mode: u8, cpu_boost: u8, gpu_boost: u8) -> bool {
        if mode <= 2 {
            if mode == 2 && self.have_feature("creator_mode".to_string()) == false {
                mode = 1;
            }
            self.power = mode;
            self.set_power(0x01);
            self.set_power(0x02);
        } else if mode == 4 {
            self.power =  mode;
            self.fan_rpm = 0;
            self.get_power_mode(0x01);
            self.set_power(0x01);
            self.get_cpu_boost();
            self.set_cpu_boost(cpu_boost);
            self.get_gpu_boost();
            self.set_gpu_boost(gpu_boost);
            self.get_power_mode(0x02);
            self.set_power(0x02);
        }

        return true;
    }

    fn set_rpm(&mut self, zone: u8) -> bool {
        let mut report:RazerPacket = RazerPacket::new(0x0d, 0x01, 0x03);
        // Set fan RPM
        report.args[0] = 0x00;
        report.args[1] = zone;
        report.args[2] = self.fan_rpm;
        if let Some(_) = self.send_report(report) {
            return true;
        }

        return false;
    }

    pub fn set_fan_rpm(&mut self, value: u16) -> bool {
        if self.power != 4 {
            match value == 0 {
                true => self.fan_rpm = value as u8,
                false => self.fan_rpm = self.clamp_fan(value),
            }
            self.get_power_mode(0x01);
            self.set_power(0x01);
            if value != 0 {
                self.set_rpm(0x01);
            }
            self.get_power_mode(0x02);
            self.set_power(0x02);
            if value != 0 {
                self.set_rpm(0x02);
            }
        }

        return true;
    }

    pub fn get_fan_rpm(&mut self) -> u16 {
        let res: u16 = self.fan_rpm as u16;
        return res * 100;
    }

    pub fn set_logo_led_state(&mut self, mode: u8) -> bool {
        if mode > 0 {
            let mut report: RazerPacket = RazerPacket::new(0x03, 0x02, 0x03);
            report.args[0] = RazerLaptop::VARSTORE;
            report.args[1] = RazerLaptop::LOGO_LED;
            report.args[2] = self.clamp_u8(mode, 0x00, 0x02);
            self.send_report(report);
        }

        let mut report: RazerPacket = RazerPacket::new(0x03, 0x00, 0x03);
        report.args[0] = RazerLaptop::VARSTORE;
        report.args[1] = RazerLaptop::LOGO_LED;
        report.args[2] = self.clamp_u8(mode, 0x00, 0x01);
        if let Some(_) = self.send_report(report) {
            return true;
        }

        return false;
    }

    pub fn get_logo_led_state(&mut self) -> u8 {
        let mut report: RazerPacket = RazerPacket::new(0x03, 0x82, 0x03);
        report.args[0] = RazerLaptop::VARSTORE;
        report.args[1] = RazerLaptop::LOGO_LED;
        if let Some(response) = self.send_report(report){
            return response.args[2];
        }
        return 0;
    }

    pub fn set_brightness(&mut self, brightness: u8) -> bool {
        let mut report: RazerPacket = RazerPacket::new(0x03, 0x03, 0x03);
        report.args[0] = RazerLaptop::VARSTORE;
        report.args[1] = RazerLaptop::BACKLIGHT_LED;
        report.args[2] = brightness;
        if let Some(_) = self.send_report(report) {
            return true;
        }

        return false;
    }

    pub fn get_brightness(&mut self) -> u8 {
        let mut report: RazerPacket = RazerPacket::new(0x03, 0x83, 0x03);
        report.args[0] = RazerLaptop::VARSTORE;
        report.args[1] = RazerLaptop::BACKLIGHT_LED;
        report.args[2] = 0x00;
        if let Some(response) = self.send_report(report){
            return response.args[2];
        }
        return 0;
    }

    fn send_report(&mut self, mut report: RazerPacket) -> Option<RazerPacket>{
        let mut temp_buf: [u8; 91] = [0x00; 91];
        for _ in 0..3 {
            match self.device.send_feature_report(report.calc_crc().as_slice()) {
                Ok(_) => {
                    thread::sleep(time::Duration::from_micros(1000));
                    match self.device.get_feature_report(&mut temp_buf) {
                        Ok(size) => {
                            if size == 91 {
                                match bincode::deserialize::<RazerPacket>(&temp_buf){
                                    Ok(response) => {
                                        if response.remaining_packets != report.remaining_packets || 
                                            response.command_class != report.command_class ||
                                                response.command_id != report.command_id {
                                                    eprintln!("Response doesn't match request");
                                                }
                                        else if response.status == RazerPacket::RAZER_CMD_SUCCESSFUL {
                                            return Some(response);
                                        }
                                        if response.status == RazerPacket::RAZER_CMD_NOT_SUPPORTED {
                                            eprintln!("Command not supported");
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("Error: {}", e);
                                    }
                                }
                            } else {
                                eprintln!("Invalid report length: {:?}", size);
                            }
                        },
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            };

        }

        thread::sleep(time::Duration::from_micros(8000));
        return None;
    }
}


