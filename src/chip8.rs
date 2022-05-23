use std::time::Duration;

use crate::{audio::Audio, mem::Mem};

use rand::prelude::*;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::Color,
    video::Window,
    Sdl,
};

/*
_________________
| 1 | 2 | 3 | 4 |
-----------------
| Q | W | E | R |
-----------------
| A | S | D | F |
-----------------
| Z | X | C | V |
-----------------
*/
const KEYMAP: [Scancode; 16] = [
    Scancode::X,
    Scancode::Kp1,
    Scancode::Kp2,
    Scancode::Kp3,
    Scancode::Q,
    Scancode::W,
    Scancode::E,
    Scancode::A,
    Scancode::S,
    Scancode::D,
    Scancode::Z,
    Scancode::C,
    Scancode::Kp4,
    Scancode::R,
    Scancode::F,
    Scancode::V,
];

pub struct Chip8 {
    v: [u8; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    sp: u8,
    stack: Vec<u16>,
    mem: Mem,
    keys: [bool; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        // TODO What are the default values ?
        Chip8 {
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: Vec::with_capacity(16),
            mem: Mem::new(),
            keys: [false; 16],
        }
    }

    pub fn main_loop(&mut self) {
        let (sdl_context, window, audio) = self.init_sdl();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let mut event_pump = sdl_context.event_pump().unwrap();

        'running: loop {
            for (i, scancode) in KEYMAP.iter().enumerate() {
                self.keys[i] = event_pump.keyboard_state().is_scancode_pressed(*scancode);
            }

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            canvas.clear();
            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        }
    }

    fn init_sdl(&mut self) -> (Sdl, Window, Audio) {
        let sdl_context = sdl2::init().unwrap();

        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("chipr", 800, 600)
            .position_centered()
            .build()
            .unwrap();

        let _audio_context = sdl_context.audio().unwrap();

        let audio = Audio::new(&sdl_context);

        (sdl_context, window, audio)
    }

    #[allow(dead_code)]
    fn execute(&mut self) {
        let (h, l) = (self.mem.get(self.pc), self.mem.get(self.pc + 1));
        let opcode = bytes_to_word(h, l);

        match split_into_4bits(opcode) {
            (0, 0, 0xE, 0) => self.cls(),
            (0, 0, 0xE, 0xE) => self.ret(),
            (0, _, _, _) => self.sys_addr(),
            (1, _, _, _) => self.jp_addr(opcode),
            (2, _, _, _) => self.call_addr(opcode),
            (3, _, _, _) => self.se_vx_b(opcode),
            (4, _, _, _) => self.sne_vx_b(opcode),
            (5, _, _, _) => self.se_vx_vy(opcode),
            (6, _, _, _) => self.ld_vx_kk(opcode),
            (7, _, _, _) => self.add_vx_kk(opcode),
            (8, _, _, 0) => self.ld_vx_vy(opcode),
            (8, _, _, 1) => self.or_vx_vy(opcode),
            (8, _, _, 2) => self.and_vx_vy(opcode),
            (8, _, _, 3) => self.xor_vx_vy(opcode),
            (8, _, _, 4) => self.add_vx_vy(opcode),
            (8, _, _, 5) => self.sub_vx_vy(opcode),
            (8, _, _, 6) => self.shr_vx_vy(opcode),

            _ => panic!("Unimplemented: {:#06X}", opcode),
        }

        self.pc += 2;
    }

    // 0nnn
    fn sys_addr(&mut self) {
        todo!("Jump to a machine code routine at nnn")
    }

    // 00E0
    fn cls(&mut self) {
        todo!("Clear the display");
    }

    // 00EE
    fn ret(&mut self) {
        todo!("Return from a subroutine");
    }

    // 1nnn
    fn jp_addr(&mut self, opcode: u16) {
        let addr = opcode & 0b0111;
        self.pc = addr;
    }

    // 2nnn
    fn call_addr(&mut self, opcode: u16) {
        let addr = opcode & 0b0111;

        self.sp += 1;
        self.stack.push(self.pc);
        self.pc = addr;
    }

