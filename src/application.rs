use crate::{config, widgets::Window};
use gettextrs::gettext;
use glib::clone;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};
use gtk_macros::{action, stateful_action};
use std::env;

mod imp {
    use super::*;
    use glib::{subclass, WeakRef};

    use std::cell::RefCell;

    pub struct Application {
        pub window: RefCell<Option<WeakRef<Window>>>,
    }

    static PROPERTIES: [subclass::Property; 2] = [
        subclass::Property("locked", |name| {
            glib::ParamSpec::boolean(name, "locked", "locked", false, glib::ParamFlags::READWRITE)
        }),
        subclass::Property("can-be-locked", |name| {
            glib::ParamSpec::boolean(
                name,
                "can_be_locked",
                "can be locked",
                false,
                glib::ParamFlags::READWRITE,
            )
        }),
    ];

    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type ParentType = gtk::Application;
        type Type = super::Application;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn class_init(klass: &mut Self::Class) {
            //klass.install_properties(&PROPERTIES);
        }

        fn new() -> Self {
            Self {
                window: RefCell::new(None),
            }
        }
    }

    impl ObjectImpl for Application {}
    impl GtkApplicationImpl for Application {}
    impl ApplicationImpl for Application {
        fn startup(&self, app: &Self::Type) {
            self.parent_startup(app);

            libhandy::functions::init();

            let app = app.downcast_ref::<super::Application>().unwrap();
            if let Some(ref display) = gtk::gdk::Display::get_default() {
                let p = gtk::CssProvider::new();
                gtk::CssProvider::load_from_resource(&p, "/com/belmoussaoui/qrscanner/style.css");
                gtk::StyleContext::add_provider_for_display(display, &p, 500);
                let theme = gtk::IconTheme::get_for_display(display).unwrap();
                theme.add_resource_path("/com/belmoussaoui/qrscanner/icons/");
                app.set_resource_base_path(Some("/com/belmoussaoui/qrscanner/"));
            }

            action!(app, "quit", clone!(@weak app => move |_, _| app.quit()));

            // About
            action!(
                app,
                "about",
                clone!(@weak app => move |_, _| {
                    let window = app.get_active_window().unwrap();
                    let about_dialog = gtk::AboutDialogBuilder::new()
                        .program_name(&gettext("Authenticator"))
                        .modal(true)
                        .version(config::VERSION)
                        .comments(&gettext("Generate Two-Factor Codes"))
                        .website("https://gitlab.gnome.org/World/Authenticator")
                        .authors(vec!["Bilal Elmoussaoui".to_string()])
                        .artists(vec!["Alexandros Felekidis".to_string(), "Tobias Bernard".to_string()])
                        .translator_credits(&gettext("translator-credits"))
                        .logo_icon_name(config::APP_ID)
                        .license_type(gtk::License::Gpl30)
                        .transient_for(&window)
                        .build();

                    about_dialog.show();
                })
            );
        }

        fn activate(&self, app: &Self::Type) {
            if let Some(ref win) = *self.window.borrow() {
                let window = win.upgrade().unwrap();
                window.present();
                return;
            }

            let app = app.downcast_ref::<super::Application>().unwrap();
            let window = app.create_window();
            window.present();
            self.window.replace(Some(window.downgrade()));

            let settings = gio::Settings::new(config::APP_ID);
            let gtk_settings = gtk::Settings::get_default().unwrap();
            settings
                .bind(
                    "dark-mode",
                    &gtk_settings,
                    "gtk-application-prefer-dark-theme",
                )
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();

            let is_dark_mode = settings.get_boolean("dark-mode");
            stateful_action!(app, "dark-mode", is_dark_mode, move |action, _| {
                let state = action.get_state().unwrap();
                let action_state: bool = state.get().unwrap();
                let is_dark_mode = !action_state;
                action.set_state(&is_dark_mode.to_variant());
                if let Err(err) = settings.set_boolean("dark-mode", is_dark_mode) {
                    error!("Failed to switch dark mode: {} ", err);
                }
            });

            app.set_accels_for_action("app.quit", &["<primary>q"]);
            app.set_accels_for_action("app.dark-mode", &["<primary>t"]);
            app.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);
            app.set_accels_for_action("add.scan-qr", &["<primary>s"]);
        }
    }
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn run() {
        info!("QRScanner ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        let app = glib::Object::new::<Application>(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &gio::ApplicationFlags::empty()),
        ])
        .unwrap();

        let args: Vec<String> = env::args().collect();
        ApplicationExtManual::run(&app, &args);
    }

    fn create_window(&self) -> Window {
        Window::new(&self.clone())
    }
}
