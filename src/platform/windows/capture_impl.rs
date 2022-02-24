use crate::app::canvas::Bound2;
use image::{
    DynamicImage::ImageRgba8, GenericImage, GenericImageView, ImageBuffer, RgbImage, Rgba,
    RgbaImage,
};

use windows::{
    core::*,
    Data::Xml::Dom::*,
    Win32::System::{DataExchange::*, Threading::*},
    Win32::{Foundation::*, Graphics::Gdi::*, System::Threading::*, UI::WindowsAndMessaging::*},
};

use std::ffi::c_void;
use std::mem;

pub struct CaptureImplWin {
    pub hwnd: HWND,
    pub hdc_src: HDC,
    pub hdc_mem: CreatedHDC,
    pub bitmap: HBITMAP,
    pub old_bitmap: HGDIOBJ,
    pub bi: BITMAPINFOHEADER,
    pub raw_data_bgra: Vec<u8>,
    pub bound: Bound2,
    pub bitmap_info: BITMAP,
}

unsafe impl Send for CaptureImplWin {}

impl Drop for CaptureImplWin {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(self.hwnd, self.hdc_src);
            DeleteDC(self.hdc_mem);
            DeleteObject(self.bitmap);
        }
    }
}

/// This is the capture routine on windows using GDI
impl CaptureImplWin {
    pub fn new(hwnd: HWND, bound: Bound2) -> Self {
        let rect = bound.rect();
        unsafe {
            let hdc_src = GetDC(hwnd); // the whole desktop
            let hdc_mem = CreateCompatibleDC(hdc_src);
            let bitmap = CreateCompatibleBitmap(hdc_src, rect.2 as i32, rect.3 as i32);
            let old_bitmap = SelectObject(hdc_mem, bitmap);
            let mut bitmap_info: BITMAP = BITMAP::default();

            GetObjectW(
                bitmap,
                mem::size_of::<BITMAP>() as i32,
                (&mut bitmap_info) as *mut _ as *mut c_void,
            );

            let mut bi: BITMAPINFOHEADER = BITMAPINFOHEADER::default();

            bi.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
            bi.biWidth = bitmap_info.bmWidth;
            bi.biHeight = bitmap_info.bmHeight;
            bi.biPlanes = 1;
            bi.biBitCount = 32;
            bi.biCompression = BI_RGB as u32;
            bi.biSizeImage = 0;
            bi.biXPelsPerMeter = 0;
            bi.biYPelsPerMeter = 0;
            bi.biClrUsed = 0;
            bi.biClrImportant = 0;

            let dw_bmp_size =
                ((bitmap_info.bmWidth * bi.biBitCount as i32 + 31) / 32) * 4 * bitmap_info.bmHeight;

            let mut raw_data = vec![0u8; dw_bmp_size as usize];

            CaptureImplWin {
                hwnd,
                hdc_src,
                hdc_mem,
                bitmap,
                old_bitmap,
                bi,
                raw_data_bgra: raw_data,
                bound,
                bitmap_info,
            }
        }
    }

    pub fn begin_capture(&mut self) {
        unsafe {
            self.old_bitmap = SelectObject(self.hdc_mem, self.bitmap);
        }
    }

    pub fn end_capture(&mut self) {
        unsafe {
            SelectObject(self.hdc_mem, self.old_bitmap);
        }
    }

    pub async fn capture_image_async(&mut self) -> RgbaImage {
        self.capture_image()
    }

    pub fn capture_image(&mut self) -> RgbaImage {
        self.capture_image_raw();
        let img = ImageBuffer::from_fn(
            self.bitmap_info.bmWidth as u32,
            self.bitmap_info.bmHeight as u32,
            |x, y| {
                let ind = (self.bitmap_info.bmHeight as u32 - 1u32 - y)
                    * self.bitmap_info.bmWidth as u32
                    + x;
                let ind = ind as usize;
                unsafe {
                    let ptr = self.raw_data_bgra.as_ptr().add(ind * 4);
                    image::Rgba([*ptr.add(0), *ptr.add(1), *ptr.add(2), *ptr.add(3)])
                }
            },
        );
        img
    }

