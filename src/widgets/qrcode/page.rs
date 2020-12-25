use super::QRCodePaintable;
use crate::qrcode::QRCode;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;
    use glib::subclass;

    #[derive(CompositeTemplate)]
    pub struct QRCodePage {
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,
        pub paintable: QRCodePaintable,
    }

    impl ObjectSubclass for QRCodePage {
        const NAME: &'static str = "QRCodePage";
        type Type = super::QRCodePage;
        type ParentType = gtk::Box;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn class_init(klass: &mut Self::Class) {
            klass.set_template_from_resource("/com/belmoussaoui/qrscanner/ui/qrcode_page.ui");
            Self::bind_template_children(klass);
        }

        fn new() -> Self {
            Self {
                picture: TemplateChild::default(),
                paintable: QRCodePaintable::new(),
            }
        }
    }

    impl ObjectImpl for QRCodePage {
        fn constructed(&self, obj: &Self::Type) {
            obj.init_template();
            obj.init_widgets();
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for QRCodePage {}
    impl BoxImpl for QRCodePage {}
}

glib::wrapper! {
    pub struct QRCodePage(ObjectSubclass<imp::QRCodePage>) @extends gtk::Widget, gtk::Box;
}

impl QRCodePage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a QRCodePage widget")
    }

    fn init_widgets(&self) {
        let self_ = imp::QRCodePage::from_instance(self);
        self_.picture.get().set_paintable(Some(&self_.paintable));
    }

    pub fn set_qrcode(&self, code: QRCode) {
        let self_ = imp::QRCodePage::from_instance(self);
        self_.paintable.set_qrcode(code);
    }
}
