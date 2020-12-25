use glib::{Receiver, Sender};
use gst::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene};

mod camera_sink {
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct Frame(
        pub gst_video::VideoFrame<gst_video::video_frame::Readable>,
        pub RefCell<Option<gst::Buffer>>,
    );

    impl Frame {
        pub fn new(buffer: &gst::Buffer, info: &gst_video::VideoInfo) -> Self {
            let video_frame =
                gst_video::VideoFrame::from_buffer_readable(buffer.clone(), &info).unwrap();
            Self(video_frame, RefCell::new(Some(buffer.clone())))
        }

        pub fn width(&self) -> u32 {
            self.0.width()
        }

        pub fn height(&self) -> u32 {
            self.0.height()
        }

        pub fn texture(&self) -> gdk::MemoryTexture {
            let format = match self.0.format() {
                gst_video::VideoFormat::Bgra => gdk::MemoryFormat::B8g8r8a8,
                gst_video::VideoFormat::Argb => gdk::MemoryFormat::A8r8g8b8,
                gst_video::VideoFormat::Rgba => gdk::MemoryFormat::R8g8b8a8,
                gst_video::VideoFormat::Abgr => gdk::MemoryFormat::A8b8g8r8,
                gst_video::VideoFormat::Rgb => gdk::MemoryFormat::R8g8b8,
                gst_video::VideoFormat::Bgr => gdk::MemoryFormat::B8g8r8,
                _ => gdk::MemoryFormat::A8r8g8b8,
            };
            let width = self.0.width() as i32;
            let height = self.0.height() as i32;
            let rowstride = self.0.plane_stride()[0] as usize;

            let buffer = {
                let bytes = self.1.borrow_mut().take().unwrap();
                bytes.into_mapped_buffer_readable().unwrap()
            };

            gdk::MemoryTexture::new(
                width,
                height,
                format,
                &glib::Bytes::from_owned(buffer),
                rowstride,
            )
        }
    }

    pub enum Action {
        FrameChanged(Frame),
        CodeDetected(String),
    }

    use super::*;

    mod imp {
        use super::*;
        use glib::subclass;
        use gst::subclass::prelude::*;
        use gst_base::subclass::prelude::*;
        use gst_video::subclass::prelude::*;
        use std::sync::Mutex;

        pub struct CameraSink {
            pub info: Mutex<Option<gst_video::VideoInfo>>,
            pub sender: Mutex<Option<Sender<Action>>>,
        }

        impl ObjectSubclass for CameraSink {
            const NAME: &'static str = "CameraSink";
            type Type = super::CameraSink;
            type ParentType = gst_video::VideoSink;
            type Instance = gst::subclass::ElementInstanceStruct<Self>;
            type Class = subclass::simple::ClassStruct<Self>;

            glib::object_subclass!();

            fn new() -> Self {
                Self {
                    info: Mutex::new(None),
                    sender: Mutex::new(None),
                }
            }

            fn class_init(klass: &mut Self::Class) {
                klass.set_metadata(
                    "GTK Camera Sink",
                    "Sink/Camera/Video",
                    "A GTK Camera sink",
                    "Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>",
                );

                let caps = gst_video::video_make_raw_caps(&[
                    gst_video::VideoFormat::Bgra,
                    gst_video::VideoFormat::Argb,
                    gst_video::VideoFormat::Rgba,
                    gst_video::VideoFormat::Abgr,
                    gst_video::VideoFormat::Rgb,
                    gst_video::VideoFormat::Bgr,
                ])
                .any_features()
                .build();
                let src_pad_template = gst::PadTemplate::new(
                    "src",
                    gst::PadDirection::Src,
                    gst::PadPresence::Always,
                    &caps,
                )
                .unwrap();
                klass.add_pad_template(src_pad_template);

                let sink_pad_template = gst::PadTemplate::new(
                    "sink",
                    gst::PadDirection::Sink,
                    gst::PadPresence::Always,
                    &caps,
                )
                .unwrap();
                klass.add_pad_template(sink_pad_template);
            }
        }

        impl ObjectImpl for CameraSink {}
        impl ElementImpl for CameraSink {}
        impl BaseSinkImpl for CameraSink {
            fn set_caps(
                &self,
                _element: &Self::Type,
                caps: &gst::Caps,
            ) -> Result<(), gst::LoggableError> {
                let video_info = gst_video::VideoInfo::from_caps(caps).unwrap();
                let mut info = self.info.lock().unwrap();
                info.replace(video_info);

                Ok(())
            }
        }
        impl VideoSinkImpl for CameraSink {
            fn show_frame(
                &self,
                _element: &Self::Type,
                buffer: &gst::Buffer,
            ) -> Result<gst::FlowSuccess, gst::FlowError> {
                if let Some(info) = &*self.info.lock().unwrap() {
                    let frame = Frame::new(buffer, info);
                    let sender = self.sender.lock().unwrap();
                    sender
                        .as_ref()
                        .unwrap()
                        .send(Action::FrameChanged(frame))
                        .unwrap();
                }
                Ok(gst::FlowSuccess::Ok)
            }
        }
    }

    glib::wrapper! {
        pub struct CameraSink(ObjectSubclass<imp::CameraSink>) @extends gst_video::VideoSink, gst_base::BaseSink, gst::Element, gst::Object;
    }
    unsafe impl Send for CameraSink {}
    unsafe impl Sync for CameraSink {}

