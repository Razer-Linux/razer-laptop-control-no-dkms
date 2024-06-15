use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use gtk::{
    Box, Label, Scale, Stack, StackSwitcher, Switch, ToolItem, Toolbar
};
use gtk::{glib, glib::clone};
        
// sudo apt install libgdk-pixbuf2.0-dev libcairo-dev libatk1.0-dev
// sudo apt install libpango1.0-dev

#[path = "../comms.rs"]
mod comms;
mod widgets;

use widgets::*;

fn send_data(opt: comms::DaemonCommand) -> Option<comms::DaemonResponse> {
    match comms::bind2() {
        Ok(socket) => comms::send_to_daemon(opt, socket),
        Err(error) => {
            println!("Error opening socket: {error}");
            None
        }
    }
}

fn get_bho() -> Option<(bool, u8)> {
    let response = send_data(comms::DaemonCommand::GetBatteryHealthOptimizer())?;

    use comms::DaemonResponse::*;
    match response {
        GetBatteryHealthOptimizer { is_on, threshold } => {
            Some((is_on, threshold))
        }
        response => {
            // This should not happen
            println!("Instead of GetBatteryHealthOptimizer got {response:?}");
            None
        }
    }
}

fn set_bho(is_on: bool, threshold: u8) -> Option<bool> {
    let response = send_data(comms::DaemonCommand::SetBatteryHealthOptimizer {
        is_on, threshold
    })?;

    use comms::DaemonResponse::*;
    match response {
        SetBatteryHealthOptimizer { result } => {
            Some(result)
        }
        response => {
            // This should not happen
            println!("Instead of SetBatteryHealthOptimizer got {response:?}");
            None
        }
    }
}

fn get_logo() -> Option<u8> {
    let response = send_data(comms::DaemonCommand::GetLogoLedState{ ac: 1 })?;

    use comms::DaemonResponse::*;
    match response {
        GetLogoLedState { logo_state } => {
            Some(logo_state)
        }
        response => {
            // This should not happen
            println!("Instead of GetLogoLedState got {response:?}");
            None
        }
    }
}

fn set_logo(value: u8) -> Option<bool> {
    let response = send_data(comms::DaemonCommand::SetLogoLedState{ ac: 1, logo_state: value })?;

    use comms::DaemonResponse::*;
    match response {
        SetLogoLedState { result } => {
            Some(result)
        }
        response => {
            // This should not happen
            println!("Instead of SetLogoLedState got {response:?}");
            None
        }
    }
}

fn get_power() -> Option<u8> {
    let response = send_data(comms::DaemonCommand::GetPwrLevel{ ac: 1 })?;

    use comms::DaemonResponse::*;
    match response {
        GetPwrLevel { pwr } => {
            Some(pwr)
        }
        response => {
            // This should not happen
            println!("Instead of GetPwrLevel got {response:?}");
            None
        }
    }
}

fn get_fan_speed(ac: usize) -> Option<i32> {
    let response = send_data(comms::DaemonCommand::GetFanSpeed{ ac: 1 })?;

    use comms::DaemonResponse::*;
    match response {
        GetFanSpeed { rpm } => {
            Some(rpm)
        }
        response => {
            // This should not happen
            println!("Instead of GetFanSpeed got {response:?}");
            None
        }
    }
}

