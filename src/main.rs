extern crate gstreamer as gst;
use gst::prelude::*;

extern crate gstreamer_app as gst_app;
extern crate glib;

fn create_pipeline(
) -> Result<gst::Pipeline, failure::Error> {

    gst::init()?;

    let pipeline = gst::Pipeline::new(None);
    let convert = gst::ElementFactory::make("videoconvert", None).unwrap();
    let sink = gst::ElementFactory::make("autovideosink", None).unwrap();

    pipeline.add_many(&[&convert, &sink])?;
    gst::Element::link_many(&[&convert, &sink])?;
    
    let appsink = sink.dynamic_cast::<gst_app::AppSink>().expect("Sink element is expeected to be an appsink!");

    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::new()
            .new_sample(|appsink| {
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

                let _data = Vec::from(map.as_slice());
                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    Ok(pipeline)
}

fn main() {
    println!("Hello, world!");
}
