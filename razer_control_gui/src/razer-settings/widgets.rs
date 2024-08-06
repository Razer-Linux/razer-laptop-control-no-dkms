use std::cell::Cell;

use gtk::prelude::*;
use gtk::{
    Box, Frame, Label, ListBox, ListBoxRow, Separator, Widget, Grid
};

pub struct SettingsPage {
    // TODO: Can I make this a widget? This is self originally
    pub master_container: Box
}

impl SettingsPage {
    
    pub fn new() -> SettingsPage {
        let master_container = Box::new(gtk::Orientation::Vertical, 15);
        master_container.set_margin_start(80);
        master_container.set_margin_end(80);
        master_container.set_margin_top(15);
        master_container.set_margin_bottom(15);

        SettingsPage {
            master_container
        }
    }

    pub fn add_section(&self, title: Option<&str>) -> SettingsSection {
        let section = SettingsSection::new(title);
        self.master_container.pack_start(&section.master_container, false, false, 0);
        section
    }

}

pub struct SettingsRow {
    // TODO: Can I make this a widget? This is self originally
    pub master_container: ListBoxRow
}

impl SettingsRow {
    
    pub fn new(
        label: &impl IsA<Widget>,
        main_widget: &impl IsA<Widget>,
        // alternative_widget: Option<&impl IsA<Widget>>
    ) -> SettingsRow {
        let master_container = ListBoxRow::new();

        // TODO: Faltan cosas, hay un stack que IMO no tiene sentido por ahora

        let hbox = Box::new(gtk::Orientation::Horizontal, 0);
        hbox.set_border_width(5);
        hbox.set_margin_start(20);
        hbox.set_margin_end(20);
        // master_container.add(&hbox);

        let grid = Grid::new();
        grid.set_column_spacing(15);
        // hbox.pack_start(&grid, true, true, 0);

        let description_box = Box::new(gtk::Orientation::Vertical, 0);
        description_box.set_hexpand(true);
        description_box.set_halign(gtk::Align::Start);
        description_box.set_valign(gtk::Align::Center);
        // self.label.props.xalign = 0.0
        description_box.add(label);

        grid.attach(&description_box, 0, 0, 1, 1);
        grid.attach_next_to(main_widget /*stack*/, Some(&description_box), gtk::PositionType::Right, 1, 1);
        hbox.add(&grid); // TODO: No es así como lo hacen
        
        master_container.add(&hbox);

        SettingsRow {
            master_container
        }
    }

    pub fn add_section(&self, title: Option<&str>) -> SettingsSection {
        let section = SettingsSection::new(title);
        // self.master_container.pack_start(&section.master_container, false, false, 0); TODO: It should be this
        self.master_container.add(&section.master_container);
        section
    }

}

pub struct SettingsSection {
    // TODO: Can I make this a widget? This is self originally
    pub master_container: Box,
    container: Box,
    frame: Frame,
    need_separator: Cell<bool>
}

impl SettingsSection {

    pub fn new(title: Option<&str>) -> SettingsSection {
        let master_container = Box::new(gtk::Orientation::Vertical, 10);

        if let Some(title) = title {
            let header_box = Box::new(gtk::Orientation::Vertical, 0);
            header_box.set_spacing(5);
            master_container.add(&header_box);

            let label = Label::new(None);
            label.set_markup(&format!("<b>{}</b>", title));
            // Aligmnent 0, 0.5
            label.set_halign(gtk::Align::Start);
            header_box.add(&label);
        }

        let frame = Frame::new(None);
        frame.set_shadow_type(gtk::ShadowType::In);
        frame.style_context().add_class("view");
        // bho_frame.set_hexpand(true);
        // Algo de size group

        let container = Box::new(gtk::Orientation::Vertical, 0);
        frame.add(&container);

        SettingsSection {
            master_container,
            container,
            frame,
            need_separator: Cell::new(false)
        }
    }

    pub fn add_row(&self, widget: &impl IsA<Widget>) {
        let vbox = Box::new(gtk::Orientation::Vertical, 0);

        if self.need_separator.get() {
            let separator = Separator::new(gtk::Orientation::Horizontal);
            vbox.add(&separator);
        }

        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::None);
        list_box.add(widget);
        vbox.add(&list_box);
        self.container.add(&vbox);

        if self.frame.parent().is_none() {
            self.master_container.add(&self.frame);
        }

        self.need_separator.set(true);
    }
}
