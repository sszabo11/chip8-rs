use log::{info, trace, warn};
use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};
use std::{fs, io::stdout};

const RAM_SIZE: usize = 4096;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

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
        dbg!("{}", rom.len());
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

        dbg!(
            "First byte: {:x} {:b} {}",
            self.memory[self.pc as usize],
            self.memory[self.pc as usize],
            self.memory[self.pc as usize]
        );
        dbg!(
            "Second byte: {:x} {:b} {}",
            self.memory[self.pc as usize + 1],
            self.memory[self.pc as usize + 1],
            self.memory[self.pc as usize + 1]
        );
        //dbg!("Bytes: {}", first_byte + second_byte);

        let op = (first_byte << 8) | second_byte;
        //let combined = second_byte << 8;

        //dbg!("Shift to left: {:x} {:b}", combined, combined);
        dbg!("OP: {:x} {:b} {}", op, op, op);
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
            (1, n1, n2, n3) => {
                // JUMP
                self.pc = (n1 << 8) | (n2 << 4) | n3;

                dbg!("n1 n2 n3: {:x} {:x} {:x}", n1, n2, n3);
                dbg!("PC: {:x}", self.pc);
            }

            (2, n1, n2, n3) => {
                // CALL
                self.pc = (n1 << 8) | (n2 << 4) | n3;
                dbg!("n1 n2 n3: {:x} {:x} {:x}", n1, n2, n3);
                dbg!("PC: {:x}", self.pc);

                //dbg!("nn1: {:x} nn2: {:x} nn3: {:x}", n1)

                self.push_to_stack(self.pc);
            }
            (0, 0, 0xE, 0xE) => {
                // RET
                self.pc = self.pop_from_stack();
                dbg!("PC: {:x}", self.pc);
            }

            (6, x, n1, n2) => {
                // SET REGISTER TO VX

                self.v_reg[x as usize] = ((n1 << 4) | n2) as u8;
            }

            (7, x, n1, n2) => {
                // ADD VALUE REGISTER TO VX

                // TODO
                //self.v_reg[x as usize] = ((n1 << 4) | n2) as u8;
            }
            (0xA, n1, n2, n3) => {
                // SET INDEX REGISTER I

                self.i_reg = (n1 << 8) | (n2 << 4) | n3;
            }
            (0xD, x, y, n) => {
                // DISPLAY/DRAW

                //let d = self.i_reg as f64 / SCREEN_WIDTH as f64;

                //let start_y = d.floor() as u16;

                //let start_x = self.i_reg as usize % SCREEN_WIDTH;

                //dbg!("i: {} x: {} y: {}", self.i_reg, start_x, start_y);

                let x_coord = self.v_reg[x as usize] as u16;
                let y_coord = self.v_reg[y as usize] as u16;

                dbg!("Coords x: {} y: {}", x_coord, y_coord);

                let mut flipped = false;
                for i in 0..n {
                    let sprite_byte = self.memory[(self.i_reg + i) as usize];

                    dbg!("Spr btye: {:b} {:x}", sprite_byte, sprite_byte);
                    for j in 0..8 {
                        //let pixel_bit = sprite_byte[j];

                        dbg!(
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
            _ => panic!("Invalid opcode: {:x}", op),
        };
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
            .window("rust-sdl2 demo", 800, 600)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        loop {
            // Fectch
            let op = self.fetch_instruction();

            // Decode and execute
            let exec = self.execute_instruction(op);

            self.draw(&mut canvas)
        }
    }
}

fn main() {
    let mut chip8 = Chip8::new();

    let rom = fs::read("./rom/ibm.ch8").expect("Failed to read rom");

    //chip8.execute_instruction(0x2D21);

    //chip8.execute_instruction(0xD123);

    chip8.load_rom(rom.as_slice());
    chip8.run();
}
