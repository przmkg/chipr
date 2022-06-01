use eframe::egui::{Context, Sense, Visuals};
use eframe::emath::pos2;
use eframe::epaint::{Color32, Pos2, Rect, Rounding, Vec2};
use eframe::{egui, App, Frame, NativeOptions};
use std::io::prelude::*;
use std::path::PathBuf;
use std::{fs::File, io::BufReader};

use chip8::{Chip8, HEIGHT, WIDTH};
use mem::Mem;

mod chip8;
mod instr;
mod mem;

fn main() -> std::io::Result<()> {
    eframe::run_native(
        "Chipr",
        NativeOptions::default(),
        Box::new(|cc| Box::new(Chip8Emu::new(cc))),
    );
}

struct Chip8Emu {
    rom_path: Option<PathBuf>,
    chip8: Option<Chip8>,
}

impl Chip8Emu {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());

        Self {
            rom_path: None,
            chip8: None,
        }
    }

    fn start_chip8(&mut self) {
        let path = self.rom_path.clone().unwrap();
        let file = File::open(path).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut buffer = Vec::new();
        buf_reader.read_to_end(&mut buffer).unwrap();

        let mut mem = Mem::new();
        mem.load_rom(buffer);

        self.chip8 = Some(Chip8::new(mem));
    }
}

impl App for Chip8Emu {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if ctx.input().key_pressed(egui::Key::Escape) {
            frame.quit();
        }

        egui::SidePanel::left("load_panel").show(ctx, |ui| {
            ui.heading("Menu");

            if ui.button("Open ROM").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.rom_path = Some(path.clone());
                    self.start_chip8();
                }
            }

            if ui.button("Reset ROM").clicked() {
                println!("PATH: {:?}", self.rom_path);
                if self.rom_path.is_some() {
                    self.start_chip8();
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Draw the game
            let (response, painter) = ui.allocate_painter(
                Vec2::new((WIDTH * 4) as f32, (HEIGHT * 4) as f32),
                Sense::hover(),
            );

            if let Some(chip8) = &mut self.chip8 {
                chip8.execute();

                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        if chip8.gfx[y * WIDTH + x] {
                            painter.rect_filled(
                                Rect::from_min_size(
                                    response.rect.left_top()
                                        + Vec2::new((x * 4) as f32, (y * 4) as f32),
                                    Vec2::splat(4.0),
                                ),
                                Rounding::none(),
                                Color32::WHITE,
                            );
                        }
                    }
                }
            }
        });
    }
}
