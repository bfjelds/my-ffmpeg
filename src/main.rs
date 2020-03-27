extern crate gstreamer as gst;
use gst::prelude::*;

extern crate gstreamer_app as gst_app;
extern crate glib;

fn create_pipeline() -> Result<gst::Pipeline, failure::Error> {
    gst::init()?;

    let pipeline = gst::Pipeline::new(None);
    let convert = gst::ElementFactory::make("videoconvert", None).ok_or("Missing element: videoconvert")?;
    let sink = gst::ElementFactory::make("autovideosink", None).ok_or("Missing element: autovideosink")?;

    pipeline.add_many(&[&src, &sink, &sink])?;
    gst::Element::link_many(&[&src, &sink, &sink])?;
    
    let appsink = sink.dynamic_cast::<gst_app::AppSink>().expect("Sink element is expeected to be an appsink!");

    appsink.set_callbacks(
        gst_app::AppSinkCallacks::new()
            .new_sample(|appsink| {
                let sample = appsink.pull_sample().ok_or("Error in pull_sample")?;
                let buffer = sample.get_buffer().ok_or("Failed to get buffer from appsink")?;
                let map = buffer.map_readable().ok_or("Failed to map buffer readable")?;

                let caps = sample.get_caps()?;
                let s = caps.get_structure(0)?;
                let width = s.get("width")?;
                let height = s.get("height")?;

                let data = Vec::from(map.as_slice());

            })
    )
}

fn main() {
    println!("Hello, world!");
}
