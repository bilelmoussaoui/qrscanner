use super::QRCodePaintable;
use crate::qrcode::QRCode;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(CompositeTemplate)]
    pub struct QRCodeCreatePage {
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub textview: TemplateChild<gtk::TextView>,
        pub paintable: QRCodePaintable,
    }

    impl ObjectSubclass for QRCodeCreatePage {
        const NAME: &'static str = "QRCodeCreatePage";
        type Type = super::QRCodeCreatePage;
        type ParentType = gtk::Box;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn class_init(klass: &mut Self::Class) {
            klass.set_template_from_resource("/com/belmoussaoui/qrscanner/ui/qrcode_create.ui");
            Self::bind_template_children(klass);
        }

        fn new() -> Self {
            Self {
                picture: TemplateChild::default(),
                textview: TemplateChild::default(),
                paintable: QRCodePaintable::new(),
            }
        }
    }

    impl ObjectImpl for QRCodeCreatePage {
        fn constructed(&self, obj: &Self::Type) {
            obj.init_template();
            obj.init_widgets();
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for QRCodeCreatePage {}
    impl BoxImpl for QRCodeCreatePage {}
}

glib::wrapper! {
    pub struct QRCodeCreatePage(ObjectSubclass<imp::QRCodeCreatePage>) @extends gtk::Widget, gtk::Box;
}

impl QRCodeCreatePage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a QRCodeCreatePage widget")
    }

    fn init_widgets(&self) {
        let self_ = imp::QRCodeCreatePage::from_instance(self);
        self_.picture.get().set_paintable(Some(&self_.paintable));

        self_.textview.get().get_buffer().connect_changed(
            glib::clone!(@weak self as page => move |buffer| {
                let (start_iter, end_iter) = buffer.get_bounds();
                let content = buffer.get_text(&start_iter, &end_iter, true);
                page.regen_qrcode(&content);
            }),
        );
    }

    pub fn regen_qrcode(&self, for_content: &str) {
        let self_ = imp::QRCodeCreatePage::from_instance(self);

        let qrcode = QRCode::from_string(for_content);
        self_.paintable.set_qrcode(qrcode);
    }
}
