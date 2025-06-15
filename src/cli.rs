use spin::Mutex;
use lazy_static::lazy_static;
use crate::drivers::vga_buffer::WRITER;
use crate::{println, print};

pub struct Cli {
    input_buffer: [u8; 128],
    buffer_index: usize,
    active: bool,
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
            active: false,
        }
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.clear_input();
        self.display_prompt();
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    fn clear_input(&mut self) {
        self.input_buffer = [0; 128];
        self.buffer_index = 0;
    }

    fn display_prompt(&self) {
        let mut writer = WRITER.lock();
        writer.write_string("> ");
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

    fn clear(&mut self) {
        let mut writer = WRITER.lock();

        writer.clear_screen();
    }

    fn handle_char(&mut self, c: char) {
        if self.buffer_index < self.input_buffer.len() - 1 {
            self.input_buffer[self.buffer_index] = c as u8;
            self.buffer_index += 1;
            print!("{}", c);
        }
    }

    fn handle_backspace(&mut self) {
        if self.buffer_index > 0 {
            self.buffer_index -= 1;
            self.input_buffer[self.buffer_index] = 0;

            let mut writer = WRITER.lock();
            writer.backspace();
        }
    }

    fn execute_command(&mut self) {
        let raw_input = core::str::from_utf8(&self.input_buffer[..self.buffer_index])
            .unwrap_or("")
            .trim();

        println!(); // New line after command

        let mut parts = raw_input.splitn(2, ' ');
        let command = parts.next().unwrap_or("");
        let args = parts.next().unwrap_or("");

        match command {
            "hello" => println!("Hello World!"),
            "yeet" => self.clear(),
            "scream" => println!("{}", args),
            "bye" => {
                println!("See ya, nerd.");
                delay();
                use x86_64::instructions::port::Port;
                unsafe {
                    let mut port: Port<u16> = Port::new(0x604);
                    port.write(0x2000);
                }

                loop {
                    x86_64::instructions::hlt();
                }
            },
            "oops" => {
                println!("Oopsie daisy. Rebooting...");
                delay();
                unsafe {
                    use x86_64::instructions::port::Port;
                    let mut port: Port<u8> = Port::new(0x64);
                    while port.read() & 0x02 != 0 {}
                    let mut port: Port<u8> = Port::new(0x64);
                    port.write(0xFE);
                }
            },
            "" => {},
            _ => println!("Unknown command: {}", command),
        }

        self.clear_input();
        self.display_prompt();
    }
}

lazy_static! {
    pub static ref CLI: Mutex<Cli> = Mutex::new(Cli::new());
}