use glutin::{
    dpi::{LogicalPosition, PhysicalPosition},
    event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
};

use super::{
    action::Action,
    window::{AppWindow, Target},
};

#[derive(Debug)]
pub struct MouseData {
    pub button: MouseButton,
    pub position: PhysicalPosition<f64>,
}

#[derive(Debug)]
pub struct KeyInputData {
    pub virtual_keycode: VirtualKeyCode,
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    InvokeRegionSelector(Action),
    DoAction(Action),
}

#[derive(Debug, Clone, Copy)]
pub struct UserEvent {
    pub sender: Target,
    pub receiver: Target,
    pub event: Event,
}

impl UserEvent {
    pub fn new(sender: Target, receiver: Target, event: Event) -> Self {
        Self {
            sender,
            receiver,
            event,
        }
    }

    pub fn build_action_event(sender: Target, receiver: Target, action: Action) -> Self {
        Self {
            sender,
            receiver,
            event: Event::DoAction(action),
        }
    }
}

pub trait WindowEventHandler {
    fn on_mouse_press_event(&mut self, data: &MouseData);

    fn on_mouse_release_event(&mut self, data: &MouseData);

    fn on_mouse_move_event(&mut self, data: &MouseData);

    fn on_keyboard_event(&mut self, data: &KeyInputData);

    fn handle_redraw_event(&mut self);

    fn on_user_event(&mut self, data: &UserEvent);

    fn set_visible(&mut self, visible: bool);

    fn send_user_event(&self, receiver: Target, event: Event);

    fn on_focus_event(&mut self, focus: bool);
}
