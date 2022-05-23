use chip8::Chip8;

mod audio;
mod chip8;
mod mem;

fn main() {
    let mut chip8 = Chip8::new();

    chip8.main_loop();
}
