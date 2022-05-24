use std::io::prelude::*;
use std::{fs::File, io::BufReader};

use chip8::Chip8;
use mem::Mem;

mod audio;
mod chip8;
mod instr;
mod mem;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Missing path to rom.\nUsage: chipr rom_path");
        std::process::exit(-1);
    }

    let file = File::open(args.get(1).unwrap())?;
    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();
    buf_reader.read_to_end(&mut buffer)?;

    let mut mem = Mem::new();
    mem.load_rom(buffer);

    let mut chip8 = Chip8::new(mem);

    chip8.main_loop();

    Ok(())
}
