use spin::Mutex;
use lazy_static::lazy_static;
use crate::drivers::vga_buffer::{WRITER, BUFFER_WIDTH, Color, ALL_COLORS};
use crate::{println};
use pc_keyboard::KeyCode;

pub fn get_color_by_name(name: &str) -> Option<Color> {
    match name {
        "Black" => Some(Color::Black),
        "Blue" => Some(Color::Blue),
        "Green" => Some(Color::Green),
        "Cyan" => Some(Color::Cyan),
        "Red" => Some(Color::Red),
        "Magenta" => Some(Color::Magenta),
        "Brown" => Some(Color::Brown),
        "LightGray" => Some(Color::LightGray),
        "DarkGray" => Some(Color::DarkGray),
        "LightBlue" => Some(Color::LightBlue),
        "LightGreen" => Some(Color::LightGreen),
        "LightCyan" => Some(Color::LightCyan),
        "LightRed" => Some(Color::LightRed),
        "Pink" => Some(Color::Pink),
        "Yellow" => Some(Color::Yellow),
        "White" => Some(Color::White),
        _ => None,
    }
}

const PROMPT: &str = "> ";
const PROMPT_LEN: usize = 2;

pub struct Cli {
    input_buffer: [u8; 128],
    buffer_index: usize,
    cursor_index: usize,
    active: bool,
    prompt_row: usize,
}

fn delay() {
    for _ in 0..1_000_000 {
        x86_64::instructions::nop();
    }
}

impl Cli {
    pub fn new() -> Self {
        Cli {
            input_buffer: [0; 128],
            buffer_index: 0,
            cursor_index: 0,
            active: false,
            prompt_row: 0,
        }
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.clear_input();
        self.display_prompt();
    }

    fn clear_input(&mut self) {
        self.input_buffer = [0; 128];
        self.buffer_index = 0;
        self.cursor_index = 0;
    }

    fn display_prompt(&mut self) {
        let mut writer = WRITER.lock();
        self.prompt_row = writer.cursor_row();
        writer.write_string(PROMPT);
    }

    pub fn handle_input(&mut self, c: char) {
        if !self.active {
            return;
        }

        match c {
            '\n' => self.execute_command(),
            '\x08' => self.handle_backspace(),
            _ => self.handle_char(c),
        }
    }

    pub fn handle_special_key(&mut self, key: KeyCode) {
        if !self.active {
            return;
        }

        match key {
            KeyCode::ArrowLeft => {
                if self.cursor_index > 0 {
                    self.cursor_index -= 1;
                    self.redraw_input();
                }
            }
            KeyCode::ArrowRight => {
                if self.cursor_index < self.buffer_index {
                    self.cursor_index += 1;
                    self.redraw_input();
                }
            }
            _ => {}
        }
    }

    fn redraw_input(&self) {
        let mut writer = WRITER.lock();

        // How many rows the input occupies
        let total_len = PROMPT_LEN + self.buffer_index;
        let rows = total_len / BUFFER_WIDTH + 1;

        // Clear all affected rows
        for i in 0..rows {
            writer.set_cursor(self.prompt_row + i, 0);
            writer.clear_current_line();
        }

        // Reset to prompt start
        writer.set_cursor(self.prompt_row, 0);

        // Draw prompt
        writer.write_string(PROMPT);

        // Draw input buffer
        for i in 0..self.buffer_index {
            writer.write_byte(self.input_buffer[i]);
        }

        // Absolute cursor positioning
        let visual_index = PROMPT_LEN + self.cursor_index;
        let row_offset = visual_index / BUFFER_WIDTH;
        let col = visual_index % BUFFER_WIDTH;

        writer.set_cursor(self.prompt_row + row_offset, col);
    }