    impl CameraSink {
        pub fn new(sender: Sender<Action>) -> Self {
            let sink = glib::Object::new(&[]).expect("Failed to create a CameraSink");
            let priv_ = imp::CameraSink::from_instance(&sink);
            priv_.sender.lock().unwrap().replace(sender);
            sink
        }
    }
}

mod imp {
    use super::*;
    use glib::subclass;
    use gst_player::subclass::prelude::*;
    use std::cell::RefCell;

    pub struct CameraPaintable {
        pub sink: camera_sink::CameraSink,
        pub pipeline: gst::Pipeline,
        pub sender: Sender<camera_sink::Action>,
        pub image: RefCell<Option<gdk::Paintable>>,
        pub size: RefCell<Option<(u32, u32)>>,
        pub receiver: RefCell<Option<Receiver<camera_sink::Action>>>,
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
            type_.add_interface::<gst_player::PlayerVideoRenderer>();
        }

        fn new() -> Self {
            let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            let receiver = RefCell::new(Some(r));

            Self {
                pipeline: gst::Pipeline::new(Some("camera")),
                sink: camera_sink::CameraSink::new(sender.clone()),
                image: RefCell::new(None),
                sender,
                receiver,
                size: RefCell::new(None),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.add_signal(
                "code-detected",
                glib::SignalFlags::RUN_FIRST,
                &[String::static_type()],
                glib::Type::Unit,
            );
        }
    }

    impl ObjectImpl for CameraPaintable {
        fn constructed(&self, obj: &Self::Type) {
            obj.init_widgets();
            self.parent_constructed(obj);
        }
        fn dispose(&self, _obj: &Self::Type) {
            self.pipeline.set_state(gst::State::Null).unwrap();
        }
    }

    impl PaintableImpl for CameraPaintable {
        fn get_intrinsic_height(&self, _paintable: &Self::Type) -> i32 {
            if let Some((_, height)) = *self.size.borrow() {
                height as i32
            } else {
                0
            }
        }
        fn get_intrinsic_width(&self, _paintable: &Self::Type) -> i32 {
            if let Some((width, _)) = *self.size.borrow() {
                width as i32
            } else {
                0
            }
        }

        fn snapshot(
            &self,
            _paintable: &Self::Type,
            snapshot: &gdk::Snapshot,
            width: f64,
            height: f64,
        ) {
            let snapshot_gtk = snapshot.downcast_ref::<gtk::Snapshot>().unwrap();
            snapshot_gtk.append_color(
                &gdk::RGBA::black(),
                &graphene::Rect::new(0f32, 0f32, width as f32, height as f32),
            );
            if let Some(ref image) = *self.image.borrow() {
                image.snapshot(snapshot, width, height);
            }
        }
    }

    impl PlayerVideoRendererImpl for CameraPaintable {
        fn create_video_sink(
            &self,
            video_renderer: &Self::Type,
            _player: &gst_player::Player,
        ) -> gst::Element {
            let src = gst::ElementFactory::make("autovideosrc", Some("source")).unwrap();
            let convert1 = gst::ElementFactory::make("videoconvert", None).unwrap();
            let zbar = gst::ElementFactory::make("zbar", None).unwrap();
            let convert2 = gst::ElementFactory::make("videoconvert", None).unwrap();
            self.pipeline
                .add_many(&[
                    &src,
                    &convert1,
                    &zbar,
                    &convert2,
                    &self.sink.clone().upcast(),
                ])
                .unwrap();

            src.link(&convert1).unwrap();
            convert1.link(&zbar).unwrap();
            zbar.link(&convert2).unwrap();
            convert2.link(&self.sink).unwrap();
            self.pipeline.set_state(gst::State::Playing).unwrap();

            let bus = self.pipeline.get_bus().unwrap();
            bus.add_watch_local(glib::clone!(@strong self.sender as sender => move |_, msg| {
                use gst::MessageView;
                match msg.view() {
                    MessageView::Element(e) => {
                        if let Some(s) = e.get_structure() {
                            let qrcode = s.get::<String>("symbol").unwrap().unwrap();
                            gtk_macros::send!(sender, camera_sink::Action::CodeDetected(qrcode));
                        }
                    }
                    MessageView::Error(err) => {
                        println!(
                            "Error from {:?}: {} ({:?})",
                            err.get_src().map(|s| s.get_path_string()),
                            err.get_error(),
                            err.get_debug()
                        );
                    }
                    _ => (),
                };

                glib::Continue(true)
            }))
            .expect("Failed to add bus watch");

            self.sink.clone().upcast()
        }
    }
}

glib::wrapper! {
    pub struct CameraPaintable(ObjectSubclass<imp::CameraPaintable>) @implements gdk::Paintable, gst_player::PlayerVideoRenderer;
}

impl CameraPaintable {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a CameraPaintable")
    }

    pub fn init_widgets(&self) {
        let self_ = imp::CameraPaintable::from_instance(self);

        let receiver = self_.receiver.borrow_mut().take().unwrap();
        receiver.attach(
            None,
            glib::clone!(@weak self as paintable => move |action| paintable.do_action(action)),
        );
    }

    fn do_action(&self, action: camera_sink::Action) -> glib::Continue {
        let self_ = imp::CameraPaintable::from_instance(self);
        match action {
            camera_sink::Action::FrameChanged(frame) => {
                let (width, height) = (frame.width(), frame.height());
                self_.size.replace(Some((width, height)));
                let texture = frame.texture();
                self_.image.replace(Some(texture.upcast()));
                self.invalidate_contents();
            }
            camera_sink::Action::CodeDetected(code) => {
                self.emit("code-detected", &[&code]).unwrap();
            }
        }

        glib::Continue(true)
    }
}
