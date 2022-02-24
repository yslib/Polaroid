pub trait Graphics {
    /// Draw a given rect on desktop
    fn draw_rect(&self, x: i32, y: i32, w: u32, h: u32);

    fn draw_rect_frame(&self, x: i32, y: i32, w: u32, h: u32);

    fn clear(&self, color: (f32, f32, f32, f32));
}
