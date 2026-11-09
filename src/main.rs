use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::fs::File;
use std::time::Duration;
use std::io::{self,Read};

struct System {
    registers: [u8; 16],
    i: u16,
    pc: usize,
    memory: [u8; 4096],
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u16,
    key: [u8; 16],
}

fn load_game(sys: &mut System, path: String) -> io::Result<()> {
    let mut file = File::open(path).unwrap();
    let _ =file.read(&mut sys.memory).unwrap();
    Ok(())
}

fn init_system() -> System {
    System {
        registers: [0; 16],
        i: 0,
        pc: 16,
        memory: [0; 4096],
        gfx: [0; 64 * 32],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
        key: [0; 16],
    }
}

fn emulate_cycle(sys: &mut System){
    let pc: usize = sys.pc.clone();
    let opcode = ((sys.memory[pc] as u16) << 8 ) | sys.memory[pc] as u16;
    sys.pc += 2;
    
    match opcode & 0xF000 {
        0x1000 => {
            let nnn: u16 = opcode & (0x0FFF);
            sys.pc = nnn.into();
        }
        _ => {
            println!("unknown opcode: {opcode:#x} at pc: {pc:#x}, exiting");
            std::process::exit(1);
            }
    }
}

fn main() {
    let mut system = init_system();

    let _ = load_game(&mut system, String::into("pong.rom".to_string()));
    
    loop {
        emulate_cycle(&mut system);
    }
}
