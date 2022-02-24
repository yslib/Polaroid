use glutin::{
    event_loop::EventLoopProxy,
    window::{Window, WindowId},
    ContextWrapper, PossiblyCurrent, WindowedContext,
};
use std::collections::HashMap;

use super::{
    action::Action,
    canvas::{Bound2, RegionSelector, Renderable},
    event::{Event, KeyInputData, MouseData, UserEvent, WindowEventHandler},
    graphics::Graphics,
};

// use log::{debug, info};

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Target {
    Application,
    Action,
    Window(AppWindow),
}

pub type WindowHashMap = HashMap<WindowId, Box<dyn WindowEventHandler>>;
pub type WindowIDDHashMap = HashMap<AppWindow, WindowId>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[allow(unused)]
pub enum AppWindow {
    AllWindow,
    RegionSelectorCanvasWindow,
    ConfigWindow,
}

pub struct CanvasWindow {
    pub windowed_context: Option<ContextWrapper<PossiblyCurrent, Window>>,
    pub graphics: Box<dyn Graphics>,
    pub event_proxy: EventLoopProxy<UserEvent>,
    pub region_selector: RegionSelector,
    pub invoke_type: Action,
    window_id: Target,
}

impl CanvasWindow {
    pub fn new(
        windowed_context: WindowedContext<PossiblyCurrent>,
        graphics: Box<dyn Graphics>,
        event_proxy: EventLoopProxy<UserEvent>,
        window_id: Target,
    ) -> Self {
        CanvasWindow {
            windowed_context: Some(windowed_context),
            graphics,
            event_proxy,
            window_id,
            invoke_type: Action::ImageCapture,
            region_selector: RegionSelector::new(),
        }
    }
}

impl CanvasWindow {
    pub fn swap_buffers(&self) {
        self.windowed_context
            .as_ref()
            .unwrap()
            .swap_buffers()
            .expect("swap buffer");
    }

    #[allow(unused)]
    pub fn make_current(&mut self) {
        if !self.windowed_context.as_ref().unwrap().is_current() {
            self.windowed_context = Some(unsafe {
                self.windowed_context
                    .take()
                    .unwrap()
                    .make_current()
                    .expect("context swap")
            });
        }
    }

    pub fn request_redraw(&self) {
        self.windowed_context
            .as_ref()
            .unwrap()
            .window()
            .request_redraw();
    }

    #[allow(unused)]
    pub fn get_selector_region(&self) -> Bound2 {
        self.region_selector.bound
    }
}

impl WindowEventHandler for CanvasWindow {
    fn on_mouse_press_event(&mut self, data: &MouseData) {
        self.region_selector.set_visible(true);
        self.region_selector.set_first(data.position.into());
    }

    #[allow(unused)]
    fn on_mouse_release_event(&mut self, data: &MouseData) {
        let bound = self.region_selector.bound;
        if bound.empty() == false {
            let action = match self.invoke_type {
                Action::ImageCapture => Action::DoImageCapture(bound),
                Action::GifCapture => Action::DoGifCapture(bound),
                _ => {
                    panic!("unexpected action");
                }
            };
            self.send_user_event(Target::Action, Event::DoAction(action));
            self.request_redraw();
        }
    }

    fn send_user_event(&self, receiver: Target, event: Event) {
        let user_event = UserEvent::new(self.window_id, receiver, event);
        self.event_proxy.send_event(user_event).unwrap();
    }

    fn on_mouse_move_event(&mut self, data: &MouseData) {
        self.region_selector.set_second(data.position.into());
        self.request_redraw();
    }

    #[allow(unused)]
    fn on_keyboard_event(&mut self, data: &KeyInputData) {
        // unimplemented!();
    }

    fn on_focus_event(&mut self, focus: bool) {
        if focus {
            //
        } else {
            self.region_selector.set_visible(false);
        }
    }

    fn handle_redraw_event(&mut self) {
        self.graphics.clear((0.0, 0.0, 0.0, 0.5));
        self.region_selector.update(&*self.graphics); // ???
        self.swap_buffers();
    }

    ///
    /// This can receive user event whenever the window is visible or unvisible
    fn on_user_event(&mut self, data: &UserEvent) {
        match data.event {
            crate::app::event::Event::InvokeRegionSelector(action) => {
                self.set_visible(true);
                self.invoke_type = action;
            }
            _ => {}
        }
    }

    fn set_visible(&mut self, visible: bool) {
        self.windowed_context.as_ref().map(|f| {
            //info!("set main window visible: {}", visible);
            f.window().set_visible(visible);

            use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32Handle};

            use windows::{
                core::*, Data::Xml::Dom::*, Win32::Foundation::*, Win32::System::Threading::*,
                Win32::UI::WindowsAndMessaging::*,
            };
            if cfg!(windows) {
                let handle = f.window().raw_window_handle();
                unsafe {
                    match handle {
                        RawWindowHandle::Win32(Win32Handle {
                            hwnd, hinstance: _, ..
                        }) => {
                            let hwnd = HWND(hwnd as isize);
                            let mut exstyle =
                                WINDOW_EX_STYLE(GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32);
                            exstyle = exstyle | WS_EX_TOOLWINDOW;
                            SetWindowLongW(hwnd, GWL_EXSTYLE, exstyle.0 as i32);
                        }
                        _ => (),
                    }
                }
            }
        });
    }
}