    fn handle_char(&mut self, c: char) {
        if self.buffer_index >= self.input_buffer.len() - 1 {
            return;
        }

        for i in (self.cursor_index..self.buffer_index).rev() {
            self.input_buffer[i + 1] = self.input_buffer[i];
        }

        self.input_buffer[self.cursor_index] = c as u8;
        self.buffer_index += 1;
        self.cursor_index += 1;

        self.redraw_input();
    }

    fn handle_backspace(&mut self) {
        if self.cursor_index == 0 {
            return;
        }

        for i in self.cursor_index..self.buffer_index {
            self.input_buffer[i - 1] = self.input_buffer[i];
        }

        self.buffer_index -= 1;
        self.cursor_index -= 1;
        self.input_buffer[self.buffer_index] = 0;

        self.redraw_input();
    }

    fn clear(&mut self) {
        WRITER.lock().clear_screen();
    }

    fn execute_command(&mut self) {
        let raw_input = core::str::from_utf8(&self.input_buffer[..self.buffer_index])
            .unwrap_or("")
            .trim();

        println!();

        let mut parts = raw_input.splitn(2, ' ');
        let command = parts.next().unwrap_or("");
        let args = parts.next().unwrap_or("");

        match command {
            "hello" => println!("Hello World!"),
            "whyver" => {
                println!("OS Name: {}", crate::os_info::NAME);
                println!("OS Version: {}", crate::os_info::VERSION);
                println!("Description: {}", crate::os_info::DESCRIPTION);
                println!("GitHub: {}", crate::os_info::GITHUB);
            },
            "info" => {
                match args {
                    "hello" => {
                        println!("It prints \"Hello World!\" on the screen.");
                    },
                    "scream" => {
                        println!("It echoes your message back to you.");
                    },
                    "yeet" => {
                        println!("Clears the screen.");
                    },
                    "bye" => {
                        println!("Shuts down the system. (may not work on real hardware)");
                    },
                    "oops" => {
                        println!("Reboots the system. (may not work on real hardware)");
                    },
                    "listcolors" => {
                        println!("Lists the available colors for this system.");
                    },
                    "setfg" => {
                        println!("Sets the foreground color (the text color) of the screen.\nThe value must only be one of the ones shown in command \"listcolors\".");
                    },
                    "setfg" => {
                        println!("Sets the background color of the screen.\nThe value must only be one of the ones shown in command \"listcolors\".");
                    },
                    "whyver" => {
                        println!("Shows the information about the current OS release on this system.");
                    },
                    "info" => {
                        println!("It explains what every command does.");
                    },
                    _ => {
                        println!("Unknown command: {}", args);
                    }
                }
            },
            "yeet" => self.clear(),
            "scream" => println!("{}", args),
            "setfg" => {
                match get_color_by_name(args) {
                    Some(color) => WRITER.lock().set_foreground(color),
                    None => println!("Invalid color: {}", args),
                }
            },
            "setbg" => {
                match get_color_by_name(args) {
                    Some(color) => WRITER.lock().set_background(color),
                    None => println!("Invalid color: {}", args),
                }
            },
            "listcolors" => {
                for color in ALL_COLORS {
                    println!("{:?}", color);
                }
            },
            "bye" => {
                println!("See ya, nerd.");
                delay();
                delay();
                use x86_64::instructions::port::Port;
                unsafe {
                    let mut port: Port<u16> = Port::new(0x604);
                    port.write(0x2000);
                }
                loop {
                    x86_64::instructions::hlt();
                }
            }
            "oops" => {
                println!("Oopsie daisy. Rebooting...");
                delay();
                delay();
                unsafe {
                    use x86_64::instructions::port::Port;
                    let mut port: Port<u8> = Port::new(0x64);
                    while port.read() & 0x02 != 0 {}
                    port.write(0xFE);
                }
            }
            "" => {}
            _ => println!("Unknown command: {}", command),
        }

        self.clear_input();
        self.display_prompt();
    }
}

lazy_static! {
    pub static ref CLI: Mutex<Cli> = Mutex::new(Cli::new());
}