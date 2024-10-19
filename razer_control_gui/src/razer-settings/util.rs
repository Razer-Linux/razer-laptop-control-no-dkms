/// This function attempts to determine whether we are running on AC power.
/// 
/// At the moment it might not support every device or power supply. It is used
/// to determine which tab to show when loading the GUI.
pub fn check_if_running_on_ac_power() -> Option<bool> {
    let result = std::fs::read("/sys/class/power_supply/AC0/online");
    
    result.map(|contents| {
        println!("contents: {contents:?}");
        
        contents == "1\n".as_bytes()
    }).ok()
}
