use glutin::{
    event_loop::EventLoopProxy,
    window::{Window, WindowId},
    ContextWrapper, PossiblyCurrent, WindowedContext,
};
use windows::Win32::UI::WindowsAndMessaging::RegisterClassA;
use crate::app::{
    action::Action,
    canvas::{Bound2, RegionSelector, Renderable, Canvas},
    event::{Event, KeyInputData, MouseData, UserEvent},
    graphics::Graphics,
    window::{Target, EventListener, WindowInstance, AppWindow}
};

pub struct GifSelectorWindowImpl {
    pub region_selector: RegionSelector,
    win_inst: WindowInstance,
    window_id: Target,
}

impl GifSelectorWindowImpl{
    pub fn new(win_inst:WindowInstance)->Self{
        GifSelectorWindowImpl{
            window_id:Target::Window(AppWindow::GifSelectorCanvasWindow),
            region_selector: RegionSelector::new(),
            win_inst:win_inst
        }
    }
}

impl EventListener for GifSelectorWindowImpl {
    fn on_mouse_press_event(&mut self, data: &MouseData) {
        self.region_selector.set_visible(true);
        self.region_selector.set_first(data.position.into());
    }

    #[allow(unused)]
    fn on_mouse_release_event(&mut self, data: &MouseData) {
        let bound = self.region_selector.bound;
        if bound.empty() == false {
            let action = Action::DoGifCapture(bound);
            self.win_inst.send_user_event(self.window_id, Target::Action, Event::DoAction(action));
            self.win_inst.request_redraw();
        }
    }


    fn on_mouse_move_event(&mut self, data: &MouseData) {
        self.region_selector.set_second(data.position.into());
        self.win_inst.request_redraw();
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
        self.region_selector.update(&*self.win_inst.graphics); // ???
    }

    ///
    /// This can receive user event whenever the window is visible or unvisible
    fn on_user_event(&mut self, data: &UserEvent) {
        match data.event {
            crate::app::event::Event::InvokeRegionSelector(action) => {
                self.set_visible(true);
            }
            _ => {}
        }
    }

    fn set_visible(&mut self, visible:bool) {
        self.win_inst.set_visible(visible);
    }

}

