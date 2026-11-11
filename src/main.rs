use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::fs::File;
use std::io::{self, Read};
use std::time::Duration;

struct System {
    registers: [u8; 16],
    i: usize,
    pc: usize,
    memory: [u8; 4096],
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: usize,
    key: [u8; 16],
}

fn load_game(sys: &mut System, path: String) -> io::Result<()> {
    let mut buf: [u8;4096-512] = [0;4096-512];
    let mut file = File::open(path).unwrap();
    let n = file.read(&mut sys.memory[512..]).unwrap();
    Ok(())
}

fn init_system() -> System {
    System {
        registers: [0; 16],
        i: 0,
        pc: 0x200,
        memory: [0; 4096],
        gfx: [0; 64 * 32],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
        key: [0; 16],
    }
}

fn emulate_cycle(sys: &mut System) {
    let pc: usize = sys.pc.clone();
    let opcode: u16 = ((sys.memory[pc] as u16) << 8) | sys.memory[pc + 1] as u16;
    println!("Opcode: {opcode:#x} at pc: {pc:#x}");
    match opcode & 0xF000 {
        0x0000 => {
            let dt: u16 = opcode & 0x00FF;
            match dt {
                0x00E0 => {}
                0x00EE => {
                    
                }
                _ => {
                    println!("unknown opcode: {opcode:#x} at pc: {pc:#x}, exiting");
                    std::process::exit(1);
                }
            }
        }
        0x1000 => {
            let nnn: u16 = opcode & (0x0FFF);
            sys.pc = nnn.into();
        }
        0x2000 => {
            let nnn: u16 = opcode & (0x0FFF);
            sys.stack[sys.sp] = pc.try_into().unwrap();
            sys.sp += 1;
            sys.pc = nnn.into();
            emulate_cycle(sys);
        }
        0x3000 => {
            let x: usize = ((opcode & (0x0F00)) >> 8).try_into().unwrap();
            let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
            if sys.registers[x] == nn {
                sys.pc += 2;
            }
        }
        0x4000 => {
            let x: usize = ((opcode & (0x0F00)) >> 8).try_into().unwrap();
            let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
            if sys.registers[x] != nn {
                sys.pc += 2;
            }
        }
        0x5000 => {
            let x: usize = ((opcode & (0x0F00)) >> 8).try_into().unwrap();
            let y: usize = ((opcode & 0x00F0) >> 4).try_into().unwrap();
            if sys.registers[x] != sys.registers[y] {
                sys.pc += 2;
            }
        }
        0x6000 => {
            let x: usize = (opcode & (0x0F00) >> 8).try_into().unwrap();
            let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
            sys.registers[x] = nn;
        }
        0x7000 => {
            let x: usize = ((opcode & (0x0F00)) >> 8).try_into().unwrap();
            let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
            sys.registers[x] = sys.registers[x].wrapping_add(nn);
        }
        0x8000 => {
            let x: usize = ((opcode & (0x0F00)) >> 8).try_into().unwrap();
            let y: usize = ((opcode & 0x00F0) >> 4).try_into().unwrap();
            let dt: u16 = opcode & 0x000F;
            match dt {
                0x0000 => {
                    sys.registers[x] = sys.registers[y].clone();
                }
                0x0001 => {
                    sys.registers[x] |= sys.registers[y].clone();
                }
                0x0002 => {
                    sys.registers[x] &= sys.registers[y].clone();
                }
                0x0003 => {
                    sys.registers[x] ^= sys.registers[y].clone();
                }
                0x0004 => {
                    if sys.registers[y] > 0xFF - sys.registers[x] {
                        sys.registers[0xF] = 1;
                    } else {
                        sys.registers[0xF] = 0;
                    }
                    sys.registers[x] = sys.registers[x].wrapping_add(sys.registers[y].clone());
                }
                0x0005 => {
                    if sys.registers[y] <= sys.registers[x] {
                        sys.registers[0xF] = 1;
                    } else {
                        sys.registers[0xF] = 0;
                    }
                    sys.registers[x] = sys.registers[x].wrapping_rem(sys.registers[y].clone());
                }
                0x0006 => {
                    sys.registers[0xF] = sys.registers[x] & 0x0001;
                    sys.registers[x] >>= 1;
                }
                0x0007 => {
                    if sys.registers[y] >= sys.registers[x] {
                        sys.registers[0xF] = 1;
                    } else {
                        sys.registers[0xF] = 0;
                    }
                    sys.registers[x] = sys.registers[y].wrapping_rem(sys.registers[x].clone());
                }
                0x000E => {
                    sys.registers[0xF] = sys.registers[x] & 0x0008;
                    sys.registers[x] <<= 1;
                }
                _ => {
                    println!("unknown opcode: {opcode:#x} at pc: {pc:#x}, exiting");
                    std::process::exit(1);
                }
            }
        }
        0x9000 => {
            let x: usize = ((opcode & (0x0F00)) >> 2).try_into().unwrap();
            let y: usize = ((opcode & 0x00F0) >> 2).try_into().unwrap();
            if sys.registers[x] != sys.registers[y] {
                sys.pc += 2;
            }
        }
        0xA000 => {
            let nnn: u16 = (opcode & 0x0FFF).try_into().unwrap();
            sys.i = nnn.into();
        }
        0xB000 => {
            let nnn: usize = (opcode & 0x0FFF).try_into().unwrap();
            let v0: usize = sys.registers[0].try_into().unwrap();
            sys.pc = v0 + nnn;
        }
        0xC000 => {
            let nnn: usize = (opcode & 0x0FFF).try_into().unwrap();
            let v0: usize = sys.registers[0].try_into().unwrap();
            sys.pc = v0 + nnn;
        }
        0xD000 => {}
        0xE000 => {}
        0xF000 => {
            let x: usize = ((opcode & 0x0F00) >> 8).try_into().unwrap();
            let dt: u16 = opcode & 0x00FF;
            match dt {
                0x0007 => {
                    sys.registers[x] = sys.delay_timer;
                }
                0x000A => {}
                0x0015 => {
                    sys.delay_timer = sys.registers[x];
                }
                0x0018 => {
                    sys.sound_timer = sys.registers[x];
                }
                0x001E => {
                    let vx: usize = sys.registers[x].try_into().unwrap();
                    sys.i += vx;
                }
                0x0029 => {}
                0x0033 => {
                    sys.memory[sys.i] = sys.registers[x] / 100;
                    sys.memory[sys.i] += (sys.registers[x] / 10) % 10;
                    sys.memory[sys.i] += (sys.registers[x] % 100) % 10;
                }
                0x0055 => {}
                0x0065 => {}
                _ => {
                    println!("Unknown opcode: {opcode:#x} at pc: {pc:#x}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            println!("unknown opcode: {opcode:#x} at pc: {pc:#x}, exiting");
            std::process::exit(1);
        }
    }
    sys.pc += 2;
}

fn main() {
    let mut system = init_system();

    let _ = load_game(&mut system, String::into("pong.rom".to_string()));

    loop {
        emulate_cycle(&mut system);
    }
}
