const RAM_SIZE: usize = 4096;

pub struct Mem {
    ram: [u8; RAM_SIZE],
}

impl Mem {
    pub fn new() -> Self {
        let fonts: [u8; 80] = [
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

        let mut ram: [u8; RAM_SIZE] = [0; RAM_SIZE];

        for i in 0..80 {
            ram[i + 0x50] = fonts[i];
        }

        Mem { ram }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        rom_data.iter().enumerate().for_each(|(i, byte)| {
            self.ram[0x200 + i] = *byte;
        });
    }

    pub fn get(&self, address: u16) -> u8 {
        if address < 0x200 {
            println!("GET ADDR: {:04X}", address);
        }
        self.ram[address as usize]
    }

    pub fn set(&mut self, address: u16, value: u8) {
        if address < 0x200 {
            println!("SET ADDR: {:04X}", address);
        }
        self.ram[address as usize] = value;
    }

    pub fn read_bytes(&self, address: u16, n: u8) -> &[u8] {
        if address < 0x200 {
            println!("RDD ADDR: {:04X}", address);
        }
        &self.ram[address as usize..address as usize + n as usize]
    }

    pub fn get_font_address(&self, font: u8) -> u16 {
        font as u16 * 5
    }
}