    ///
    /// Capture the specified region of screen to data buffer
    pub fn capture_image_raw(&mut self) {
        let rect = self.bound.rect();
        unsafe {
            // transfer pixel data from screen
            BitBlt(
                self.hdc_mem,
                0,
                0,
                rect.2 as i32,
                rect.3 as i32,
                self.hdc_src,
                rect.0 as i32,
                rect.1 as i32,
                SRCCOPY,
            );

            // copy it to memory
            GetDIBits(
                self.hdc_mem,
                HBITMAP(self.bitmap.0),
                0,
                self.bitmap_info.bmHeight as u32,
                self.raw_data_bgra.as_mut_ptr() as *mut c_void,
                &mut self.bi as *mut _ as *mut BITMAPINFO,
                DIB_RGB_COLORS,
            );

            for chunck in self.raw_data_bgra.chunks_mut(4) {
                // convert bgra to rgba
                let z = chunck[0];
                chunck[0] = chunck[2];
                chunck[2] = z;
            }
        }
    }
}

#[allow(unused)]
fn capture_img_from_screen_once(hwnd: HWND, rect: Bound2) -> RgbaImage {
    let rect = rect.rect();
    unsafe {
        let hdc_src = GetDC(hwnd); // the whole desktop
        let hdc_mem = CreateCompatibleDC(hdc_src);
        let bitmap = CreateCompatibleBitmap(hdc_src, rect.2 as i32, rect.3 as i32);
        let old_bitmap = SelectObject(hdc_mem, bitmap);

        BitBlt(
            hdc_mem,
            0,
            0,
            rect.2 as i32,
            rect.3 as i32,
            hdc_src,
            rect.0 as i32,
            rect.1 as i32,
            SRCCOPY,
        );

        let mut bitmap_info: BITMAP = BITMAP::default();
        use std::ffi::c_void;
        use std::mem;
        GetObjectW(
            bitmap,
            mem::size_of::<BITMAP>() as i32,
            (&mut bitmap_info) as *mut _ as *mut c_void,
        );

        let mut bi: BITMAPINFOHEADER = BITMAPINFOHEADER::default();

        bi.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
        bi.biWidth = bitmap_info.bmWidth;
        bi.biHeight = bitmap_info.bmHeight;
        bi.biPlanes = 1;
        bi.biBitCount = 32;
        bi.biCompression = BI_RGB as u32;
        bi.biSizeImage = 0;
        bi.biXPelsPerMeter = 0;
        bi.biYPelsPerMeter = 0;
        bi.biClrUsed = 0;
        bi.biClrImportant = 0;

        let dw_bmp_size =
            ((bitmap_info.bmWidth * bi.biBitCount as i32 + 31) / 32) * 4 * bitmap_info.bmHeight;

        let mut raw_data = vec![0u8; dw_bmp_size as usize];

        GetDIBits(
            hdc_mem,
            HBITMAP(bitmap.0),
            0,
            bitmap_info.bmHeight as u32,
            raw_data.as_mut_ptr() as *mut c_void,
            &mut bi as *mut _ as *mut BITMAPINFO,
            DIB_RGB_COLORS,
        );

        let bitmap = SelectObject(hdc_mem, old_bitmap);

        let img = ImageBuffer::from_fn(
            bitmap_info.bmWidth as u32,
            bitmap_info.bmHeight as u32,
            |x, y| {
                let ind = (bitmap_info.bmHeight as u32 - 1u32 - y) * bitmap_info.bmWidth as u32 + x;
                let ind = ind as usize;
                let ptr = raw_data.as_ptr().add(ind * 4);
                image::Rgba([*ptr.add(2), *ptr.add(1), *ptr, *ptr.add(3)])
            },
        );

        ReleaseDC(HWND(0), hdc_src);
        DeleteDC(hdc_mem);

        DeleteObject(bitmap);
        img
    }
}
