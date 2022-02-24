pub struct CaptureConfig {
    pub gif_capture_fps: u32,
}

impl CaptureConfig {
    pub fn new() -> Self {
        CaptureConfig {
            gif_capture_fps: 15,
        }
    }
}

impl Default for CaptureConfig {
    fn default() -> Self {
        CaptureConfig {
            gif_capture_fps: 15,
        }
    }
}
