use eframe::egui::{Context, Sense, Visuals};
use eframe::epaint::{Color32, Rect, Rounding, Vec2};
use eframe::{egui, App, Frame, NativeOptions};
use std::io::prelude::*;
use std::path::PathBuf;
use std::{fs::File, io::BufReader};

use chip8::{Chip8, HEIGHT, WIDTH};
use mem::{Mem, RAM_SIZE};

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
            if ui.button("Open ROM").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.rom_path = Some(path.clone());
                    self.start_chip8();
                }
            }

            if self.chip8.is_none() {
                ui.set_enabled(false);
            }

            if ui.button("Start").clicked() {
                if let Some(chip8) = &mut self.chip8 {
                    chip8.paused = false;
                }
            }

            if ui.button("Stop").clicked() {
                if let Some(chip8) = &mut self.chip8 {
                    chip8.paused = true;
                }
            }

            if ui.button("Reset ROM").clicked() {
                self.start_chip8();
            }

            if ui.button("Step 1").clicked() {
                if let Some(chip8) = &mut self.chip8 {
                    chip8.execute();
                }
            }
        });

        egui::SidePanel::right("instructions").show(ctx, |ui| {
            if let Some(chip8) = &mut self.chip8 {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for i in 0..RAM_SIZE {
                        ui.label(format!("{:#04X} -> {:#04X}", i, chip8.mem.get(i as u16)));
                    }
                });
            }
        });

        egui::TopBottomPanel::bottom("debug_panel").show(ctx, |ui| {
            if let Some(chip8) = &mut self.chip8 {
                ui.horizontal(|ui| {
                    ui.label(format!("I = {:#04X}", chip8.i));
                    ui.label(format!("PC = {:#04X}", chip8.pc));
                });

                egui::Grid::new("v_regs").striped(true).show(ui, |ui| {
                    for i in 0..16 {
                        if i != 0 && i % 4 == 0 {
                            ui.end_row();
                        }

                        ui.label(format!("V{:X}={:02X}", i, chip8.v[i]));
                    }
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Draw the game
            // TODO Probably not very efficient
            let (response, painter) = ui.allocate_painter(
                Vec2::new((WIDTH * 4) as f32, (HEIGHT * 4) as f32),
                Sense::hover(),
            );

            if let Some(chip8) = &mut self.chip8 {
                chip8.run();

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
