use log::{info, trace, warn};
use rand::random;
use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, video::Window,
};
use std::{fs, io::stdout};

const RAM_SIZE: usize = 4096;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;

const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

const SCALE: u32 = 15;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Chip8 {
    memory: [u8; RAM_SIZE],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    pc: u16,
    stack: [u16; STACK_SIZE],
    delay_timer: u8,
    sound_timer: u8,
    keys: [bool; NUM_KEYS],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    opcode: u16,
}

impl Chip8 {
    fn new() -> Self {
        let mut emu = Self {
            memory: [0; RAM_SIZE],
            sp: 0,
            pc: START_ADDR,
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            stack: [0; STACK_SIZE],
            delay_timer: 0,
            keys: [false; NUM_KEYS],
            sound_timer: 0,
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            opcode: 0,
        };

        emu.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        emu
    }

    fn load_rom(&mut self, rom: &[u8]) {
        println!("{}", rom.len());
        let size = rom.len() + START_ADDR as usize;
        self.memory[START_ADDR as usize..size].copy_from_slice(rom);

        for i in self.memory {
            //print!("{:x}, ", i);
        }
    }

    fn push_to_stack(&mut self, value: u16) {
        self.stack[self.sp as usize] = value;
        self.sp += 1;
    }
    fn pop_from_stack(&mut self) -> u16 {
        self.sp -= 1;
        let value = self.stack[self.sp as usize];
        self.stack[self.sp as usize] = 0;
        value
    }

    fn reset(&mut self) {
        self.pc = START_ADDR;
        self.sp = 0;
        self.i_reg = 0;
        self.sound_timer = 0;
        self.delay_timer = 0;
        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        self.v_reg = [0; NUM_REGS];
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.opcode = 0;
    }

    fn fetch_instruction(&mut self) -> u16 {
        let first_byte: u16 = self.memory[self.pc as usize] as u16; // USIZE WILL OVERFLOW?
        let second_byte: u16 = self.memory[(self.pc + 1) as usize] as u16;

        println!(
            "First byte: {:x} {:b} {}",
            self.memory[self.pc as usize],
            self.memory[self.pc as usize],
            self.memory[self.pc as usize]
        );
        println!(
            "Second byte: {:x} {:b} {}",
            self.memory[self.pc as usize + 1],
            self.memory[self.pc as usize + 1],
            self.memory[self.pc as usize + 1]
        );
        //println!("Bytes: {}", first_byte + second_byte);

        let op = (first_byte << 8) | second_byte;
        //let combined = second_byte << 8;

        //println!("Shift to left: {:x} {:b}", combined, combined);
        println!("OP: {:x} {:b} {}", op, op, op);
        self.pc += 2;
        op
    }

