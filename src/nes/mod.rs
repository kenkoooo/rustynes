mod parser;
mod rom;
mod ram;
mod bus;
mod cpu;
mod ppu;
mod cpu_registers;
mod keypad;
mod renderer;
mod types;
mod helper;
mod dma;
mod apu;

pub use self::ppu::background;
pub use self::ppu::Tile;
pub use self::ppu::{SpriteWithCtx, Sprite, SpritePosition};
pub use self::keypad::*;
pub use self::renderer::*;

use self::ppu::*;
use self::rom::Rom;
use self::ram::Ram;
use self::bus::cpu_bus;
use self::dma::*;
use self::apu::*;
use nes::types::Data;

#[derive(Debug)]
pub struct Context {
    ppu: Ppu,
    program_rom: Box<Rom>,
    work_ram: Box<Ram>,
    cpu_registers: cpu_registers::Registers,
    keypad: Keypad,
    dma: Dma,
    apu: Apu,
    nmi: bool,
    renderer: Renderer,
}

pub fn reset(ctx: &mut Context) {
    let mut cpu_bus = cpu_bus::Bus::new(&ctx.program_rom,
                                        &ctx.work_ram,
                                        &mut ctx.ppu,
                                        &mut ctx.apu,
                                        &mut ctx.keypad,
                                        &mut ctx.dma);
    cpu::reset(&mut ctx.cpu_registers, &mut cpu_bus);
}

pub fn run(ctx: &mut Context, key_state: u8) {
    ctx.keypad.update(key_state);
    loop {
        let mut cycle: u16 = 0;
        if ctx.dma.should_run() {
            ctx.dma.run(&ctx.work_ram, &mut ctx.ppu);
            cycle = 514;
        } else {
            let mut cpu_bus = cpu_bus::Bus::new(&ctx.program_rom,
                                                &ctx.work_ram,
                                                &mut ctx.ppu,
                                                &mut ctx.apu,
                                                &mut ctx.keypad,
                                                &mut ctx.dma);
            cycle += cpu::run(&mut ctx.cpu_registers, &mut cpu_bus, &mut ctx.nmi) as u16;
        }
        ctx.apu.run(cycle);
        let is_ready = ctx.ppu.run((cycle * 3) as usize, &mut ctx.nmi);
        if is_ready {
            if ctx.ppu.background.0.len() != 0 {
                ctx.renderer
                    .render(&ctx.ppu.background.0, &ctx.ppu.sprites);
            }
            break;
        }
    }
}

impl Context {
    pub fn new(buf: &mut [Data]) -> Self {
        let cassette = parser::parse(buf);
        Context {
            cpu_registers: cpu_registers::Registers::new(),
            program_rom: Box::new(Rom::new(cassette.program_rom)),
            ppu: Ppu::new(cassette.character_ram,
                          PpuConfig { is_horizontal_mirror: cassette.is_horizontal_mirror }),
            work_ram: Box::new(Ram::new(vec![0; 0x0800])),
            keypad: Keypad::new(),
            dma: Dma::new(),
            apu: Apu::new(),
            nmi: false,
            renderer: Renderer::new(),
        }
    }
}
