use std::time::Duration;

use crate::{
    audio::Audio,
    instr::{bytes_to_word, split_into_4bits, Instructions},
    mem::Mem,
};

use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::Color,
    rect::Rect,
    video::Window,
    Sdl,
};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

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

enum Operation {
    None,
    ClearDisplay,
    WaitForKey(u8),
}

pub struct Chip8 {
    pub v: [u8; 16],
    pub i: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub pc: u16,
    pub sp: u8,
    pub stack: Vec<u16>,
    pub mem: Mem,
    pub keys: [bool; 16],
    pub gfx: [[bool; HEIGHT]; WIDTH],
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
            canvas.clear();
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
            (0, 0, 0xE, 0) => {
                self.cls();
                ret = Operation::ClearDisplay;
            }
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

        ret
    }

    fn ld_vx(&mut self, x: u8, key_value: u8) {
        self.v[x as usize] = key_value;
    }
}
