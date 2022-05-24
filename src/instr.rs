use crate::chip8::{Chip8, HEIGHT, WIDTH};
use rand::prelude::*;

const ADDR_MASK: u16 = 0xFFF;

pub trait Instructions {
    // 0nnn
    fn sys_addr(&mut self);

    // 00E0
    fn cls(&mut self);

    // 00EE
    fn ret(&mut self);

    // 1nnn
    fn jp_addr(&mut self, opcode: u16);

    // 2nnn
    fn call_addr(&mut self, opcode: u16);

    // 3xkk
    fn se_vx_b(&mut self, opcode: u16);

    // 4xkk
    fn sne_vx_b(&mut self, opcode: u16);

    // 5xy0
    fn se_vx_vy(&mut self, opcode: u16);

    // 6xkk
    fn ld_vx_kk(&mut self, opcode: u16);

    // 7xkk
    fn add_vx_kk(&mut self, opcode: u16);

    // 8xy0
    fn ld_vx_vy(&mut self, opcode: u16);

    // 8xy1
    fn or_vx_vy(&mut self, opcode: u16);

    // 8xy2
    fn and_vx_vy(&mut self, opcode: u16);

    // 8xy3
    fn xor_vx_vy(&mut self, opcode: u16);

    // 8xy4
    fn add_vx_vy(&mut self, opcode: u16);

    // 8xy5
    fn sub_vx_vy(&mut self, opcode: u16);

    // 8xy6
    fn shr_vx_vy(&mut self, opcode: u16);

    // 8xy7
    fn subn_vx_vy(&mut self, opcode: u16);

    // 8xyE
    fn shl_vx_vy(&mut self, opcode: u16);

    // 9xy0
    fn sne_vx_vy(&mut self, opcode: u16);

    // Annn
    fn ld_i_addr(&mut self, opcode: u16);

    // Bnnn
    fn jp_v0_addr(&mut self, opcode: u16);

    // Cxkk
    fn rnd_vx_kk(&mut self, opcode: u16);

    // Dxyn
    fn drw_vx_vy_nibble(&mut self, opcode: u16);

    // Ex9E
    fn skp_vx(&mut self, opcode: u16);

    // ExA1
    fn sknp_vx(&mut self, opcode: u16);

    // Fx07
    fn ld_vx_dt(&mut self, opcode: u16);

    // Fx0A
    fn ld_vx_k(&mut self, opcode: u16) -> u8;

    // Fx15
    fn ld_dt_vx(&mut self, opcode: u16);

    // Fx18
    fn ld_st_vx(&mut self, opcode: u16);

    // Fx1E
    fn add_i_vx(&mut self, opcode: u16);

    // Fx29
    fn ld_f_vx(&mut self, opcode: u16);

    // Fx33
    fn ld_b_vx(&mut self, opcode: u16);

    // Fx55
    fn ld_addri_vx(&mut self, opcode: u16);

    // Fx65
    fn ld_vx_addri(&mut self, opcode: u16);
}

impl Instructions for Chip8 {
    // 0nnn
    fn sys_addr(&mut self) {
        todo!("Jump to a machine code routine at nnn");
    }

    // 00E0
    fn cls(&mut self) {
        self.pc += 2;
    }

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
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // 4xkk
    fn sne_vx_b(&mut self, opcode: u16) {
        let (x, kk) = get_xkk(opcode);

        if self.v[x] != kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // 5xy0
    fn se_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        if self.v[x] == self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // 6xkk
    fn ld_vx_kk(&mut self, opcode: u16) {
        let (x, kk) = get_xkk(opcode);

        self.v[x] = kk;
        self.pc += 2;
    }

    // 7xkk
    fn add_vx_kk(&mut self, opcode: u16) {
        let (x, kk) = get_xkk(opcode);

        self.v[x] += kk;
        self.pc += 2;
    }

    // 8xy0
    fn ld_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        self.v[x] = self.v[y];
        self.pc += 2;
    }

