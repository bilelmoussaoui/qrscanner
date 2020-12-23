use gtk::prelude::*;

pub struct Window {
    pub widget: gtk::ApplicationWindow,
}

impl Window {
    pub fn new() -> Self {
        let builder = gtk::Builder::from_resource("/com/belmoussaoui/qrscanner/window.ui");
        let widget: gtk::ApplicationWindow = builder
            .get_object("window")
            .expect("Failed to find the window object");

        Self { widget }
    }
}
