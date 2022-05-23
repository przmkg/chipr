const RAM_SIZE: usize = 4096;

pub struct Mem {
    // TODO Load the data into ram
    ram: [u8; RAM_SIZE],
}

impl Mem {
    pub fn new() -> Self {
        Mem { ram: [0; RAM_SIZE] }
    }

    pub fn get(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }
}
