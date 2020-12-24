use crate::{application::Application, config, qrcode::QRCode, widgets::QRCodePaintable};
use gtk::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, CompositeTemplate};

mod imp {
    use super::*;
    use glib::subclass;
    use libhandy::subclass::application_window::ApplicationWindowImpl as HdyApplicationWindowImpl;

    #[derive(Debug, CompositeTemplate)]
    pub struct Window {
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub dark_mode_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub view_switcher_title: TemplateChild<libhandy::ViewSwitcherTitle>,
        #[template_child]
        pub switcher_bar: TemplateChild<libhandy::ViewSwitcherBar>,
        #[template_child]
        pub textview: TemplateChild<gtk::TextView>,
        pub qrcode_paintable: QRCodePaintable,
    }

    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = libhandy::ApplicationWindow;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                picture: TemplateChild::default(),
                textview: TemplateChild::default(),
                dark_mode_button: TemplateChild::default(),
                switcher_bar: TemplateChild::default(),
                view_switcher_title: TemplateChild::default(),
                qrcode_paintable: QRCodePaintable::new(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_template_from_resource("/com/belmoussaoui/qrscanner/ui/window.ui");
            Self::bind_template_children(klass);
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            obj.init_template();

            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl HdyApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, libhandy::ApplicationWindow, gio::ActionMap, gio::ActionGroup;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        let window = glib::Object::new::<Window>(&[("application", app)]).unwrap();
        app.add_window(&window);

        if config::PROFILE == "Devel" {
            window.get_style_context().add_class("devel");
        }
        window.init();
        window.setup_actions(app);
        window.setup_signals(app);
        window
    }

    pub fn regen_qrcode(&self, for_content: &str) {
        let self_ = imp::Window::from_instance(self);

        let qrcode = QRCode::from_string(for_content);
        self_.qrcode_paintable.set_qrcode(qrcode);
    }

    fn init(&self) {
        let self_ = imp::Window::from_instance(self);

        self_
            .picture
            .get()
            .set_paintable(Some(&self_.qrcode_paintable));

        let gtk_settings = gtk::Settings::get_default().unwrap();
        let dark_mode_button = self_.dark_mode_button.get();
        gtk_settings.connect_property_gtk_application_prefer_dark_theme_notify(move |settings| {
            if !settings.get_property_gtk_application_prefer_dark_theme() {
                dark_mode_button.set_icon_name("dark-mode-symbolic");
            } else {
                dark_mode_button.set_icon_name("light-mode-symbolic");
            }
        });

        let switcher_bar = self_.switcher_bar.get();
        self_
            .view_switcher_title
            .get()
            .connect_property_title_visible_notify(move |view_switcher| {
                switcher_bar.set_reveal(view_switcher.get_title_visible());
            });

        self_.textview.get().get_buffer().connect_changed(
            glib::clone!(@weak self as win => move |buffer| {
                let (start_iter, end_iter) = buffer.get_bounds();
                let content = buffer.get_text(&start_iter, &end_iter, true);
                win.regen_qrcode(&content);
            }),
        );
    }

    fn setup_actions(&self, app: &Application) {}

    fn setup_signals(&self, app: &Application) {}
}
