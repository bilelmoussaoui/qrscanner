use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene};

mod imp {
    use super::*;
    use glib::subclass;
    pub struct CameraPaintable {
    }

    impl ObjectSubclass for CameraPaintable {
        const NAME: &'static str = "CameraPaintable";
        type Type = super::CameraPaintable;
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib::object_subclass!();

        fn type_init(type_: &mut subclass::InitializingType<Self>) {
            type_.add_interface::<gdk::Paintable>();
        }

        fn new() -> Self {
            Self {
            }
        }
    }

    impl ObjectImpl for CameraPaintable {}

    impl PaintableImpl for CameraPaintable {
        fn snapshot(
            &self,
            _paintable: &Self::Type,
            snapshot: &gdk::Snapshot,
            width: f64,
            height: f64,
        ) {
            let snapshot = snapshot.downcast_ref::<gtk::Snapshot>().unwrap();
            snapshot.append_color(
                &gdk::RGBA::black(),
                &graphene::Rect::new(0f32, 0f32, width as f32, height as f32),
            );
        }
    }
}

glib::wrapper! {
    pub struct CameraPaintable(ObjectSubclass<imp::CameraPaintable>) @implements gdk::Paintable;
}

impl CameraPaintable {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a CameraPaintable")
    }

}
