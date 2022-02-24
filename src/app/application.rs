use std::cell::Cell;
use std::ops::DerefMut;
use std::path::PathBuf;

use crate::support;

use glutin::event_loop::EventLoopProxy;
use glutin::{
    dpi::PhysicalPosition,
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, ModifiersState, MouseButton,
        VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
    ContextBuilder, ContextWrapper, NotCurrent, PossiblyCurrent,
};

use super::{
    action::{Action, AppContext, Execute, KeyBinding},
    capture::CaptureDevice,
    event::{KeyInputData, MouseData, UserEvent, WindowEventHandler},
    graphics::Graphics,
    graphics_impl::opengl_impl::GraphicsOpenGLImpl,
    window::{AppWindow, CanvasWindow, Target, WindowHashMap, WindowIDDHashMap},
};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32Handle};

#[allow(unused)]
use windows::{
    core::*, Data::Xml::Dom::*, Win32::Foundation::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

#[allow(unused)]
pub struct ApplicationBuilder {
    app_name: String,
    config_file_path: PathBuf,
}

#[derive(Debug, Copy, Clone)]
struct InputStateWrapper {
    pub mods: ModifiersState,
    pub mouse_state: ElementState,
    pub mouse_pos: PhysicalPosition<f64>,
    pub mouse_begin: PhysicalPosition<f64>,
    pub mouse_prev_pos: PhysicalPosition<f64>,
    pub mouse_btn: MouseButton,
}

impl Default for InputStateWrapper {
    fn default() -> Self {
        InputStateWrapper {
            mods: ModifiersState::empty(),
            mouse_state: ElementState::Released,
            mouse_begin: From::from((0i32, 0i32)),
            mouse_pos: From::from((0i32, 0i32)),
            mouse_prev_pos: From::from((0i32, 0i32)),
            mouse_btn: MouseButton::Left,
        }
    }
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        ApplicationBuilder {
            app_name: "".to_owned(),
            config_file_path: PathBuf::from("".to_owned()),
        }
    }
    pub fn with_name(mut self, name: &str) -> Self {
        self.app_name = name.to_owned();
        self
    }

    #[allow(unused)]
    pub fn with_config_file_path<U: AsRef<PathBuf>>(mut self, path: U) -> Self {
        self.config_file_path = path.as_ref().to_owned();
        self
    }

    fn platform_config(&self, windowed_context: &ContextWrapper<PossiblyCurrent, Window>) {
        if cfg!(windows) {
            let handle = windowed_context.window().raw_window_handle();
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
    }

    fn reload_keybinding_actions(&self) -> Vec<KeyBinding<VirtualKeyCode>> {
        vec![
            KeyBinding {
                action: Action::ImageCapture,
                mods: ModifiersState::CTRL | ModifiersState::ALT,
                key: VirtualKeyCode::Key1,
            },
            KeyBinding {
                action: Action::GifCapture,
                mods: ModifiersState::CTRL | ModifiersState::ALT,
                key: VirtualKeyCode::Key2,
            },
            KeyBinding {
                action: Action::Suspend,
                mods: ModifiersState::empty(),
                key: VirtualKeyCode::Escape,
            },
        ]
    }

    fn create_graphics<'a, 'b>(
        &self,
        windowed_context: &'a ContextWrapper<PossiblyCurrent, Window>,
        event_loop: &'b EventLoop<UserEvent>,
    ) -> Box<dyn Graphics> {
        use std::cell::RefCell;

        let monitor = event_loop
            .available_monitors().last()
            .expect("Invalid monitor handle");
        let size = monitor.size();
        let render_api = support::load(windowed_context);
        Box::new(GraphicsOpenGLImpl {
            render_api: RefCell::new(render_api),
            desktop_size: (size.width, size.height),
        })
    }

    fn create_window_context(
        &self,
        event_loop: &EventLoop<UserEvent>,
    ) -> ContextWrapper<NotCurrent, Window> {
        let wb = WindowBuilder::new()
            .with_title(self.app_name.clone())
            .with_decorations(false)
            .with_transparent(true)
            .with_maximized(true)
            .with_always_on_top(false)
            .with_visible(true);

        let windowed_context = ContextBuilder::new()
            .with_gl_profile(glutin::GlProfile::Core)
            .build_windowed(wb, event_loop)
            .unwrap();
        windowed_context
    }

    ///
    ///Create the fullscreen transparent window for the region selector
    fn create_main_window(
        &self,
        event_loop: &EventLoop<UserEvent>,
        window: &mut WindowHashMap,
        window_index: &mut WindowIDDHashMap,
    ) {
        let windowed_context = self.create_window_context(event_loop);

        let windowed_context = unsafe { windowed_context.make_current().expect("make current") };

        // windowed_context.window().set_outer_position(LogicalPosition::new(0, 0));

        self.platform_config(&windowed_context);

        let graphics = self.create_graphics(&windowed_context, &event_loop);

        let window_id = windowed_context.window().id();
        let win = CanvasWindow::new(
            windowed_context,
            graphics,
            event_loop.create_proxy(),
            Target::Window(AppWindow::RegionSelectorCanvasWindow),
        );

        window_index
            .entry(AppWindow::RegionSelectorCanvasWindow)
            .or_insert(window_id);
        window.entry(window_id).or_insert(Box::new(win));
    }

    pub fn build(self, event_loop: &EventLoop<UserEvent>) -> std::io::Result<Application> {
        let mut window_id_hashmap = WindowIDDHashMap::new();
        let mut window_hashmap = WindowHashMap::new();
        self.create_main_window(event_loop, &mut window_hashmap, &mut window_id_hashmap);

        let app = Application {
            app_name: self.app_name.clone(),
            event_proxy: event_loop.create_proxy(),
            capture_device: CaptureDevice::new()?,
            keybinding_actions: self.reload_keybinding_actions(),
            windows: window_hashmap,
            windows_index: window_id_hashmap,
            state: Cell::new(InputStateWrapper::default()),
        };
        Ok(app)
    }
}

#[allow(unused)]
pub struct Application {
    app_name: String,
    keybinding_actions: Vec<KeyBinding<VirtualKeyCode>>,
    event_proxy: EventLoopProxy<UserEvent>,
    capture_device: CaptureDevice,
    windows: WindowHashMap,
    windows_index: WindowIDDHashMap,
    state: Cell<InputStateWrapper>,
}

impl Application {
    pub fn handle_device_keyboard_event(&mut self, input: KeyboardInput) {
        input.virtual_keycode.map(|k| {
            let mut app_ctx = AppContext {
                event_proxy: &mut self.event_proxy,
                window_hash: &mut self.windows,
                window_id_hash: &mut self.windows_index,
                capture_device: &mut self.capture_device,
            };
            #[allow(deprecated)]
            let mods = input.modifiers;
            for binding in &self.keybinding_actions {
                if binding.is_triggered(mods, k) {
                    binding.action.execute(&mut app_ctx);
                }
            }
        });
    }

    pub fn find_window(&mut self, app_window: AppWindow) -> Option<&mut dyn WindowEventHandler> {
        if let Some(win) = self.windows_index.get(&app_window) {
            if let Some(main_window) = self.windows.get_mut(win) {
                return Some(main_window.deref_mut());
            }
        }
        None
    }

    pub fn handle_user_event(&mut self, data: UserEvent) {
        println!("handle_user_event: {:?}", data);
        match (data.sender, data.receiver, data.event) {
            (_, Target::Window(app_window), _) => {
                match app_window {
                    AppWindow::AllWindow => {
                        // handle event for all windows
                        for (_, win) in self.windows.iter_mut() {
                            win.on_user_event(&data);
                        }
                    }
                    _ => {
                        self.find_window(app_window).map(|w| w.on_user_event(&data));
                    }
                }
            }
            (_, Target::Action, crate::app::event::Event::DoAction(action)) => {
                // handle event for action
                let mut app_ctx = AppContext {
                    event_proxy: &mut self.event_proxy,
                    window_hash: &mut self.windows,
                    window_id_hash: &mut self.windows_index,
                    capture_device: &mut self.capture_device,
                };
                action.execute(&mut app_ctx);
            }
            _ => {
                //log::warn!("Wrong User Event");
            }
        }
    }

    pub fn run(&mut self, event_loop: EventLoop<UserEvent>) {
        let mut event_loop = event_loop;
        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { window_id, .. } | Event::RedrawRequested(window_id) => {
                    if let Some(window) = self.windows.get_mut(&window_id) {
                        match event {
                            Event::WindowEvent { event, .. } => {
                                // translate mouse event
                                let mut s = self.state.get();
                                match event {
                                    #[allow(unused)]
                                    WindowEvent::Resized(physical_size) => (),
                                    WindowEvent::ModifiersChanged(modifier) => {
                                        s.mods = modifier;
                                    }
                                    WindowEvent::KeyboardInput { input, .. } => {
                                        input.virtual_keycode.map(|k| {
                                            let data = KeyInputData { virtual_keycode: k };
                                            window.on_keyboard_event(&data);
                                        });
                                    }
                                    WindowEvent::CursorMoved { .. }
                                    | WindowEvent::MouseInput { .. } => match event {
                                        WindowEvent::CursorMoved { position, .. } => {
                                            s.mouse_prev_pos = From::from((position.x, position.y));
                                            if s.mouse_state == ElementState::Pressed {
                                                s.mouse_pos = From::from((position.x, position.y));
                                                let mouse_data = MouseData {
                                                    button: s.mouse_btn,
                                                    position,
                                                };
                                                window.on_mouse_move_event(&mouse_data);
                                            }
                                        }
                                        WindowEvent::MouseInput { state, button, .. } => {
                                            s.mouse_state = state;
                                            let mouse_data = MouseData {
                                                button,
                                                position: s.mouse_prev_pos,
                                            };
                                            match state {
                                                ElementState::Pressed => {
                                                    s.mouse_begin = s.mouse_prev_pos;
                                                    s.mouse_btn = button;
                                                    window.on_mouse_press_event(&mouse_data);
                                                }
                                                ElementState::Released => {
                                                    s.mouse_begin = s.mouse_prev_pos;
                                                    s.mouse_btn = button;
                                                    window.on_mouse_release_event(&mouse_data);
                                                }
                                            }
                                        }
                                        _ => {
                                            panic!("unexpected mouse event")
                                        }
                                    },
                                    WindowEvent::Focused(focus) => {
                                        window.on_focus_event(focus);
                                    }
                                    _ => (),
                                }
                                self.state.set(s);
                            }
                            Event::RedrawRequested(..) => {
                                window.handle_redraw_event();
                            }
                            _ => (),
                        }
                    } else {
                        // log::warn!("No such window: {:?}", window_id);
                    }
                }
                Event::UserEvent(user_event) => {
                    self.handle_user_event(user_event);
                }
                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::Key(input) => {
                        self.handle_device_keyboard_event(input);
                    }
                    _ => (),
                },
                _ => (),
            }
        });
    }
}