fn set_fan_speed(value: i32) -> Option<bool> {
    let response = send_data(comms::DaemonCommand::SetFanSpeed{ ac: 1, rpm: value })?;

    use comms::DaemonResponse::*;
    match response {
        SetFanSpeed { result } => {
            Some(result)
        }
        response => {
            // This should not happen
            println!("Instead of SetFanSpeed got {response:?}");
            None
        }
    }
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let app = Application::builder()
        .application_id("com.example.hello")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(640)
            .default_height(480)
            .title("Razer Settings")
            .build();
        
        let bho = get_bho().unwrap();
        let logo = get_logo().unwrap();
        let fan_speed = get_fan_speed(1).unwrap();

        let settings_page = SettingsPage::new();

        // Logo section
        let settings_section = settings_page.add_section(Some("Logo"));
            let label = Label::new(Some("Turn on logo"));
            let switch = Switch::new();
            switch.set_state(logo == 1);
            switch.connect_changed_active(|switch| {
                let on = switch.is_active();
                set_logo(if on { 1 } else { 0 });
                let logo = get_logo().unwrap_or(0);
                switch.set_active(logo == 1);
            });
        let row = SettingsRow::new(&label, &switch);
        settings_section.add_row(&row.master_container);

        // Battery Health Optimizer section
        let settings_section = settings_page.add_section(Some("Battery Health Optimizer"));
            let label = Label::new(Some("Enable Battery Health Optimizer"));
            let switch = Switch::new();
            switch.set_state(bho.0);
        let row = SettingsRow::new(&label, &switch);
        settings_section.add_row(&row.master_container);
            let label = Label::new(Some("Theshold"));
            let scale = Scale::with_range(gtk::Orientation::Horizontal, 65f64, 80f64, 1f64);
            scale.set_value(bho.1 as f64);
            scale.set_width_request(100);
            scale.connect_change_value(clone!(@weak switch => @default-return gtk::glib::Propagation::Stop, move |scale, stype, value| {
                let is_on = switch.is_active();
                let threshold = scale.value() as u8;

                set_bho(is_on, threshold).unwrap();

                let (is_on, threshold) = get_bho().unwrap();
                
                scale.set_value(threshold as f64);
                scale.set_visible(is_on);
                switch.set_sensitive(is_on);

                return gtk::glib::Propagation::Stop;
            }));
            scale.set_sensitive(bho.0);
            switch.connect_changed_active(clone!(@weak scale => move |switch| {
                let is_on = switch.is_active();
                let threshold = scale.value() as u8;
                
                set_bho(is_on, threshold); // Ignoramos errores ya que leemos
                                           // el resultado de vuelta

                let (is_on, threshold) = get_bho().unwrap();
                
                scale.set_value(threshold as f64);
                scale.set_visible(is_on);
                switch.set_sensitive(is_on);
            }));
        let row = SettingsRow::new(&label, &scale);
        settings_section.add_row(&row.master_container);

        // Fan Speed Section
        let settings_section = settings_page.add_section(Some("Fan Speed"));
            let label = Label::new(Some("Auto"));
            let switch = Switch::new();
            let auto = fan_speed == 0;
            switch.set_state(auto);
        let row = SettingsRow::new(&label, &switch);
        settings_section.add_row(&row.master_container);
            let label = Label::new(Some("Fan Speed"));
            let scale = Scale::with_range(gtk::Orientation::Horizontal, 3500f64, 5000f64, 1f64);
            scale.set_value(fan_speed as f64);
            scale.set_sensitive(fan_speed != 0);
            scale.set_width_request(100);
            scale.connect_change_value(clone!(@weak switch => @default-return gtk::glib::Propagation::Stop, move |scale, stype, value| {
                set_fan_speed(value as i32).unwrap();
                let fan_speed = get_fan_speed(1).unwrap();
                let auto = fan_speed == 0;
                scale.set_value(fan_speed as f64);
                scale.set_sensitive(!auto);
                switch.set_state(auto);
                return gtk::glib::Propagation::Stop;
            }));
            switch.connect_changed_active(clone!(@weak scale => move |switch| {
                set_fan_speed(if switch.is_active() { 0 } else { 3500 }).unwrap();
                let fan_speed = get_fan_speed(1).unwrap();
                let auto = fan_speed == 0;
                scale.set_value(fan_speed as f64);
                scale.set_sensitive(!auto);
                switch.set_state(auto);
            }));
        let row = SettingsRow::new(&label, &scale);
        settings_section.add_row(&row.master_container);

        let stack = Stack::new();
        let battery_stack = Label::new(Some("Not ready"));
        battery_stack.set_valign(gtk::Align::Center);
        battery_stack.set_halign(gtk::Align::Center);
        stack.add_titled(&settings_page.master_container, "AC", "AC");
        stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
        stack.add_titled(&battery_stack, "Battery", "Battery");
        stack.connect_screen_changed(|_, _| {
            println!("Page changed");
        });

        let stack_switcher = StackSwitcher::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();

        stack_switcher.set_stack(Some(&stack));
        stack_switcher.set_halign(gtk::Align::Center);
        stack_switcher.connect_screen_changed(|_, _| {
            println!("Page changed");
        });
        
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let toolbar = Toolbar::new();
        toolbar.style_context().add_class("primary-toolbar");
        vbox.pack_start(&toolbar, false, false, 0);
        vbox.pack_start(&stack, true, true, 0);
        // header_bar.set_title(Some("Razer Settings"));
        // header_bar.set_child(Some(&stack_switcher));
        // window.set_titlebar(Some(&header_bar));
        let tool_item = ToolItem::new();
        gtk::prelude::ToolItemExt::set_expand(&tool_item, true);
        tool_item.style_context().add_class("raised");
        let stask_switcher_holder = Box::new(gtk::Orientation::Horizontal, 0);
        stask_switcher_holder.set_border_width(1);
        stask_switcher_holder.pack_start(&stack_switcher, true, true, 0);
        tool_item.add(&stask_switcher_holder);
        toolbar.insert(&tool_item, 0);

        window.set_child(Some(&vbox));

        window.show_all();
    });

    app.run();
}
