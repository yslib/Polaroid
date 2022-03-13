use glutin::{
    event_loop::EventLoopProxy,
    window::{Window, WindowId},
    ContextWrapper, PossiblyCurrent, WindowedContext,
};
use std::collections::HashMap;

use super::{
    action::Action,
    canvas::{Bound2, RegionSelector, Renderable, Canvas},
    event::{Event, KeyInputData, MouseData, UserEvent},
    graphics::{Graphics, self}
};

// use log::{debug, info};

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Target {
    Application,
    Action,
    Window(AppWindow),
}

pub type WindowHashMap = HashMap<WindowId, Box<dyn EventListener + 'static>>;
pub type WindowIDDHashMap = HashMap<AppWindow, WindowId>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[allow(unused)]
pub enum AppWindow {
    AllWindow,
    RegionSelectorCanvasWindow,
    GifSelectorCanvasWindow,
    ConfigWindow,
}

pub trait EventListener {
    fn on_mouse_press_event(&mut self, data: &MouseData);

    fn on_mouse_release_event(&mut self, data: &MouseData);

    fn on_mouse_move_event(&mut self, data: &MouseData);

    fn on_keyboard_event(&mut self, data: &KeyInputData);

    fn handle_redraw_event(&mut self);

    fn on_redraw_event(&mut self, graphics: &dyn Graphics);

    fn on_user_event(&mut self, data: &UserEvent);

    fn on_focus_event(&mut self, focus: bool);

    fn set_visible(&mut self, visible:bool);
}

pub trait WinInst: EventListener{
    fn set_win_visible(&mut self, visible:bool);

    fn window_id(&self)->WindowId;
}



pub struct WindowInstance {
    pub windowed_context: Option<ContextWrapper<PossiblyCurrent, Window>>,
    pub graphics: Box<dyn Graphics>,
    pub event_proxy: EventLoopProxy<UserEvent>,
    pub region_selector: RegionSelector,
    pub invoke_type: Action,
}

impl WindowInstance {
    pub fn new(
        windowed_context: WindowedContext<PossiblyCurrent>,
        graphics: Box<dyn Graphics>,
        event_proxy: EventLoopProxy<UserEvent>,
    ) -> Self {
        WindowInstance {
            windowed_context: Some(windowed_context),
            graphics,
            event_proxy,
            invoke_type: Action::ImageCapture,
            region_selector: RegionSelector::new(),
        }
    }

}

impl WindowInstance {
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

    pub fn send_user_event(&self,sender:Target, receiver: Target, event: Event) {
        let user_event = UserEvent::new(sender, receiver, event);
        self.event_proxy.send_event(user_event).unwrap();
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

    pub fn set_visible(&mut self, visible: bool) {
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