    // 3xkk
    fn se_vx_b(&mut self, opcode: u16) {
        let (x, kk) = get_xkk(opcode);

        if self.v[x] == kk {
            self.skip_instruction();
        }
    }

    // 4xkk
    fn sne_vx_b(&mut self, opcode: u16) {
        let (x, kk) = get_xkk(opcode);

        if self.v[x] != kk {
            self.skip_instruction();
        }
    }

    // 5xy0
    fn se_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        if self.v[x] == self.v[y] {
            self.skip_instruction();
        }
    }

    // 6xkk
    fn ld_vx_kk(&mut self, opcode: u16) {
        let (x, kk) = get_xkk(opcode);

        self.v[x] = kk;
    }

    // 7xkk
    fn add_vx_kk(&mut self, opcode: u16) {
        let (x, kk) = get_xkk(opcode);

        self.v[x] += kk;
    }

    // 8xy0
    fn ld_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        self.v[x] = self.v[y];
    }

    // 8xy1
    fn or_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        self.v[x] |= self.v[y];
    }

    // 8xy2
    fn and_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        self.v[x] &= self.v[y];
    }

    // 8xy3
    fn xor_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        self.v[x] ^= self.v[y];
    }

    // 8xy4
    fn add_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        let (result, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = result;
        self.v[0xF] = if carry { 1 } else { 0 };
    }

    // 8xy5
    fn sub_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        let (result, overflow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = result;
        self.v[y] = if overflow { 0 } else { 1 };
    }

    // 8xy6
    fn shr_vx_vy(&mut self, opcode: u16) {
        let (x, _) = get_xy(opcode);

        // TODO Unsure
        self.v[0xF] = self.v[x] & 1;
        self.v[x] = self.v[x] >> 1;
    }

    // 8xy7
    fn subn_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        let (result, overflow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = result;
        self.v[y] = if overflow { 1 } else { 0 };
    }

    // 8xyE
    fn shl_vx_vy(&mut self, opcode: u16) {
        let (x, _) = get_xy(opcode);

        // TODO Unsure
        self.v[0xF] = self.v[x] & 0b1000_0000;
        self.v[x] = self.v[x] << 1;
    }

    // 9xy0
    fn sne_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        if self.v[x] != self.v[y] {
            self.skip_instruction();
        }
    }

    // Annn
    fn ld_i_addr(&mut self, opcode: u16) {
        let addr = opcode & 0b0111;

        self.i = addr;
    }

    // Bnnn
    fn jp_v0_addr(&mut self, opcode: u16) {
        let addr = opcode & 0b0111;

        self.pc = addr + self.v[0] as u16;
    }

    // Cxkk
    fn rnd_vx_kk(&mut self, opcode: u16) {
        let (x, kk) = get_xkk(opcode);

        let mut rng = rand::thread_rng();
        let rnd: u8 = rng.gen();

        self.v[x] = rnd & kk;
    }

    // Dxyn
    fn drw_vx_vy_nibble(&mut self, opcode: u16) {
        todo!("Implement drawing");
    }

    // Ex9E
    fn skp_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        if self.keys[x as usize] {
            self.skip_instruction();
        }
    }

    // ExA1
    fn sknp_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        if !self.keys[x as usize] {
            self.skip_instruction();
        }
    }

    fn skip_instruction(&mut self) {
        self.pc += 2;
    }
}

fn get_xkk(n: u16) -> (usize, u8) {
    let x = ((n & 0100) >> 2) as usize;
    let kk = (n & 0011) as u8;

    (x, kk)
}

fn get_xy(n: u16) -> (usize, usize) {
    let x = ((n & 0100) >> 2) as usize;
    let y = ((n & 0010) >> 2) as usize;

    (x, y)
}

fn bytes_to_word(h: u8, l: u8) -> u16 {
    ((h as u16) << 8) | l as u16
}

fn split_into_4bits(n: u16) -> (u8, u8, u8, u8) {
    let [h, l] = n.to_be_bytes();
    (h >> 4, h & 0b00001111, l >> 4, l & 0b00001111)
}
