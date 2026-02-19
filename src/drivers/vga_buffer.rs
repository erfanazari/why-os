use volatile::Volatile;
use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

pub const ALL_COLORS: [Color; 16] = [
    Color::Black,
    Color::Blue,
    Color::Green,
    Color::Cyan,
    Color::Red,
    Color::Magenta,
    Color::Brown,
    Color::LightGray,
    Color::DarkGray,
    Color::LightBlue,
    Color::LightGreen,
    Color::LightCyan,
    Color::LightRed,
    Color::Pink,
    Color::Yellow,
    Color::White,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode
}

#[repr(transparent)]
pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    background_color: Color,
    foreground_color: Color,
    buffer: &'static mut Buffer
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        background_color: Color::Black,
        foreground_color: Color::Green,
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn cursor_row(&self) -> usize {
        self.row_position
    }

    pub fn cursor_col(&self) -> usize {
        self.column_position
    }

    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.row_position = row;
        self.column_position = col;
        self.update_cursor();
    }

    pub fn set_color_code(&mut self) {
        self.color_code = ColorCode::new(self.foreground_color, self.background_color);
        self.clear_screen();
    }

    pub fn set_foreground(&mut self, color: Color) {
        self.foreground_color = color;
        self.set_color_code();
    }

    pub fn set_background(&mut self, color: Color) {
        self.background_color = color;
        self.set_color_code();
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code
                });
                self.column_position += 1;

                if self.column_position < BUFFER_WIDTH {
                    self.buffer.chars[row][self.column_position].write(ScreenChar {
                        ascii_character: b' ',
                        color_code,
                    });
                }

                self.update_cursor();
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
            self.column_position = 0;
        } else {
            // Scroll everything up
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
            self.column_position = 0;
        }
        self.update_cursor();
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
        self.update_cursor();
    }

    pub fn backspace(&mut self) {
        if self.column_position == 0 {
            if self.row_position > 0 {
                self.row_position -= 1;
                self.column_position = BUFFER_WIDTH - 1;
            }
        } else {
            self.column_position -= 1;
        }

        let row = self.row_position;
        let col = self.column_position;
        self.buffer.chars[row][col].write(ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        });
        self.update_cursor();
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
        self.row_position = 0;
        self.update_cursor();
    }

    pub fn clear_current_line(&mut self) {
        let row = self.row_position;

        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }

        self.column_position = 0;
        self.update_cursor();
    }

    pub fn update_cursor(&self) {
        let pos = (self.row_position * BUFFER_WIDTH + self.column_position) as u16;

        unsafe {
            let mut command_port: Port<u8> = Port::new(0x3D4);
            let mut data_port: Port<u8> = Port::new(0x3D5);

            // Low byte of cursor pos
            command_port.write(0x0F);
            data_port.write((pos & 0xFF) as u8);

            // High byte
            command_port.write(0x0E);
            data_port.write((pos >> 8) as u8);
        }
    }

}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;   // new

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}