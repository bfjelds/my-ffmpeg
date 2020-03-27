extern crate gstreamer as gst;
use gst::prelude::*;

extern crate gstreamer_app as gst_app;
extern crate gstreamer_video as gst_video;
extern crate glib;


fn create_pipeline(
    rtsp_url: &String,
    frame_sender: std::sync::mpsc::Sender<Vec<u8>>
) -> Result<gst::Pipeline, failure::Error> {

    gst::init()?;

    // let pipeline = gst::Pipeline::new(None);
    // let convert = gst::ElementFactory::make("videoconvert", None).unwrap();
    // let sink = gst::ElementFactory::make("autovideosink", None).unwrap();

    // pipeline.add_many(&[&convert, &sink])?;
    // gst::Element::link_many(&[&convert, &sink])?;
    
    // let appsink = sink.dynamic_cast::<gst_app::AppSink>().expect("Sink element is expeected to be an appsink!");


    let pipeline = gst::ElementFactory::make("playbin", None).unwrap();
    let pipeline = pipeline.dynamic_cast::<gstreamer::Pipeline>().unwrap();

    let appsink = gst::ElementFactory::make("appsink", Some("sink")).unwrap();
    let appsink: gst_app::AppSink = appsink.dynamic_cast().unwrap();

    let caps = gstreamer::Caps::builder("video/x-raw")
        .field("format", &gst_video::VideoFormat::Rgb.to_string())
        .field("pixel-aspect-ratio", &gst::Fraction::from((1, 1)))
        .build();
    appsink.set_caps(Some(&caps));

    let closure_sender = frame_sender.clone();
    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::new()
            .new_sample(move |appsink| {
                let sample = appsink.pull_sample().unwrap();
                let buffer = sample.get_buffer().unwrap();
                let map = buffer.map_readable().unwrap();

                let caps = sample.get_caps().unwrap();
                println!("Sample caps: {:?}", caps);
                let s = caps.get_structure(0).unwrap();
                println!("Sample caps structure: {:?}", s);
                let width: i32 = caps.iter().next().unwrap().get("width").unwrap().unwrap();
                let height: i32 = caps.iter().next().unwrap().get("height").unwrap().unwrap();
                println!("Sample caps [w,h]: {},{}", width, height);

                let data = Vec::from(map.as_slice());
                closure_sender.send(data).unwrap();
                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    pipeline.set_property("uri", &Some(rtsp_url)).unwrap();
    pipeline.set_property("video-sink", &appsink).unwrap();
    pipeline.set_state(gstreamer::State::Paused).unwrap();

    Ok(pipeline)
}

fn main() {
    println!("my-ffmpeg start");

    // let mut frame_tx = bus::Bus::new(10);
    // let mut frame_rx = frame_tx.add_rx();
    let (frame_tx, frame_rx) = std::sync::mpsc::channel();

    let pipeline = create_pipeline(&std::env::var("RTSP_URL").unwrap(), frame_tx).unwrap();
    // Start pipeline
    pipeline.set_state(gstreamer::State::Playing).unwrap();

    std::thread::spawn(move ||{

        loop {
            let frame: Vec<u8> = frame_rx.recv().unwrap();
            println!("[{:?}] Received frame of size: {}", std::time::SystemTime::now(), frame.len());
        }

    });
    println!("my-ffmpeg end");
}
