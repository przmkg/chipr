const RAM_SIZE: usize = 4096;

pub struct Mem {
    ram: [u8; RAM_SIZE],
}

impl Mem {
    pub fn new() -> Self {
        Mem { ram: [0; RAM_SIZE] }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        // TODO Optimize
        rom_data.iter().enumerate().for_each(|(i, byte)| {
            self.ram[0x200 + i] = *byte;
        });
    }

    pub fn get(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn set(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
    }

    pub fn read_bytes(&self, address: u16, n: u8) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(n as usize);

        for i in 0..=n {
            bytes.push(self.ram[i as usize]);
        }

        bytes
    }
}
