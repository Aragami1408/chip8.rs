use std::{fs::{File, self}, path::Path, io::Read};
use rand::{thread_rng, Rng, distributions::Uniform, ThreadRng, prelude::Distribution};

const START_ADDRESS: usize = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8;FONTSET_SIZE] = [
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
	0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];
const FONTSET_START_ADDRESS: u32 = 0x50;

const VIDEO_WIDTH: u8 = 64;
const VIDEO_HEIGHT: u8 = 32;


#[derive(Debug)]
struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8;16],
    video: [u32; 64*32],
    opcode: u16,

    rand_gen: ThreadRng,
    rand_byte: Uniform<u8>
}


impl Chip8 {
    fn new() -> Self {
        Self {
            registers: [0;16],
            memory: [0;4096],
            index: 0,
            pc: START_ADDRESS as u16,
            stack: [0;16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0;16],
            video: [0;64*32],
            opcode: 0,

            rand_gen: thread_rng(),
            rand_byte: Uniform::new(0x00,0xff),
        }

    }


    fn load_rom(&mut self, filename: &str) {
        let mut f = File::open(filename).expect("no file found");
        let metadata = fs::metadata(filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");

        for i in 0..buffer.len() {
            self.memory[START_ADDRESS + i] = buffer[i];
        }
    }



    fn cls(&mut self) {
        for elem in self.video.iter_mut() {*elem = 0;}
    }

    fn ret(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn jp_addr(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;

        self.pc = address;
    }

    fn call(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;

        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = address;

    }

    fn se_vx_kk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let kk: u8 = (self.opcode & 0x00ff) as u8;

        if self.registers[vx as usize] == kk {
            self.pc += 2;
        }
    }

    fn sne_vx_kk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let kk: u8 = (self.opcode & 0x00ff) as u8;

        if self.registers[vx as usize] != kk {
            self.pc += 2;
        }
    }

    fn se_vx_vy(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00f0) >> 4) as u8;

    
        if self.registers[vx as usize] == vy {
            self.pc += 2;
        }
    }

    fn ld_vx_kk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let kk: u8 = (self.opcode & 0x00ff) as u8;

        self.registers[vx as usize] = kk;
    }

    fn add_vx_kk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let kk: u8 = (self.opcode & 0x00ff) as u8;

        self.registers[vx as usize] += kk;

    }

    fn ld_vx_vy(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00f0) >> 4) as u8;

        self.registers[vx as usize] = self.registers[vy as usize];
    }

    fn or_vx_vy(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00f0) >> 4) as u8;

        self.registers[vx as usize] |= self.registers[vy as usize];
    }

    fn and_vx_vy(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00f0) >> 4) as u8;

        self.registers[vx as usize] &= self.registers[vy as usize];
    }

    fn xor_vx_vy(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00f0) >> 4) as u8;

        self.registers[vx as usize] ^= self.registers[vy as usize];
    }

    fn add_vx_vy(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00f0) >> 4) as u8;

        let sum: u16 = (self.registers[vx as usize] + self.registers[vy as usize]) as u16;

        self.registers[0xf] = if sum > 0xff { 1 } else { 0 };

        self.registers[vx as usize] = sum as u8 & 0xff;
    }

    fn sub_vx_vy(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00f0) >> 4) as u8;

        self.registers[0xf] = if self.registers[vx as usize] > self.registers[vy as usize] { 1 } else { 0 };

        self.registers[vx as usize] -= self.registers[vy as usize];

    }

    fn shr_vx(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        self.registers[0xf] = self.registers[vx as usize] & 0x1;
        self.registers[vx as usize] >>= 1;

    }

    fn subn_vx_vy(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00f0) >> 4) as u8;
        self.registers[0xf] = if self.registers[vx as usize] <= self.registers[vy as usize] { 1 } else { 0 };
        self.registers[vy as usize] -= self.registers[vx as usize];
    }
    
    fn shl_vx(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;

        self.registers[0xf] = (self.registers[vx as usize] & 0x80) >> 7;

        self.registers[vx as usize] <<= 1;

    }

    fn sne_vx_vy(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00f0) >> 4) as u8;

        if self.registers[vx as usize] != self.registers[vy as usize] {
            self.pc += 2;
        }
    }

    fn ld_i_addr(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;

        self.index = address;
    }

    fn jp_v0_addr(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;
        self.pc = address + self.registers[0x0] as u16;
    }

    fn rnd_vx_kk(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let kk: u8 = (self.opcode & 0x00ff) as u8;

        self.registers[vx as usize] = self.rand_byte.sample(&mut self.rand_gen) & kk;
    
    }

    fn drw_vx_vy_n(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        let vy: u8 = ((self.opcode & 0x00f0) >> 4) as u8;
        let height: u8 = (self.opcode & 0x000f) as u8;

        let x_pos: u8 = self.registers[vx as usize] % VIDEO_WIDTH;
        let y_pos: u8 = self.registers[vy as usize] % VIDEO_HEIGHT;

        self.registers[0xf] = 0;

        for row in 0..height as u16 {
            let sprite_byte: u8 = self.memory[(self.index + row) as usize];

            for col in 0..8 as u8 {
                let sprite_pixel: u8 = sprite_byte & (0x80 >> col);          
                let index: usize = ((y_pos + row as u8) * VIDEO_WIDTH + (x_pos + col as u8)) as usize;
                let screen_pixel: &mut u32 = &mut self.video[index];

                if sprite_pixel == 0xff {
                    if *screen_pixel == 0xffffffff {
                        self.registers[0xf] = 1;
                    }

                    *screen_pixel ^= 0xffffffff;
                }
            }
        }

    }

    fn skp_vx(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;

        if self.keypad[self.registers[vx as usize] as usize] == 0xff {
            self.pc += 2;
        }
    }

    fn sknp_vx(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;

        if self.keypad[self.registers[vx as usize] as usize] != 0xff {
            self.pc += 2;
        }

    }

    fn ld_vx_dt(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;

        self.registers[vx as usize] = self.delay_timer;

    }
    
    fn ld_vx_k(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        for i in 0..16 as u8 {
            if self.keypad[i as usize] == 0xff {
                self.registers[vx as usize] = i;
                break;
            }
            else {
                self.pc -= 2;
                break;
            }
        }
    }

    fn ld_dt_vx(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        self.delay_timer = self.registers[vx as usize];
    }

    fn ld_st_vx(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        self.sound_timer = self.registers[vx as usize];
    }

    fn add_i_vx(&mut self) {
        let vx: u8 = ((self.opcode & 0x0f00) >> 8) as u8;
        self.index += self.registers[vx as usize] as u16; 
    }

    fn ld_f_vx(&mut self) {
        
    }


}


fn main() {

}
