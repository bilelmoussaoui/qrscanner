use crate::qrcode::QRCode;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene};

mod imp {
    use super::*;
    use glib::subclass;
    use std::cell::RefCell;
    pub struct QRCodePaintable {
        pub qrcode: RefCell<Option<QRCode>>,
    }

    impl ObjectSubclass for QRCodePaintable {
        const NAME: &'static str = "QRCodePaintable";
        type Type = super::QRCodePaintable;
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn type_init(type_: &mut subclass::InitializingType<Self>) {
            type_.add_interface::<gdk::Paintable>();
        }

        fn new() -> Self {
            Self {
                qrcode: RefCell::new(None),
            }
        }
    }

    impl ObjectImpl for QRCodePaintable {}

    impl PaintableImpl for QRCodePaintable {
        fn get_flags(&self, _paintable: &Self::Type) -> gdk::PaintableFlags {
            // Fixed size
            gdk::PaintableFlags::SIZE
        }

        fn get_intrinsic_width(&self, _paintable: &Self::Type) -> i32 {
            200
        }

        fn get_intrinsic_height(&self, _paintable: &Self::Type) -> i32 {
            200
        }

        fn snapshot(
            &self,
            _paintable: &Self::Type,
            snapshot: &gdk::Snapshot,
            width: f64,
            height: f64,
        ) {
            let snapshot = snapshot.downcast_ref::<gtk::Snapshot>().unwrap();
            let square_size = 8f32; // Each square is 16px

            if let Some(ref qrcode) = *self.qrcode.borrow() {
                let start_pos_x = ((width as f32) - (qrcode.width as f32) * square_size) / 2f32;
                let start_pos_y = ((height as f32) - (qrcode.height as f32) * square_size) / 2f32;

                qrcode.items.iter().enumerate().for_each(|(y, line)| {
                    line.iter().enumerate().for_each(|(x, is_dark)| {
                        let color = if *is_dark {
                            gdk::RGBA::black()
                        } else {
                            gdk::RGBA {
                                red: 0.0,
                                blue: 0.0,
                                green: 0.0,
                                alpha: 0.0,
                            }
                        };
                        let position = graphene::Rect::new(
                            start_pos_x + (x as f32) * square_size,
                            start_pos_y + (y as f32) * square_size,
                            square_size,
                            square_size,
                        );

                        snapshot.append_color(&color, &position);
                    });
                });
            }
        }
    }
}

glib::wrapper! {
    pub struct QRCodePaintable(ObjectSubclass<imp::QRCodePaintable>) @implements gdk::Paintable;
}

impl QRCodePaintable {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a QRCodePaintable")
    }

    pub fn set_qrcode(&self, qrcode: QRCode) {
        let self_ = imp::QRCodePaintable::from_instance(self);
        self_.qrcode.replace(Some(qrcode));
        self.invalidate_contents();
    }
}