    // 8xy1
    fn or_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        self.v[x] |= self.v[y];
        self.pc += 2;
    }

    // 8xy2
    fn and_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        self.v[x] &= self.v[y];
        self.pc += 2;
    }

    // 8xy3
    fn xor_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        self.v[x] ^= self.v[y];
        self.pc += 2;
    }

    // 8xy4
    fn add_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        let (result, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = result;
        self.v[0xF] = if carry { 1 } else { 0 };
        self.pc += 2;
    }

    // 8xy5
    fn sub_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        let (result, overflow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = result;
        self.v[y] = if overflow { 0 } else { 1 };
        self.pc += 2;
    }

    // 8xy6
    fn shr_vx_vy(&mut self, opcode: u16) {
        let (x, _) = get_xy(opcode);

        // TODO Unsure
        self.v[0xF] = self.v[x] & 1;
        self.v[x] = self.v[x] >> 1;
        self.pc += 2;
    }

    // 8xy7
    fn subn_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        let (result, overflow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = result;
        self.v[y] = if overflow { 1 } else { 0 };
        self.pc += 2;
    }

    // 8xyE
    fn shl_vx_vy(&mut self, opcode: u16) {
        let (x, _) = get_xy(opcode);

        // TODO Unsure
        self.v[0xF] = self.v[x] & 0b1000_0000;
        self.v[x] = self.v[x] << 1;
        self.pc += 2;
    }

    // 9xy0
    fn sne_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        if self.v[x] != self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Annn
    fn ld_i_addr(&mut self, opcode: u16) {
        let addr = opcode & ADDR_MASK;

        self.i = addr;
        self.pc += 2;
    }

    // Bnnn
    fn jp_v0_addr(&mut self, opcode: u16) {
        let addr = opcode & ADDR_MASK;

        self.pc = addr + self.v[0] as u16;
    }

    // Cxkk
    fn rnd_vx_kk(&mut self, opcode: u16) {
        let (x, kk) = get_xkk(opcode);

        let rnd: u8 = rand::thread_rng().gen();

        self.v[x] = rnd & kk;
        self.pc += 2;
    }

    // Dxyn
    fn drw_vx_vy_nibble(&mut self, opcode: u16) {
        // TODO Not very pretty
        // TODO Wrap pixels around
        let (x, y, n) = get_xyn(opcode);
        let pos_x = self.v[x] as usize;
        let pos_y = self.v[y] as usize;
        let bytes = self.mem.read_bytes(self.i, n);

        bytes.iter().enumerate().for_each(|(idx_y, byte)| {
            let bits = byte_to_bit_array(*byte);

            for (idx_x, bit) in bits.iter().enumerate() {
                if pos_x + idx_x < WIDTH && pos_y + idx_y < HEIGHT {
                    let previous_value = self.gfx[pos_x + idx_x][pos_y + idx_y];

                    self.v[0xF] = if previous_value == *bit && *bit == true {
                        1
                    } else {
                        0
                    };

                    self.gfx[pos_x + idx_x][pos_y + idx_y] ^= *bit;
                }
            }
        });
        self.pc += 2;
    }

    // Ex9E
    fn skp_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        if self.keys[x as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // ExA1
    fn sknp_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        if !self.keys[x as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Fx07
    fn ld_vx_dt(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        self.v[x as usize] = self.delay_timer;
        self.pc += 2;
    }

    // Fx0A
    fn ld_vx_k(&mut self, opcode: u16) -> u8 {
        let (x, _) = get_xkk(opcode);
        self.pc += 2;
        x as u8
    }

    // Fx15
    fn ld_dt_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        self.delay_timer = self.v[x as usize];
        self.pc += 2;
    }

    // Fx18
    fn ld_st_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        self.sound_timer = self.v[x as usize];
        self.pc += 2;
    }

    // Fx1E
    fn add_i_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        self.i += self.v[x as usize] as u16;
        self.pc += 2;
    }

    // Fx29
    fn ld_f_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);
        let font = self.v[x] & 0xF;

        let addr = self.mem.get_font_address(font);
        self.i = addr;
        self.pc += 2;
    }

    // Fx33
    fn ld_b_vx(&mut self, opcode: u16) {
        // todo!("Store BCD representation");
        self.pc += 2;
    }

    // Fx55
    fn ld_addri_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        for i in 0..=x {
            let addr = self.i + i as u16;
            self.mem.set(addr, self.v[i]);
        }
        self.pc += 2;
    }

    // Fx65
    fn ld_vx_addri(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        for i in 0..=x {
            let addr = self.i + i as u16;
            self.v[i] = self.mem.get(addr);
        }
        self.pc += 2;
    }
}

pub fn bytes_to_word(h: u8, l: u8) -> u16 {
    ((h as u16) << 8) | l as u16
}

pub fn split_into_4bits(n: u16) -> (u8, u8, u8, u8) {
    let [h, l] = n.to_be_bytes();
    (h >> 4, h & 0xF, l >> 4, l & 0xF)
}

fn get_xkk(n: u16) -> (usize, u8) {
    let x = ((n & 0x0F00) >> 8) as usize;
    let kk = (n & 0x00FF) as u8;

    (x, kk)
}

fn get_xy(n: u16) -> (usize, usize) {
    let x = ((n & 0x0F00) >> 8) as usize;
    let y = ((n & 0x00F0) >> 4) as usize;

    (x, y)
}

fn get_xyn(n: u16) -> (usize, usize, u8) {
    let (x, y) = get_xy(n);
    let nn = (n & 0xF) as u8;

    (x, y, nn)
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
