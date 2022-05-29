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
        // Ignore...
    }

    // 00E0
    fn cls(&mut self) {
        self.pc += 2;
    }

    // 00EE
    fn ret(&mut self) {
        if let Some(value) = self.stack.pop() {
            self.pc = value;
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

        // TODO Should it wrap ?
        let (res, overflow) = self.v[x].overflowing_add(kk);
        self.v[x] = res;
        self.v[0xF] = if overflow { 1 } else { 0 };

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

        let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = result;
        self.v[0xF] = if overflow { 1 } else { 0 };

        self.pc += 2;
    }

    // 8xy5
    fn sub_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        let (result, overflow) = self.v[x].overflowing_sub(self.v[y]);

        self.v[x] = result;
        self.v[0xF] = if overflow { 1 } else { 0 };

        self.pc += 2;
    }

    // 8xy6
    fn shr_vx_vy(&mut self, opcode: u16) {
        let (x, _) = get_xy(opcode);

        self.v[0xF] = self.v[x] & 1;
        let (result, _) = self.v[x].overflowing_shr(1);
        self.v[x] = result;

        self.pc += 2;
    }

    // 8xy7
    fn subn_vx_vy(&mut self, opcode: u16) {
        let (x, y) = get_xy(opcode);

        let (result, overflow) = self.v[y].overflowing_sub(self.v[x]);

        self.v[x] = result;
        self.v[0xF] = if overflow { 0 } else { 1 };

        self.pc += 2;
    }

    // 8xyE
    fn shl_vx_vy(&mut self, opcode: u16) {
        let (x, _) = get_xy(opcode);

        self.v[0xF] = self.v[x] & 0x80;
        let (result, _) = self.v[x].overflowing_shl(1);
        self.v[x] = result;
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
        let (rx, ry, n) = get_xyn(opcode);
        let data = self.mem.read_bytes(self.i, n);

        self.v[0xF] = 0;
        data.iter().enumerate().for_each(|(j, byte)| {
            let bits = byte_to_bit_array(*byte);

            for i in 0..bits.len() {
                let x = (self.v[rx] as usize + i) % WIDTH;
                let y = (self.v[ry] as usize + j) % HEIGHT;

                let position = y * WIDTH + x;

                let previous_value = self.gfx[position];

                if previous_value && previous_value == bits[i] {
                    self.v[0xF] = 1;
                }

                self.gfx[position] ^= bits[i];
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

        // TODO Set overflow like on the Amiga implementation
        self.i += self.v[x as usize] as u16;
        self.pc += 2;
    }

    // Fx29
    fn ld_f_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);
        let font = self.v[x] & 0xF;

        let addr = self.mem.get_font_address(font);
        println!("{}", addr);
        self.i = addr;
        self.pc += 2;
    }

    // Fx33
    fn ld_b_vx(&mut self, opcode: u16) {
        let (x, _) = get_xkk(opcode);

        let addr = self.i;
        let value = self.v[x];

        // TODO Unsure
        self.mem.set(addr, value / 100);
        self.mem.set(addr + 1, (value / 10) % 10);
        self.mem.set(addr + 2, (value % 100) % 10);

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
        (b & 0x80) == 0x80,
        (b & 0x40) == 0x40,
        (b & 0x20) == 0x20,
        (b & 0x10) == 0x10,
        (b & 0x08) == 0x08,
        (b & 0x04) == 0x04,
        (b & 0x02) == 0x02,
        (b & 0x01) == 0x01,
    ]
}

#[cfg(test)]
mod tests {
    use crate::instr::byte_to_bit_array;

    #[test]
    fn test_byte_to_bit_array() {
        assert_eq!(
            byte_to_bit_array(0b1100_0001),
            [true, true, false, false, false, false, false, true]
        );
    }
}
