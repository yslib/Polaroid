use std::borrow::Cow;
use std::io::Write;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::app::window::AppWindow;

use image::codecs::gif::GifEncoder as Encoder;
use image::RgbaImage;
use tokio::time::Interval;

use super::window::Target;
use super::{
    canvas::Bound2,
    capture::CaptureDevice,
    event::{Event, UserEvent, WindowEventHandler},
    window::{WindowHashMap, WindowIDDHashMap},
};
use chrono::Duration;
use glutin::{event::ModifiersState, event_loop::EventLoopProxy};
//use log::debug;
pub trait Execute<A: ActionContext> {
    fn execute(&self, ctx: &mut A);
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum Action {
    ImageCapture,
    DoImageCapture(Bound2),
    DoGifCapture(Bound2),
    GifCapture,
    StopGifCaptureAndSave,
    StopGifCaptureAndDrop,
    Suspend,
}

impl<A> Execute<A> for Action
where
    A: ActionContext,
{
    fn execute(&self, ctx: &mut A) {
        match self {
            Self::ImageCapture => ctx.invoke_image_capture(),
            Self::GifCapture => ctx.invoke_gif_capture(),
            Self::Suspend => ctx.suspend(),
            Self::DoGifCapture(rect) => {
                ctx.do_gif_capture(*rect, 15, 30f64);
            }
            Self::DoImageCapture(rect) => {
                ctx.do_image_capture(*rect);
                ctx.suspend();
            }
            Self::StopGifCaptureAndSave => {
                ctx.suspend();
            }
            Self::StopGifCaptureAndDrop => {
                ctx.suspend();
            }
        }
    }
}

pub trait ActionContext {
    fn invoke_image_capture(&mut self);
    fn invoke_gif_capture(&mut self);
    fn do_image_capture(&mut self, rect: Bound2);
    fn do_gif_capture(&mut self, rect: Bound2, fps: u32, duration: f64);
    fn suspend(&mut self);
    fn stop_gif_capture_and_save(&mut self);
    fn stop_gif_capture_and_drop(&mut self);
}

pub struct AppContext<'a> {
    pub event_proxy: &'a mut EventLoopProxy<UserEvent>,
    pub capture_device: &'a mut CaptureDevice,
    pub window_hash: &'a mut WindowHashMap,
    pub window_id_hash: &'a mut WindowIDDHashMap,
}

impl<'a> AppContext<'a> {
    pub fn find_window(&mut self, app_window: AppWindow) -> Option<&mut dyn WindowEventHandler> {
        if let Some(win) = self.window_id_hash.get(&app_window) {
            if let Some(main_window) = self.window_hash.get_mut(win) {
                return Some(main_window.deref_mut());
            }
        }
        None
    }

    pub fn create_timestamp_str(&self) -> String {
        use chrono::Utc;
        chrono::offset::Local::now()
            .format("%F-%H-%M-%S")
            .to_string()
    }

    pub fn get_save_path(&self) -> PathBuf {
        use directories::UserDirs;
        let user_dir = UserDirs::new();
        user_dir
            .desktop_dir()
            .map_or(PathBuf::new(), |f| f.to_path_buf())
    }

    pub fn check_file_exists<T: AsRef<std::ffi::OsStr>>(&self, path: T) -> bool {
        Path::new(&path).is_file()
    }
}

impl<'a> ActionContext for AppContext<'a> {
    ///
    /// Invokes the static image capture canvas for the selection
    fn invoke_image_capture(&mut self) {
        let event = Event::InvokeRegionSelector(Action::ImageCapture);
        let user_event = UserEvent::new(
            Target::Action,
            Target::Window(AppWindow::RegionSelectorCanvasWindow),
            event,
        );
        self.event_proxy.send_event(user_event);
    }

    ///
    /// Invokes the GIF image capture canvas for the selection
    fn invoke_gif_capture(&mut self) {
        // debug!("invoke_gif_capture");
        //
        let event = Event::InvokeRegionSelector(Action::GifCapture);
        let user_event = UserEvent::new(
            Target::Action,
            Target::Window(AppWindow::RegionSelectorCanvasWindow),
            event,
        );
        self.event_proxy.send_event(user_event);
    }

    ///
    /// Back the capture canvas when finished
    fn suspend(&mut self) {
        // debug!("suspend");
        let target_win = self
            .find_window(AppWindow::RegionSelectorCanvasWindow)
            .unwrap();
        target_win.set_visible(false);
        self.capture_device.stop_capture();
    }

    ///
    /// capture static image
    fn do_image_capture(&mut self, rect: Bound2) {
        let image = self.capture_device.capture_image(rect);
        let ts = self.create_timestamp_str();
        let filename = format!("CAP_{}.png", ts);
        let mut save_path = self.get_save_path();
        save_path.push(filename);
        image.save(save_path).unwrap();
    }

    ///
    /// Capture gif image
    fn do_gif_capture(&mut self, bound: Bound2, fps: u32, duration: f64) {
        let rect = bound.rect();
        let ts = self.create_timestamp_str();
        let filename = format!("CAP_{}.gif", ts);
        let mut save_path = self.get_save_path();
        save_path.push(filename);

        self.capture_device.capture_gif_async(
            bound,
            fps,
            duration,
            Box::new(|encode_data| {
                let mut encode_data = encode_data;
                let mut image = std::fs::File::create(save_path).unwrap();
                image.write(&mut encode_data);
            }),
        );
    }

    fn stop_gif_capture_and_save(&mut self) {}

    fn stop_gif_capture_and_drop(&mut self) {}
}

pub struct KeyBinding<T: Eq> {
    pub action: Action,
    pub mods: ModifiersState,
    pub key: T,
}

impl<T: Eq> KeyBinding<T> {
    #[inline(always)]
    pub fn is_triggered(&self, mods: ModifiersState, key: T) -> bool {
        self.mods == mods && self.key == key
    }
}
