use bootloader_api::info::{FrameBuffer, FrameBufferInfo};
use conquer_once::spin::OnceCell;
use font_constants::BACKUP_CHAR;
use lazy_static::lazy_static;
use noto_sans_mono_bitmap::{get_raster, RasterizedChar};
use spin::Mutex;
use x86_64::instructions::interrupts;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::sys::framebuffer::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    interrupts::without_interrupts(|| {
        write!(WRITER.get().unwrap().lock(), "{args}").unwrap();
    });
}

lazy_static! {
    pub static ref WRITER: OnceCell<Mutex<FrameBufferWriter>> = OnceCell::uninit();
}

pub fn init(framebuffer: &'static mut FrameBuffer) {
    let info = framebuffer.info();

    let writer = FrameBufferWriter::new(framebuffer.buffer_mut(), info);

    WRITER.init_once(|| Mutex::new(writer));
}

const LINE_SPACING: usize = 2;
const LETTER_SPACING: usize = 0;

const BORDER_PADDING: usize = 2;

mod font_constants {
    use noto_sans_mono_bitmap::{get_raster_width, FontWeight, RasterHeight};

    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;

    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FONT_WEIGHT, CHAR_RASTER_HEIGHT);

    pub const BACKUP_CHAR: char = '�';
}

fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(
            c,
            font_constants::FONT_WEIGHT,
            font_constants::CHAR_RASTER_HEIGHT,
        )
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}

#[derive(Debug)]
pub struct FrameBufferWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl FrameBufferWriter {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut writer = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
        };

        writer.clear();

        writer
    }

    fn newline(&mut self) {
        self.y_pos += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return();
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.framebuffer.fill(0);
    }

    const fn width(&self) -> usize {
        self.info.width
    }

    const fn height(&self) -> usize {
        self.info.height
    }

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            '\u{8}' => {
                if self.x_pos > BORDER_PADDING {
                    self.x_pos -= font_constants::CHAR_RASTER_WIDTH;

                    self.write_rendered_char(&get_char_raster(' '));

                    self.x_pos -= font_constants::CHAR_RASTER_WIDTH;
                }
            }
            '\u{20}'..='\u{7e}' => {
                let new_x = self.x_pos + font_constants::CHAR_RASTER_WIDTH;
                if new_x >= self.width() {
                    self.newline();
                }

                let new_y = self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_y >= self.height() {
                    self.clear();
                }

                self.write_rendered_char(&get_char_raster(c));
            }
            _ => {}
        }
    }

    fn write_rendered_char(&mut self, rendered_char: &RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format {
            bootloader_api::info::PixelFormat::Rgb => [intensity, intensity, intensity / 2, 0],
            bootloader_api::info::PixelFormat::Bgr => [intensity / 2, intensity, intensity, 0],
            bootloader_api::info::PixelFormat::U8 => {
                [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0]
            }
            other => {
                self.info.pixel_format = bootloader_api::info::PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in framebuffer", other)
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { core::ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl core::fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
