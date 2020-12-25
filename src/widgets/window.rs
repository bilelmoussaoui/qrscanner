use crate::{
    application::Application,
    config,
    qrcode::QRCode,
    widgets::{CameraPaintable, QRCodeCreatePage, QRCodePage},
};
use gtk::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, CompositeTemplate};

mod imp {
    use super::*;
    use glib::subclass;
    use libhandy::subclass::application_window::ApplicationWindowImpl as HdyApplicationWindowImpl;

    #[derive(Debug, CompositeTemplate)]
    pub struct Window {
        #[template_child]
        pub deck: TemplateChild<libhandy::Leaflet>,
        #[template_child]
        pub create_page: TemplateChild<QRCodeCreatePage>,
        #[template_child]
        pub code_page: TemplateChild<QRCodePage>,
        #[template_child]
        pub camera_picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub dark_mode_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub view_switcher_title: TemplateChild<libhandy::ViewSwitcherTitle>,
        #[template_child]
        pub switcher_bar: TemplateChild<libhandy::ViewSwitcherBar>,
        pub camera_paintable: CameraPaintable,
        pub player: gst_player::Player,
    }

    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = libhandy::ApplicationWindow;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn new() -> Self {
            let ctx = gst_player::PlayerGMainContextSignalDispatcher::new(
                glib::MainContext::get_thread_default().as_ref(),
            );
            let camera_paintable = CameraPaintable::new();
            let player = gst_player::Player::new(
                Some(&camera_paintable.clone().upcast()),
                Some(&ctx.upcast()),
            );

            Self {
                player,
                camera_paintable,
                deck: TemplateChild::default(),
                camera_picture: TemplateChild::default(),
                create_page: TemplateChild::default(),
                code_page: TemplateChild::default(),
                dark_mode_button: TemplateChild::default(),
                switcher_bar: TemplateChild::default(),
                view_switcher_title: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            QRCodeCreatePage::static_type();
            QRCodePage::static_type();
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

    pub fn show_code_detected(&self, code: QRCode) {
        let self_ = imp::Window::from_instance(self);
        self_.code_page.get().set_qrcode(code);
        self_.deck.get().set_visible_child_name("code");
    }

    fn init(&self) {
        let self_ = imp::Window::from_instance(self);

        self_.player.play();

        self_
            .camera_paintable
            .connect_local(
                "code-detected",
                false,
                glib::clone!(@weak self as win => move |args| {
                    let code = args.get(1).unwrap().get::<String>().unwrap().unwrap();
                    win.show_code_detected(QRCode::from_string(&code));

                    None
                }),
            )
            .unwrap();

        self_
            .camera_picture
            .get()
            .set_paintable(Some(&self_.camera_paintable));

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
    }

    fn setup_actions(&self, app: &Application) {}

    fn setup_signals(&self, app: &Application) {}
}
