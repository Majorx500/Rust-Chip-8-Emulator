use rand::rng;
use sola_raylib::prelude::*;
use std::fs::File;
use std::io::{self, Read};
use std::thread;
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
    const FONT: [u8; 80] = [
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

    for i in 0..80 {
        sys.memory[i] = FONT[i];
    }
    let mut file = File::open(path).unwrap();
    file.read(&mut sys.memory[512..]).unwrap();
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
                0x00E0 => {
                    for i in 0..64 * 32 {
                        sys.gfx[i] = 0;
                    }
                    sys.pc += 2;
                }
                0x00EE => {
                    sys.pc = sys.stack[sys.sp - 1].clone().into();
                    sys.sp -= 1;
                    sys.pc += 2
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
        }
        0x3000 => {
            let x: usize = ((opcode & (0x0F00)) >> 8).try_into().unwrap();
            let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
            if sys.registers[x] == nn {
                sys.pc += 2;
            }
            sys.pc += 2;
        }
        0x4000 => {
            let x: usize = ((opcode & (0x0F00)) >> 8).try_into().unwrap();
            let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
            if sys.registers[x] != nn {
                sys.pc += 2;
            }
            sys.pc += 2;
        }
        0x5000 => {
            let x: usize = ((opcode & (0x0F00)) >> 8).try_into().unwrap();
            let y: usize = ((opcode & 0x00F0) >> 4).try_into().unwrap();
            if sys.registers[x] != sys.registers[y] {
                sys.pc += 2;
            }
            sys.pc += 2;
        }
        0x6000 => {
            let x: usize = ((opcode & 0x0F00) >> 8).try_into().unwrap();
            let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
            sys.registers[x] = nn;
            sys.pc += 2;
        }
        0x7000 => {
            let x: usize = ((opcode & (0x0F00)) >> 8).try_into().unwrap();
            let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
            sys.registers[x] = sys.registers[x].wrapping_add(nn);
            sys.pc += 2;
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
                    sys.registers[x] = sys.registers[x].wrapping_sub(sys.registers[y].clone());
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
                    sys.registers[x] = sys.registers[y].wrapping_sub(sys.registers[x].clone());
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
            sys.pc += 2;
        }
        0x9000 => {
            let x: usize = ((opcode & (0x0F00)) >> 2).try_into().unwrap();
            let y: usize = ((opcode & 0x00F0) >> 2).try_into().unwrap();
            if sys.registers[x] != sys.registers[y] {
                sys.pc += 2;
            }
            sys.pc += 2;
        }
        0xA000 => {
            let nnn: u16 = (opcode & 0x0FFF).try_into().unwrap();
            sys.i = nnn.into();
            sys.pc += 2;
        }
        0xB000 => {
            let nnn: usize = (opcode & 0x0FFF).try_into().unwrap();
            let v0: usize = sys.registers[0].try_into().unwrap();
            sys.pc = v0 + nnn;
        }
        0xC000 => {
            let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
            let x: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
            let r: u8 = rand::random();
            sys.registers[x as usize] = r & nn;
            sys.pc += 2;
        }
        0xD000 => {
            let x = sys.registers[((opcode & 0x0F00) >> 8) as usize];
            let x = x % 64;
            let y = sys.registers[((opcode & 0x00F0) >> 4) as usize];
            let y = y % 32;

            let N = (opcode & 0x000F) as u8;
            sys.registers[0xF] = 0;

            for yline in 0..N {
                let pixel = sys.memory[sys.i + yline as usize];
                for xline in 0..8 {
                    if (pixel & (0x80 >> xline)) != 0 {
                        if sys.gfx[(x + xline + (y + yline).wrapping_mul(64)) as usize] == 1 {
                            sys.registers[0xF] = 1;
                        }
                        sys.gfx[(x + xline + (y + yline).wrapping_mul(64)) as usize] ^= 1;
                    }
                }
            }

            sys.pc += 2;
        }
        0xE000 => {
            let key = sys.registers[((opcode & 0x0F00) >> 8) as usize];
            let dt = opcode & 0x00FF;

            match dt {
                0x009E => {
                    if sys.key[key as usize] == 1 {
                        sys.pc += 2;
                    }
                }
                0x00A1 => {
                    if sys.key[key as usize] == 0 {
                        sys.pc += 2;
                    }
                }
                _ => {
                    println!("unknown opcode: {opcode:#x} at pc: {pc:#x}, exiting");
                    std::process::exit(1);
                }
            }
            sys.pc += 2;
        }
        0xF000 => {
            let x: usize = ((opcode & 0x0F00) >> 8).try_into().unwrap();
            let dt: u16 = opcode & 0x00FF;
            match dt {
                0x0007 => {
                    sys.registers[x] = sys.delay_timer;
                }
                0x000A => {
                    let mut s = 0;
                    for i in 0..16 {
                        s += sys.key[i];
                    }
                    if s > 0 {
                        sys.pc += 2;
                    }
                }
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
                0x0029 => {
                    sys.i = (5 * sys.registers[x & 0xF]).into();
                }
                0x0033 => {
                    sys.memory[sys.i] = sys.registers[x] / 100;
                    sys.memory[sys.i + 1] = (sys.registers[x] / 10) % 10;
                    sys.memory[sys.i + 2] = (sys.registers[x] % 100) % 10;
                }
                0x0055 => {
                    for i in 0..x + 1 {
                        sys.memory[sys.i + i] = sys.registers[i];
                    }
                }
                0x0065 => {
                    for i in 0..x + 1 {
                        sys.registers[i] = sys.memory[sys.i + i];
                    }
                }
                _ => {
                    println!("Unknown opcode: {opcode:#x} at pc: {pc:#x}");
                    std::process::exit(1);
                }
            }
            sys.pc += 2;
        }
        _ => {
            println!("unknown opcode: {opcode:#x} at pc: {pc:#x}, exiting");
            std::process::exit(1);
        }
    }

    if sys.delay_timer > 0 {
        sys.delay_timer -= 1;
    }

    if sys.sound_timer > 0 {
        if sys.sound_timer == 1 {
            println!("SOUND");
        }
        sys.sound_timer -= 1;
    }
}

fn main() {
    let mut system = init_system();

    let _ = load_game(&mut system, String::into("pong.rom".to_string()));

    let (mut rl, thread) = sola_raylib::init().size(640, 320).title("").build();
        
    while !rl.window_should_close() {
        emulate_cycle(&mut system);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        for i in 0..(64*32){
            let mut c = Color::BLACK;
            if system.gfx[i] == 1 {
                c = Color::WHITE;
            }
            d.draw_rectangle(((i % 64) * 10).try_into().unwrap(), ((i / 64) * 10).try_into().unwrap(), 10,10,c);
        }
        thread::sleep(Duration::from_millis(2));
    }

    
}
