#[macro_use]
extern crate log;
use gettextrs::*;
use gtk::gio;

mod application;
mod config;

mod qrcode;
mod widgets;

fn main() {
    pretty_env_logger::init();
    gtk::init().expect("failed to initialize gtk");
    gst::init().expect("Failed to initalize gst");
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR);
    textdomain(config::GETTEXT_PACKAGE);

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/qrscanner.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    application::Application::run();
}
