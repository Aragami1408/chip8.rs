use rand::random;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;

#[allow(dead_code)]
pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

const START_ADDR: u16 = 0x200;

const FONTSET: [u8; FONTSET_SIZE] = [
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

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }


}

impl Emu {

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode & Execute
        self.execute(op);    
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        let nnn = op & 0x0FFF;
        let nn = op & 0x00FF;
        let n = op & 0x000F;



        match (digit1, digit2, digit3, digit4) {
            (0,0,0,0) => return, // 0x0000 - NOP - Do nothing
            (0,0,0xe,0) => { // 0x00E0 - CLS - Clear screen
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT]
            },
            (0,0,0xe,0xe) => { // 0x00EE - RET - Return from Subroutine
                let ret_addr = self.stack_pop();
                self.pc = ret_addr;
            },
            (0x1,_, _, _) => { // 0x1NNN - JMP NNN - Jump to address NNN
                self.pc = nnn;
            },
            (0x2,_,_,_) => { // 0x2NNN - CALL NNN - Call Subroutine
                self.stack_push(self.pc);
                self.pc = nnn;
            },
            (0x3,_,_,_) => { // 0x3XNN - SKIP VX == NN - Skip next if VX == NN
                let x = digit2 as usize;
                if self.v_reg[x] == (nn as u8) {
                    self.pc += 2;
                }
            },
            (0x4,_,_,_) => { // 0x4XNN - SKIP VX != NN - Skip next if VX != NN
                let x = digit2 as usize;
                if self.v_reg[x] != (nn as u8) {
                    self.pc += 2;
                }
            },
            (0x5,_,_,0) => { // 0x5XY0 - SKIP VX == VY - Skip next if VX == VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },
            (0x6,_,_,_) => { // 0x6XNN - VX = NN
                let x = digit2 as usize;
                self.v_reg[x] = nn as u8;
            },
            (0x7,_,_,_) => { // 0x7XNN - VX += NN
                let x = digit2 as usize;
                self.v_reg[x] += self.v_reg[x].wrapping_add(nn as u8);
            },
            (0x8,_,_,0) => { // 0x8XY0 - VX = VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },
            (0x8,_,_,1) => { // 0x8XY1 - VX |= VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },
            (0x8,_,_,2) => { // 0x8XY2 - VX &= VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            },
            (0x8,_,_,3) => { // 0x8XY3 - VX ^= VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },
            (0x8,_,_,4) => { // 0x8XY4 - VX += VY
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry {1} else {0};

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;

            },
            (0x8,_,_,5) => { // 0x8XY4 - VX -= VY
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow {0} else {1};

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (0x8,_,_,6) => { // 0x8XY6 - VX >>= 1
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xf] = lsb;
            },
            (0x8,_,_,7) => { // 0x8XY7 - VX = VY - VX
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow {0} else {1};

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (0x8,_,_,0xE) => { // 0x8XYE - VX <<= 1
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xf] = msb;
            },
            (0x9,_,_,0) => { // 0x9XY0 - SKIP VX != VY - Skip next if VX != VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },
            (0xA,_,_,_) => { // 0xANNN - I = NNN
                self.i_reg = nnn;
            },
            (0xB,_,_,_) => { // 0xBNNN - JMP V0 + NNN
                self.pc = (self.v_reg[0] as u16) + nnn;
            },
            (0xC,_,_,_) => { // 0xCXNN - VX = rand() & NN
                let x = digit2 as usize;
                let rng: u8 = random(); 
                self.v_reg[x] = rng * (nn as u8);
            },
            (0xD,_,_,_) => { // 0xDXYN - DRAW - Draw Sprite
                // Get the (x,y) coords for our sprite
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit2 as usize] as u16;
                // The last digit determines how many rows high our sprite is
                let num_rows = digit4;
                // Keep track if any pixels were flipped
                let mut flipped = false;
                // Iterate over each row of our sprite
                for y_line in 0..num_rows {
                    // Determine which memory address our row's data is stored
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    // Iterate over each column in our row
                    for x_line in 0..8 {

                    }
                }
            },
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),

        }
    }



}

impl Emu {

    fn stack_push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn stack_pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}
