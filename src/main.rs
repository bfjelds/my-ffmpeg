extern crate image;
extern crate gstreamer as gst;
use gst::prelude::*;

extern crate gstreamer_app as gst_app;
extern crate gstreamer_video as gst_video;
extern crate glib;

use image::{ImageFormat, GenericImageView};

use std::io::prelude::*;


fn create_pipeline(
    rtsp_url: &String,
    frame_sender: std::sync::mpsc::Sender<()>
) -> Result<gst::Pipeline, failure::Error> {

    gst::init()?;

    let file_number = std::sync::Arc::new(std::sync::Mutex::new(0));

    let source = gst::ElementFactory::make("playbin", None)?;
    let pipeline = source.dynamic_cast::<gst::Pipeline>().unwrap();
    let pnmenc = gst::ElementFactory::make("pngenc", None)?;
    let sink = gst::ElementFactory::make("appsink", None)?;
    let appsink = sink.clone()
        .downcast::<gst_app::AppSink>()
        .unwrap();
    let elems = &[&pnmenc, &sink];
    let bin = gst::Bin::new(None);
    bin.add_many(elems)?;
    gst::Element::link_many(elems)?;

    // make input for bin point to first element
    let sink = elems[0].get_static_pad("sink").unwrap();
    let ghost = gst::GhostPad::new(Some("sink"), &sink)?;
    ghost.set_active(true)?;
    bin.add_pad(&ghost)?;

    pipeline.set_property("uri", &Some(rtsp_url))?;
    pipeline.set_property("video-sink", &bin.upcast::<gst::Element>())?;

    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::new()
            .new_sample({
                move |sink| {
                    let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos).unwrap();
                    {
                        let mut local_file_number = file_number.lock().unwrap();
                        if *local_file_number < 10 {
                            let current_file_number = *local_file_number;
                            *local_file_number += 1;
    
                            let buffer = sample.get_buffer().unwrap();
                            let map = buffer.map_readable().unwrap();
                            let png = image::load_from_memory_with_format(&map, ImageFormat::PNG).unwrap();

                            let filename = format!("/tmp/foo-{}.png", current_file_number);
                            png.save(&filename).unwrap();
                            println!("wrote to {}", &filename);
                            
                            let filename2 = format!("/tmp/foo-{}-buffered.png", current_file_number);
                            let mut buffer = Vec::new();
                            png.write_to(&mut buffer, ImageFormat::PNG);
                            let mut buffered_file = std::fs::File::create(&filename2).unwrap();
                            buffered_file.write_all(&buffer);
                            println!("wrote to {}", &filename2);
    
                        } else {
                            frame_sender.send(());
                        }
                    }
                    Ok(gst::FlowSuccess::Ok)
                }
            })
            .build()
    );


/*
    let pipeline = gst::ElementFactory::make("playbin", None).unwrap();
    let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();
    let sink = gst::ElementFactory::make("appsink", Some("sink")).unwrap();
    let appsink = sink.clone().downcast::<gst_app::AppSink>().unwrap();
    //let appsink: gst_app::AppSink = sink.dynamic_cast().unwrap();

    let pngenc = gst::ElementFactory::make("jpegenc", None)?;
    let elems = &[&pngenc, &sink];
    let bin = gst::Bin::new(None);
    bin.add_many(elems)?;
    gst::Element::link_many(elems)?;

    // make input for bin point to first element
    let sink = elems[0].get_static_pad("sink").unwrap();
    let ghost = gst::GhostPad::new(Some("sink"), &sink)?;
    ghost.set_active(true)?;
    bin.add_pad(&ghost)?;

    pipeline.set_property("video-sink", &bin.upcast::<gst::Element>())?;

    // Would include to get RGB string for caps, but
    // it seems to conflict with our code coverage
    // tool.  As workaround, hardcoding string.
    // let rgb_format = gst_video::VideoFormat::Rgb.to_string();
    // let rgb_format = "RGB".to_string();
    // let caps = gst::Caps::builder("video/x-raw")
    //     .field("format", &rgb_format)
    //     .field("height", &480)
    //     .field("width", &640)
    //     .build();
    // println!("app.sink.set_caps({})", caps);
    // appsink.set_caps(Some(&caps));

    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::new()
            .new_sample(move |appsink| {
                let sample = appsink.pull_sample().unwrap();
                let buffer = sample.get_buffer().unwrap();
                let map = buffer.map_readable().unwrap();
                
                {
                    let mut local_file_number = file_number.lock().unwrap();
                    if *local_file_number < 100 {
                        let current_file_number = *local_file_number;
                        *local_file_number += 1;

                        let filename = format!("/tmp/foo-{}.pnm", current_file_number);
                        let pnm = image::load_from_memory_with_format(&map, ImageFormat::PNM).unwrap();
                        pnm.save(&filename);

                        println!("wrote to {}", &filename);
                    } else {
                        frame_sender.send(());
                    }
                }
                            

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    println!("pipeline.set_property(uri): {}", rtsp_url);
    pipeline.set_property("uri", &Some(rtsp_url)).unwrap();
    pipeline.set_property("video-sink", &appsink).unwrap();
*/
    match pipeline.set_state(gst::State::Paused) {
        Ok(_) => {}
        Err(e) => {
            println!("Failed to set state of pipeline: {}", e);
            Err(e).unwrap()
        }
    }

    Ok(pipeline)
}

fn main() {
    println!("my-ffmpeg start");

    let (frame_tx, frame_rx) = std::sync::mpsc::channel();

    let pipeline = create_pipeline(&std::env::var("RTSP_URL").unwrap(), frame_tx).unwrap();
    // Start pipeline
    pipeline.set_state(gstreamer::State::Playing).unwrap();

    let _ = frame_rx.recv();
    println!("my-ffmpeg end");
}