    fn execute_instruction(&mut self, op: u16) {
        let digits: (u16, u16, u16, u16) = (op >> 12, (op >> 8) & 0xF, (op >> 4) & 0xF, op & 0xF);
        match digits {
            (0, 0, 0, 0) => return, // NOP
            (0, 0, 0xE, 0) => {
                // CLEAR SCREEN
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            (0, 0, 0xE, 0xE) => {
                // RET
                self.pc = self.pop_from_stack();
                println!("PC: {:x}", self.pc);
            }
            (1, n1, n2, n3) => {
                println!("-- JUMP --");
                // JUMP
                self.pc = (n1 << 8) | (n2 << 4) | n3;
            }

            (2, n1, n2, n3) => {
                // CALL
                println!("n1 n2 n3: {:x} {:x} {:x}", n1, n2, n3);
                println!("PC: {:x}", self.pc);

                self.push_to_stack(self.pc);
                //self.pc = (n1 << 8) | (n2 << 4) | n3;
                self.pc = (n1 << 8) | (n2 << 4) | n3;
                println!("PC a: {:x}", self.pc);
            }
            (3, x, n1, n2) => {
                // Skip if VX = NN
                let vx = self.v_reg[x as usize] as u16;

                let nn = (n1 << 4) | n2;

                if vx == nn {
                    self.pc += 2;
                }
            }
            (4, x, n1, n2) => {
                // Skip if VX != NN
                let vx = self.v_reg[x as usize] as u16;

                let nn = (n1 << 4) | n2;

                if vx != nn {
                    self.pc += 2;
                }
            }
            (5, x, y, 0) => {
                // Skip if VX = VY
                let vx = self.v_reg[x as usize] as u16;
                let vy = self.v_reg[y as usize] as u16;

                if vx == vy {
                    self.pc += 2;
                }
            }
            (6, x, n1, n2) => {
                // SET REGISTER TO VX

                self.v_reg[x as usize] = ((n1 << 4) | n2) as u8;
            }

            (7, x, _, _) => {
                // ADD VALUE REGISTER TO VX

                self.v_reg[x as usize] = self.v_reg[x as usize].wrapping_add((op & 0xFF) as u8);
            }
            (8, x, y, 0) => {
                // Set VX to VY
                let vy = self.v_reg[y as usize];

                self.v_reg[x as usize] = vy;
            }
            (8, x, y, 1) => {
                // Set VX to bitwise OR of VX and VY
                let vy = self.v_reg[y as usize];

                self.v_reg[x as usize] |= vy;
            }
            (8, x, y, 2) => {
                // Set VX to bitwise AND of VX and VY
                let vy = self.v_reg[y as usize];

                self.v_reg[x as usize] &= vy;
            }
            (8, x, y, 3) => {
                // Set VX to bitwise XOR of VX and VY
                let vy = self.v_reg[y as usize];

                self.v_reg[x as usize] ^= vy;
            }
            (8, x, y, 4) => {
                // Set VX to VX + VY
                let vy = self.v_reg[y as usize];
                let vx = self.v_reg[x as usize];

                let (new_vx, carry) =
                    self.v_reg[x as usize].overflowing_add(self.v_reg[y as usize]);

                println!("added new vx: {} {} {}", new_vx, vx, vy);
                // Set carry bit
                self.v_reg[0xF] = if carry { 1 } else { 0 };

                self.v_reg[x as usize] = new_vx;
            }
            (8, x, y, 5) => {
                // Set VX to VX - VY
                let vy = self.v_reg[y as usize];
                let vx = self.v_reg[x as usize];

                let (new_vx, borrow) =
                    self.v_reg[x as usize].overflowing_sub(self.v_reg[y as usize]);

                println!("sub new vx: {} {} {}", new_vx, vx, vy);
                // Set carry bit
                self.v_reg[0xF] = if borrow { 0 } else { 1 };

                self.v_reg[x as usize] = new_vx;
            }

            (8, x, y, 6) => {
                // Right shift
                // Put VY into VX and shift the value in VX 1 bit to the right.
                // Set flag register to the bit shiftet out
                let vy = self.v_reg[y as usize];
                let vx = self.v_reg[x as usize];

                let lsb = self.v_reg[x as usize] & 1;
                self.v_reg[x as usize] >>= 1;
                self.v_reg[0xF] = lsb;
            }
            (8, x, y, 0xE) => {
                // Left shift ?
                let msb = (self.v_reg[x as usize] >> 7) & 1;
                println!("msb: {:x}", msb);
                self.v_reg[x as usize] <<= 1;
                println!("vreg: {:x}", self.v_reg[x as usize]);
                self.v_reg[0xF] = msb;
            }
            (8, x, y, 7) => {
                // Set VX to VY - VX
                let vy = self.v_reg[y as usize];
                let vx = self.v_reg[x as usize];

                let (new_vx, borrow) =
                    self.v_reg[y as usize].overflowing_sub(self.v_reg[x as usize]);

                println!("sub new vx: {} {} {}", new_vx, vx, vy);
                // Set carry bit
                self.v_reg[0xF] = if borrow { 0 } else { 1 };

                self.v_reg[x as usize] = new_vx;
            }
            (9, x, y, 0) => {
                // Skip if VX != VY
                let vx = self.v_reg[x as usize] as u16;
                let vy = self.v_reg[y as usize] as u16;

                if vx != vy {
                    self.pc += 2;
                }
            }

            (0xA, n1, n2, n3) => {
                // SET INDEX REGISTER I

                //let nnn = op & 0xFFF;
                //self.i_reg = nnn;
                self.i_reg = (n1 << 8) | (n2 << 4) | n3;
            }
            (0xB, n1, n2, n3) => {
                let v0 = self.v_reg[0] as u16;
                let nnn = (n1 << 8) | (n2 << 4) | n3;

                let added = nnn.wrapping_add(v0);
                let added2 = v0 + (op & 0xFFF);
                println!("nnn: {}", nnn);
                println!("opn: {}", op & 0xFFF);
                println!("added {} added 2: {}", added, added2);

                self.pc = added;
            }
            (0xC, x, n1, n2) => {
                // Generate random number and AND it with NN
                let rand: u8 = random();

                let value = ((n1 << 4) | n2) as u8;

                self.v_reg[x as usize] = rand & value;
            }
            (0xD, x, y, n) => {
                // DISPLAY/DRAW

                let x_coord = self.v_reg[x as usize] as u16;
                let y_coord = self.v_reg[y as usize] as u16;

                println!("Coords x: {} y: {}", x_coord, y_coord);

                let mut flipped = false;
                for i in 0..n {
                    let sprite_byte = self.memory[(self.i_reg + i) as usize];

                    println!("Spr btye: {:b} {:x}", sprite_byte, sprite_byte);
                    for j in 0..8 {
                        //let pixel_bit = sprite_byte[j];

                        println!(
                            "S: {:b} {:x}",
                            sprite_byte & (0b10000000 >> j),
                            sprite_byte & (0b10000000 >> j)
                        );
                        if sprite_byte & (0b10000000 >> j) != 0 {
                            let x = (x_coord + j) as usize % SCREEN_WIDTH;
                            let y = (y_coord + i) as usize % SCREEN_WIDTH;

                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_reg[15] = 1;
                } else {
                    self.v_reg[15] = 0;
                }

                //let start = self.screen[self.i_reg as usize];
            }
            (0xE, x, 9, 0xE) => {
                // Skip if key in VX is pressed
                let key = self.v_reg[x as usize];
                if self.keys[key as usize] {
                    self.pc += 2;
                }
            }
            (0xE, x, 0xA, 1) => {
                // Skip if key in VX is not pressed
                let key = self.v_reg[x as usize];
                if !self.keys[key as usize] {
                    self.pc += 2;
                }
            }
            (0xF, x, 0, 7) => {
                // Sets VX to the current value of the delay timer

                self.v_reg[x as usize] = self.delay_timer;
            }
            (0xF, x, 1, 5) => {
                // Sets the delay timer to the value in VX

                self.delay_timer = self.v_reg[x as usize];
            }
            (0xF, x, 1, 8) => {
                // Sets the sound timer to the value in VX

                self.sound_timer = self.v_reg[x as usize];
            }
            (0xF, x, 1, 0xE) => {
                // Add the value in VX to the index register I.
                // ADD OTPION

                self.i_reg += self.v_reg[x as usize] as u16;
            }
            (0xF, x, 0, 0xA) => {
                // Waits for key input

                println!("WAIT");
                let key = self.keys.iter().enumerate().find(|(i, k)| **k);
                if let Some(key) = key {
                    self.v_reg[x as usize] = key.0 as u8;
                } else {
                    println!("No key");
                }
            }

            (0xF, x, 2, 9) => {
                self.i_reg = self.v_reg[x as usize] as u16 * 5;
            }
            (0xF, x, 3, 3) => {
                let vx = self.v_reg[x as usize] as f64;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0).floor() as u8;

                self.memory[self.i_reg as usize] = hundreds;
                self.memory[self.i_reg as usize + 1] = tens;
                self.memory[self.i_reg as usize + 2] = ones;
                //r.trunc()

                //if vx / 10.0 < 1.0 {
                //        let n = (vx / 10.0).rou
                //} else if vx / 100.0 < 1.0 {

                // } else if vx / 1000.0 < 1.0 {

                //}
            }
            (0xF, x, 5, 5) => {
                // Stores the value in the registers from V0 to VX into memory from the address in I
                // Eg. V0 holds 0x20; I holds 0x40; memory[0x40] = 0x20;
                // Maybe option

                for j in 0..=x {
                    self.memory[self.i_reg as usize + j as usize] = self.v_reg[j as usize];
                }
            }
            (0xF, x, 6, 5) => {
                // Takes values in memory addresses V0 to VX and loads them into the variable registers
                // Maybe option

                for j in 0..=x {
                    self.v_reg[j as usize] = self.memory[self.i_reg as usize + j as usize]
                }
            }
            _ => panic!("Invalid opcode: {:x}", op),
        };
    }

    fn parse_key(&mut self, keycode: Keycode) -> Option<usize> {
        match keycode {
            Keycode::Num0 => Some(0x0),
            Keycode::Num1 => Some(0x1),
            Keycode::Num2 => Some(0x2),
            Keycode::Num3 => Some(0x3),
            Keycode::Num4 => Some(0xC),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xD),
            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xE),
            Keycode::Z => Some(0xA),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xB),
            Keycode::V => Some(0xF),
            _ => None,
        }
    }

    fn key_press(&mut self, keycode: usize, pressed: bool) {
        self.keys[keycode] = pressed;
    }

    fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (i, pixel) in self.screen.iter().enumerate() {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;
            if *pixel {
                let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
                canvas.fill_rect(rect).unwrap();
            }
        }
        canvas.present();
    }

    fn run(&mut self) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Chip8", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();

        canvas.clear();
        canvas.present();

        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut last_cycle_time = std::time::Instant::now();

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown { keycode, .. } => {
                        let key = self.parse_key(keycode.expect("No key"));
                        if let Some(key) = key {
                            self.key_press(key, true);
                        }
                    }
                    Event::KeyUp { keycode, .. } => {
                        let key = self.parse_key(keycode.expect("No key"));
                        if let Some(key) = key {
                            self.key_press(key, false);
                        }
                    }
                    _ => {}
                };
            }
            let now = std::time::Instant::now();
            if last_cycle_time + std::time::Duration::from_micros(16) <= now {
                // Fetch
                let op = self.fetch_instruction();

                // Decode and execute
                let exec = self.execute_instruction(op);

                self.draw(&mut canvas);

                last_cycle_time = std::time::Instant::now();
            }
            canvas.present();
        }
    }
}

fn main() {
    let mut chip8 = Chip8::new();

    //let rom = fs::read("./rom/chip8.ch8").expect("Failed to read rom");

    let rom = fs::read("./rom/test_opcode.ch8").expect("Failed to read rom");

    //chip8.execute_instruction(0x2D21);

    //chip8.execute_instruction(0xD123);

    chip8.load_rom(rom.as_slice());
    chip8.run();
}
