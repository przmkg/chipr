use std::time::Duration;

use crate::{audio::Audio, mem::Mem};

use rand::prelude::*;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::Color,
    rect::Rect,
    video::Window,
    Sdl,
};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

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

const ADDR_MASK: u16 = 0b0000_1111_1111_1111;

const FONTS: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];

enum Operation {
    None,
    ClearDisplay,
    WaitForKey(u8),
}

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
    gfx: [[bool; HEIGHT]; WIDTH],
}

impl Chip8 {
    pub fn new(mem: Mem) -> Self {
        // TODO What are the default values ?
        Chip8 {
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: Vec::with_capacity(16),
            mem,
            keys: [false; 16],
            gfx: [[false; HEIGHT]; WIDTH],
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

            match self.execute() {
                Operation::None => {}
                Operation::ClearDisplay => canvas.clear(),
                // TODO Wait for keypress
                Operation::WaitForKey(target_register) => todo!(),
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

            canvas.set_draw_color(Color::WHITE);
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    if self.gfx[x][y] {
                        canvas
                            .fill_rect(Rect::new(x as i32 * 4, y as i32 * 4, 4, 4))
                            .unwrap();
                    }
                }
            }

            canvas.set_draw_color(Color::BLACK);
            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        }
    }

    fn init_sdl(&mut self) -> (Sdl, Window, Audio) {
        let sdl_context = sdl2::init().unwrap();

        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("chipr", 4 * WIDTH as u32, 4 * HEIGHT as u32)
            .position_centered()
            .build()
            .unwrap();

        let _audio_context = sdl_context.audio().unwrap();

        let audio = Audio::new(&sdl_context);

        (sdl_context, window, audio)
    }

    #[allow(dead_code)]
    fn execute(&mut self) -> Operation {
        let (h, l) = (self.mem.get(self.pc), self.mem.get(self.pc + 1));
        let opcode = bytes_to_word(h, l);

        println!("Opcode: {:#06X}", opcode);

        let mut ret = Operation::None;

        match split_into_4bits(opcode) {
            (0, 0, 0xE, 0) => ret = Operation::ClearDisplay,
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
            (8, _, _, 7) => self.subn_vx_vy(opcode),
            (8, _, _, 0xE) => self.shl_vx_vy(opcode),
            (9, _, _, 0) => self.sne_vx_vy(opcode),
            (0xA, _, _, _) => self.ld_i_addr(opcode),
            (0xB, _, _, _) => self.jp_v0_addr(opcode),
            (0xC, _, _, _) => self.rnd_vx_kk(opcode),
            (0xD, _, _, _) => self.drw_vx_vy_nibble(opcode),
            (0xE, _, 9, 0xE) => self.skp_vx(opcode),
            (0xE, _, 0xA, 1) => self.sknp_vx(opcode),
            (0xF, _, 0, 7) => self.ld_vx_dt(opcode),
            (0xF, _, 0, 0xA) => ret = Operation::WaitForKey(self.ld_vx_k(opcode)),
            (0xF, _, 1, 5) => self.ld_dt_vx(opcode),
            (0xF, _, 1, 8) => self.ld_st_vx(opcode),
            (0xF, _, 1, 0xE) => self.add_i_vx(opcode),
            (0xF, _, 2, 9) => self.ld_f_vx(opcode),
            (0xF, _, 3, 3) => self.ld_b_vx(opcode),
            (0xF, _, 5, 5) => self.ld_addri_vx(opcode),
            (0xF, _, 6, 5) => self.ld_vx_addri(opcode),

            _ => panic!("Unimplemented: {:#06X}", opcode),
        }

        self.pc += 2;

        ret
    }

    // 0nnn
    fn sys_addr(&mut self) {
        todo!("Jump to a machine code routine at nnn");
    }

    // 00E0
    // fn cls(&mut self) {}

    // 00EE
    fn ret(&mut self) {
        if let Some(value) = self.stack.pop() {
            self.pc = value;
            self.sp -= 1;
        }
    }

    // 1nnn
    fn jp_addr(&mut self, opcode: u16) {
        let addr = opcode & ADDR_MASK;
        self.pc = addr;
    }

    // 2nnn
    fn call_addr(&mut self, opcode: u16) {
        let addr = opcode & ADDR_MASK;

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
        let addr = opcode & ADDR_MASK;

        self.i = addr;
    }

    // Bnnn
    fn jp_v0_addr(&mut self, opcode: u16) {
        let addr = opcode & ADDR_MASK;

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
        // TODO Not very pretty
        let (x, y, n) = get_xyn(opcode);
        let pos_x = self.v[x] as usize;
        let pos_y = self.v[y] as usize;
        let bytes = self.mem.read_bytes(self.i, n);

        bytes.iter().enumerate().for_each(|(idx_y, byte)| {
            let bits = byte_to_bit_array(*byte);

            for (idx_x, bit) in bits.iter().enumerate() {
                // TODO Set the flag
                if pos_x + idx_x < WIDTH && pos_y + idx_y < HEIGHT {
                    self.gfx[pos_x + idx_x][pos_y + idx_y] ^= *bit;
                }
            }
        });
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

    // Fx07
    fn ld_vx_dt(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        self.v[x as usize] = self.delay_timer;
    }

    // Fx0A
    fn ld_vx_k(&mut self, opcode: u16) -> u8 {
        let (x, _) = get_xkk(opcode);
        x as u8
    }

    // Fx15
    fn ld_dt_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        self.delay_timer = self.v[x as usize];
    }

    // Fx18
    fn ld_st_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        self.sound_timer = self.v[x as usize];
    }

    // Fx1E
    fn add_i_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        self.i += self.v[x as usize] as u16;
    }

    // Fx29
    fn ld_f_vx(&mut self, opcode: u16) {
        // todo!("Set location of sprite");
    }

    // Fx33
    fn ld_b_vx(&mut self, opcode: u16) {
        // todo!("Store BCD representation");
    }

    // Fx55
    fn ld_addri_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        for i in 0..=x {
            let addr = self.i + i as u16;
            self.mem.set(addr, self.v[i]);
        }
    }

    // Fx65
    fn ld_vx_addri(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        for i in 0..=x {
            let addr = self.i + i as u16;
            self.v[i] = self.mem.get(addr);
        }
    }

    fn skip_instruction(&mut self) {
        self.pc += 2;
    }

    fn ld_vx(&mut self, x: u8, key_value: u8) {
        self.v[x as usize] = key_value;
    }
}

