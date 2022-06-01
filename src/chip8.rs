use crate::{
    instr::{bytes_to_word, split_into_4bits, Instructions},
    mem::Mem,
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
// const KEYMAP: [Scancode; 16] = [
//     Scancode::X,
//     Scancode::Num0,
//     Scancode::Num2,
//     Scancode::Num3,
//     Scancode::Q,
//     Scancode::W,
//     Scancode::E,
//     Scancode::A,
//     Scancode::S,
//     Scancode::D,
//     Scancode::Z,
//     Scancode::C,
//     Scancode::Num4,
//     Scancode::R,
//     Scancode::F,
//     Scancode::V,
// ];

pub struct Chip8 {
    pub v: [u8; 16],
    pub i: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub pc: u16,
    // pub sp: u8,
    pub stack: Vec<u16>,
    pub mem: Mem,
    pub keys: [bool; 16],
    pub gfx: [bool; HEIGHT * WIDTH],
    pub paused: bool,
    pub target_register: Option<usize>,
}

impl Chip8 {
    pub fn new(mem: Mem) -> Self {
        Chip8 {
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            // sp: 0,
            stack: Vec::with_capacity(16),
            mem,
            keys: [false; 16],
            gfx: [false; HEIGHT * WIDTH],
            paused: false,
            target_register: None,
        }
    }

    pub fn main_loop(&mut self) {
        // let lines = [
        //     Point::new(0, (HEIGHT * 4) as i32 + 1),
        //     Point::new((WIDTH * 4) as i32 + 1, (HEIGHT * 4) as i32 + 1),
        //     Point::new((WIDTH * 4) as i32 + 1, 0),
        // ];

        'running: loop {
            // for (i, scancode) in KEYMAP.iter().enumerate() {
            //     self.keys[i] = event_pump.keyboard_state().is_scancode_pressed(*scancode);

            //     if self.paused && self.keys[i] {
            //         self.v[self.target_register.unwrap()] = i as u8;
            //         self.target_register = None;
            //         self.paused = false;
            //     }
            // }
        }
    }

    fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            // audio.start_beep();
            self.sound_timer -= 1;
        } else {
            // audio.stop_beep();
        }
    }

    pub fn execute(&mut self) {
        if self.paused {
            return;
        }

        let (h, l) = (self.mem.get(self.pc), self.mem.get(self.pc + 1));
        let opcode = bytes_to_word(h, l);

        self.pc += 2;

        match split_into_4bits(opcode) {
            (0, 0, 0xE, 0) => self.cls(),
            (0, 0, 0xE, 0xE) => self.ret(),
            (0, _, _, _) => self.sys_addr(),
            (1, _, _, _) => self.jp_addr(opcode),
            (2, _, _, _) => self.call_addr(opcode),
            (3, _, _, _) => self.se_vx_kk(opcode),
            (4, _, _, _) => self.sne_vx_kk(opcode),
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
            (0xF, _, 0, 0xA) => self.ld_vx_k(opcode),
            (0xF, _, 1, 5) => self.ld_dt_vx(opcode),
            (0xF, _, 1, 8) => self.ld_st_vx(opcode),
            (0xF, _, 1, 0xE) => self.add_i_vx(opcode),
            (0xF, _, 2, 9) => self.ld_f_vx(opcode),
            (0xF, _, 3, 3) => self.ld_b_vx(opcode),
            (0xF, _, 5, 5) => self.ld_addri_vx(opcode),
            (0xF, _, 6, 5) => self.ld_vx_addri(opcode),

            _ => panic!("Unimplemented: {:#06X}", opcode),
        }

        self.tick_timers();
    }
}
