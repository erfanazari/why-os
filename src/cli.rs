use spin::Mutex;
use lazy_static::lazy_static;
use crate::drivers::vga_buffer::{WRITER, BUFFER_WIDTH, Color, ALL_COLORS};
use crate::{os_info, println};
use pc_keyboard::KeyCode;
use crate::ramfs;
use alloc::string::{String, ToString};

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

fn num_to_string(mut num: usize) -> String {
    if num == 0 {
        return String::from("0");
    }

    let mut buf = [0u8; 20]; // Enough for 64-bit numbers
    let mut i = 20;
    while num > 0 {
        i -= 1;
        buf[i] = b'0' + (num % 10) as u8;
        num /= 10;
    }

    // Convert the relevant slice to String
    let mut s = String::new();
    for &b in &buf[i..] {
        s.push(b as char);
    }
    s
}


const PROMPT: &str = "> ";
const PROMPT_LEN: usize = 2;

pub struct Cli {
    input_buffer: [u8; 128],
    buffer_index: usize,
    cursor_index: usize,
    active: bool,
    prompt_row: usize,
    current_dir: String,
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
            current_dir: "/".to_string(),
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
        writer.write_string((self.current_dir.clone() + PROMPT).as_str());
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
        let total_len = (self.current_dir.clone() + PROMPT).as_str().len() + self.buffer_index;
        let rows = total_len / BUFFER_WIDTH + 1;

        // Clear all affected rows
        for i in 0..rows {
            writer.set_cursor(self.prompt_row + i, 0);
            writer.clear_current_line();
        }

        // Reset to prompt start
        writer.set_cursor(self.prompt_row, 0);

        // Draw prompt
        writer.write_string((self.current_dir.clone() + PROMPT).as_str());

        // Draw input buffer
        for i in 0..self.buffer_index {
            writer.write_byte(self.input_buffer[i]);
        }

        // Absolute cursor positioning
        let visual_index = (self.current_dir.clone() + PROMPT).as_str().len() + self.cursor_index;
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
            "ls" => {
                if let Some(entries) = ramfs::list_dir(&*self.current_dir, args) {
                    for e in entries {
                        println!(" - {}", e);
                    }
                }
            },
            "banner" => {
                println!("          _            ____   _____ ");
                println!("         | |          / __ \\ / ____|");
                println!("__      _| |__  _   _| |  | | (___  ");
                println!("\\ \\ /\\ / / '_ \\| | | | |  | |\\___ \\ ");
                println!(" \\ V  V /| | | | |_| | |__| |____) |");
                println!("  \\_/\\_/ |_| |_|\\__, |\\____/|_____/ ");
                println!("                 __/ |              ");
                println!("                |___/    v{}     \n", os_info::VERSION);
            },
            "memtest" => {
                let mut file_index = 0;

                loop {
                    // Generate unique filename: file_0.txt, file_1.txt, ...
                    let mut filename = String::from("file_");
                    filename.push_str(&num_to_string(file_index));
                    filename.push_str(".txt");

                    let success = ramfs::create_file(&*self.current_dir, &filename, "HEEsduhkghdfjkhdfkjghdfjkghdfkghdfkjghdfkjghdfkjghdfkjghdfghdfjkghdfjkghdfkghdfkjghdfkghdfjkghdfjkghdfjkghdfjkghdfjghdfkghdfjkghdfkjghdfjkghdfkjghdfjkghdfjkghdfkjghdfjkghdfkghdfjkhdfjkghdfjkghdfkhdfgjkdfgfgddfjkhdfjkdfgjkhdfg".as_ref());
                    if !success {
                        println!("Failed to create file {}", filename);
                        break;
                    }

                    if file_index % 100 == 0 {
                        println!("Created {} files so far...", file_index);
                    }

                    file_index += 1;
                }

                println!("Stress test finished. Created {} files.", file_index);
            },
            "cd" => {
                if let Some(new_dir) = ramfs::change_directory(&*self.current_dir, args) {
                    self.current_dir = new_dir;
                    println!("Changed to {}", self.current_dir); // "/home"
                } else {
                    println!("Path not found!");
                }
            },
            "mkfile" => {
                ramfs::create_file(&*self.current_dir, args, "".as_ref());
            },
            "mkdir" => {
                ramfs::mkdir(&*self.current_dir, args);
            },
            "rem" => {
                ramfs::delete(&*self.current_dir, args);
            },
            "readfile" => {
                if let Some(data) = ramfs::read_file(&*self.current_dir, args) {
                    let text = core::str::from_utf8(&data).unwrap();
                    println!("file contents: {}", text);
                }
            },
            "hello" => println!("Hello World!"),
            "whyver" => {
                println!("OS Name: {}", crate::os_info::NAME);
                println!("OS Version: {}", crate::os_info::VERSION);
                println!("Description: {}", crate::os_info::DESCRIPTION);
                println!("GitHub: {}", crate::os_info::GITHUB);
            },
            "info" => {
                match args {
                    "ls" => {
                        println!("Lists files and directories in the current directory.");
                    }
                    "cd" => {
                        println!("Changes the current directory.\nUsage: cd <path>");
                    }
                    "mkfile" => {
                        println!("Creates an empty file in the current directory.\nUsage: mkfile <filename>");
                    }
                    "mkdir" => {
                        println!("Creates a new directory in the current directory.\nUsage: mkdir <dirname>");
                    }
                    "rem" => {
                        println!("Removes a file or directory.\nUsage: rem <name>");
                    }
                    "readfile" => {
                        println!("Reads and prints the contents of a file.\nUsage: readfile <filename>");
                    }
                    "banner" => {
                        println!("Displays the system banner and OS version.");
                    }
                    "whyver" => {
                        println!("Shows information about the current OS release.");
                    }
                    "memtest" => {
                        println!(
                            "Stress-tests the RAM filesystem by continuously creating files\n\
                 until allocation fails. Useful for testing memory limits."
                        );
                    }
                    "hello" => {
                        println!("Prints \"Hello World!\" to the screen.");
                    }
                    "scream" => {
                        println!("Echoes the given text back to the screen.\nUsage: scream <text>");
                    }
                    "yeet" => {
                        println!("Clears the screen.");
                    }
                    "bye" => {
                        println!("Shuts down the system. (may not work on real hardware)");
                    }
                    "oops" => {
                        println!("Reboots the system. (may not work on real hardware)");
                    }
                    "listcolors" => {
                        println!("Lists all available text colors.");
                    }
                    "setfg" => {
                        println!(
                            "Sets the foreground (text) color.\n\
                 Usage: setfg <color>\n\
                 Available colors can be seen using \"listcolors\"."
                        );
                    }
                    "setbg" => {
                        println!(
                            "Sets the background color.\n\
                 Usage: setbg <color>\n\
                 Available colors can be seen using \"listcolors\"."
                        );
                    }
                    "info" => {
                        println!(
                            "Explains what a command does.\n\
                 Usage: info <command>"
                        );
                    }
                    "" => {
                        println!("Usage: info <command>");
                        println!("Try: info ls");
                    }
                    _ => {
                        println!("No information available for command: {}", args);
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

                // 1️⃣ Try QEMU ACPI poweroff
                unsafe {
                    use x86_64::instructions::port::Port;
                    let mut port: Port<u16> = Port::new(0x604);
                    port.write(0x2000);
                }

                delay();
                delay();

                unsafe {
                    use x86_64::instructions::port::Port;
                    let mut port: Port<u8> = Port::new(0x64);
                    while port.read() & 0x02 != 0 {}
                    port.write(0xFE);
                }

                loop {
                    x86_64::instructions::hlt();
                }
            },
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
            },
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