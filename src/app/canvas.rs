use crate::app::graphics::Graphics;
use glm::vec2;
use glutin::{monitor::MonitorHandle, window, window::Window, ContextWrapper, PossiblyCurrent};
use std::rc::Rc;

#[derive(Debug, Copy, Clone)]
pub struct Bound2 {
    pub min: (i32, i32),
    pub max: (i32, i32),
}

impl Bound2 {
    pub fn new(p1: (i32, i32), p2: (i32, i32)) -> Self {
        Bound2 {
            min: (std::cmp::min(p1.0, p2.0), std::cmp::min(p1.1, p2.1)),
            max: (std::cmp::max(p1.0, p2.0), std::cmp::max(p1.1, p2.1)),
        }
    }

    pub fn rect(&self) -> (i32, i32, u32, u32) {
        let min = self.min;
        let max = self.max;
        (min.0, min.1, (max.0 - min.0) as u32, (max.1 - min.1) as u32)
    }

    pub fn get_width(&self) -> u32 {
        (self.max.0 - self.min.0) as u32
    }

    pub fn get_height(&self) -> u32 {
        (self.max.1 - self.min.1) as u32
    }

    pub fn empty(&self) -> bool {
        self.get_width() <= 0 || self.get_height() <= 0
    }
}

impl Default for Bound2 {
    fn default() -> Self {
        Bound2 {
            min: (0, 0),
            max: (-1, -1),
        }
    }
}

pub trait Renderable {
    fn update(&self, graphics: &dyn Graphics);
}

pub struct RegionSelector {
    pub bound: Bound2,
    visible: bool,
    first: (i32, i32),
}

impl RegionSelector {
    pub fn new() -> Self {
        RegionSelector {
            bound: Bound2::default(),
            visible: false,
            first: (0, 0),
        }
    }

    pub fn set_first(&mut self, pos: (i32, i32)) {
        self.first = pos;
        self.bound = Bound2::new(self.first, self.first);
    }

    pub fn set_second(&mut self, pos: (i32, i32)) {
        self.bound = Bound2::new(self.first, pos);
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl Renderable for RegionSelector {
    fn update(&self, graphics: &dyn Graphics) {
        if self.visible {
            let rect = self.bound.rect();
            graphics.draw_rect(rect.0, rect.1, rect.2, rect.3);
        }
    }
}

pub struct Canvas {
    pub objects: Vec<Rc<dyn Renderable>>,
    pub graphics: Box<dyn Graphics>,
}

impl Canvas {
    pub fn add_object(&mut self, object: Rc<dyn Renderable>) {
        self.objects.push(object);
    }

    pub fn on_draw(&self) {
        self.graphics.clear((0.0, 0.0, 0.0, 0.0));
        for obj in &self.objects {
            obj.update(&(*self.graphics));
        }
    }
}
