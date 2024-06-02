use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use gtk::{
    Box, Label, Scale, Stack, StackSwitcher, Switch, ToolItem, Toolbar
};
        
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

fn read_bho() -> Option<(bool, u8)> {
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
        
        let bho = read_bho().unwrap();
        let logo = get_logo().unwrap();

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
