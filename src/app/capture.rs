use super::canvas::Bound2;
use image::{
    codecs::gif::GifEncoder as Encoder, DynamicImage::ImageRgba8, GenericImage, GenericImageView,
    ImageBuffer, RgbImage, Rgba, RgbaImage,
};

use crate::platform::windows::capture_impl::CaptureImplWin;

use std::future::Future;
use std::sync::mpsc::RecvError;
use std::sync::Arc;
use std::sync::Mutex;
// TODO:: remove platform-specified mods

use windows::{
    core::*,
    Data::Xml::Dom::*,
    Win32::System::{DataExchange::*, Threading::*},
    Win32::{Foundation::*, Graphics::Gdi::*, System::Threading::*, UI::WindowsAndMessaging::*},
};

pub struct CaptureDevice {
    pub runtime: tokio::runtime::Runtime,
    pub stop_signal: Arc<Mutex<bool>>,
}

impl CaptureDevice {
    pub fn new() -> std::io::Result<Self> {
        Ok(CaptureDevice {
            runtime: tokio::runtime::Runtime::new()?,
            stop_signal: Arc::new(Mutex::new(false)),
        })
    }

    pub fn capture_image(&self, rect: Bound2) -> RgbaImage {
        CaptureImplWin::new(HWND(0), rect).capture_image()
    }

    pub fn stop_capture(&self) {
        let mut stop = self.stop_signal.lock().unwrap();
        *stop = true;
        println!("stop_capture");
    }

    pub fn capture_gif_async(
        &self,
        rect: Bound2,
        fps: u32,
        duration: f64,
        finished_cb: Box<dyn FnOnce(Vec<u8>) + Send + 'static>,
    ) {
        println!("????");
        if fps <= 0 || fps > 60 {
            println!("Wrong fps: {}, it should be in range (0, 60]", fps);
        }
        use std::time::Instant;
        use tokio::time::Interval;
        let interval = 1.0 / fps as f64;
        let dur = std::time::Duration::from_secs_f64(duration);
        let width = rect.get_width();
        let height = rect.get_height();

        let (tx, rx) = std::sync::mpsc::channel();
        *self.stop_signal.lock().unwrap() = false;
        let stop_signal_clone = self.stop_signal.clone();
        self.runtime.spawn(async move {
            let interval = std::time::Duration::from_secs_f64(interval);
            let mut cap_impl = CaptureImplWin::new(HWND(0), rect);
            let total_frames = fps * duration as u32;
            let mut frames = 0;
            let mut elapse = Instant::now();
            let end = elapse + dur;

            while frames < total_frames && elapse < end {
                let img = async { cap_impl.capture_image() }.await;
                println!("capture {}", frames);
                {
                    let mut stop = stop_signal_clone.lock().unwrap();
                    if *stop == true {
                        return;
                    }
                }
                tx.send(img);
                elapse += interval;
                frames += 1;
                std::thread::sleep(interval);
            }
            //finished_cb(frames_data);
        });

        self.runtime.spawn(async move {
            let mut encode_buf = vec![0u8; 0];
            let mut encoder = Encoder::new(&mut encode_buf);
            let mut f = 0;
            loop {
                f += 1;
                match rx.recv() {
                    Ok(img) => {
                        let mut frame = image::Frame::new(img);
                        encoder.encode_frame(frame).unwrap();
                        println!("encoding {}", f);
                    }
                    Err(RecvError) => {
                        println!("encode finished");
                        break;
                    }
                }
            }
            drop(encoder);
            finished_cb(encode_buf);
        });
    }
}