fn get_xkk(n: u16) -> (usize, u8) {
    let x = ((n & 0b0000_1111_0000_0000) >> 8) as usize;
    let kk = (n & 0b0000_0000_1111_1111) as u8;

    (x, kk)
}

fn get_xy(n: u16) -> (usize, usize) {
    let x = ((n & 0b0000_1111_0000_0000) >> 8) as usize;
    let y = ((n & 0b0000_0000_1111_0000) >> 4) as usize;

    (x, y)
}

fn get_xyn(n: u16) -> (usize, usize, u8) {
    let (x, y) = get_xy(n);
    let nn = (n & 0b1111) as u8;

    (x, y, nn)
}

fn bytes_to_word(h: u8, l: u8) -> u16 {
    ((h as u16) << 8) | l as u16
}

fn split_into_4bits(n: u16) -> (u8, u8, u8, u8) {
    let [h, l] = n.to_be_bytes();
    (h >> 4, h & 0b0000_1111, l >> 4, l & 0b0000_1111)
}

fn byte_to_bit_array(b: u8) -> [bool; 8] {
    [
        ((b & 0b1000_0000) >> 7) == 1,
        ((b & 0b0100_0000) >> 6) == 1,
        ((b & 0b0010_0000) >> 5) == 1,
        ((b & 0b0001_0000) >> 4) == 1,
        ((b & 0b0000_1000) >> 3) == 1,
        ((b & 0b0000_0100) >> 2) == 1,
        ((b & 0b0000_0010) >> 1) == 1,
        (b & 0b0000_0001) == 1,
    ]
}
