use std::io::prelude::*;
use std::{fs::File, io::BufReader};

use chip8::Chip8;
use mem::Mem;

mod audio;
mod chip8;
mod instr;
mod mem;

fn main() -> std::io::Result<()> {
    let file = File::open("roms/MAZE")?;
    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();
    buf_reader.read_to_end(&mut buffer)?;

    let mut mem = Mem::new();
    mem.load_rom(buffer);

    let mut chip8 = Chip8::new(mem);

    chip8.main_loop();

    Ok(())
}
