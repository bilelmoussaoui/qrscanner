use gettextrs::*;
use gtk::prelude::*;
use gtk::gio;

mod config;
mod window;
use crate::window::Window;

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain("qrscanner", config::LOCALEDIR);
    textdomain("qrscanner");

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/qrscanner.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    let app = gtk::Application::new(Some("com.belmoussaoui.qrscanner"), Default::default()).unwrap();
    app.connect_activate(move |app| {
        let window = Window::new();

        window.widget.set_application(Some(app));
        app.add_window(&window.widget);
        window.widget.present();
    });

    let ret = app.run(&std::env::args().collect::<Vec<_>>());
    std::process::exit(ret);
}
