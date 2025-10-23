use std::array;
use rand::{prelude::*, random_range};

const SPRITES: [u8; 80] = [
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

fn main() {
    println!("Hello, world!");
}

struct CPU {
    v: [u8; 16],
    delay: u8,
    sound: u8,
    program_counter: u16,
    stack_pointer: u8,
    stack: [u16; 16],
    i: u16
}

impl CPU {
    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("RET called on empty stack")
        } else {
            self.stack_pointer -= 1;
            self.program_counter = self.stack[self.stack_pointer as usize];
        }
    }
    // Think about adding a check to ensure nnn is 12 bits
    fn jp_addr(&mut self, nnn: u16) {
        self.program_counter = nnn;
    }

    // 2nnn
    fn call_addr(&mut self, nnn: u16) {        
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = nnn;
    }

    // 3xkk
    fn se_vx_byte(&mut self, kk: u8, x: u8) {
        if self.v[x as usize] == kk {
            self.program_counter += 2;
        }
    }

    // 4xkk
    fn sne_vx_byte(&mut self, kk: u8, x: u8) {
       if self.v[x as usize] != kk {
            self.program_counter += 2;
        } 
    }

    // 5xy0
    fn se_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[x as usize] == self.v[y as usize] {
            self.program_counter += 2;
        }
    }

    // 6xkk
    fn ld_vx_byte(&mut self, x: u8, kk: u8) {
        self.v[x as usize] = kk;
    }

    // 7xkk
    fn add_vx_byte(&mut self, x: u8, kk: u8) {
        self.v[x as usize] += kk;
    }

    // 8xy0
    fn ld_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[y as usize];
    }

    // 8xy1
    fn or_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] |= self.v[y as usize];
    }

    // 8xy2
    fn and_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] &= self.v[y as usize];
    }

    // 8xy3
    fn xor_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] ^= self.v[y as usize];
    }

    // 8xy4
    fn add_vx_vy(&mut self, x: u8, y: u8) {
        let mut a = self.v[x as usize];
        let mut b = self.v[y as usize];
        match a.checked_add(b) {
            Some(sum) => {
                self.v[x as usize] = sum;
                self.v[15] = 0;
            },
            None => {
                self.v[x as usize] += self.v[y as usize];
                self.v[15] = 1;
            }
        }
    }

    // 8xy5
    fn sub_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[x as usize] > self.v[y as usize] {
           self.v[15] = 1; 
        } else {
            self.v[15] = 0;
        }

        self.v[x as usize] -= self.v[y as usize]; 
    }

    // 8xy6
    fn shr_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[x as usize] % 2 == 0 {
           self.v[15] = 0;
        } else {
           self.v[15] = 1; 
        }

        self.v[x as usize] /= 2;
    }

    // 8xy7
    fn subn_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[y as usize] > self.v[x as usize] {
           self.v[15] = 1; 
        } else {
            self.v[15] = 0;
        }

        self.v[x as usize] = self.v[y as usize] - self.v[x as usize];
    }

    // 8xyE
    fn shl_vx_vy(&mut self, x: u8) {
        let msb = (self.v[x as usize] >> 7) & 1;

        if msb == 1 {
           self.v[15] = 1; 
        } else {
           self.v[15] = 0; 
        }

        self.v[x as usize] *= 2;
    }

    // 9xy0
    fn sne_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[y as usize] != self.v[x as usize] {
            self.program_counter += 2;
        }
    }

    // Annn
    fn ld_i_addr(&mut self, nnn: u16) {
        self.i = nnn;
    }

    // Bnnn
    fn jp_v0_addr(&mut self, nnn: u16) {
        let v0 = self.v[0] as u16;
        self.program_counter = nnn + v0;
    }

    // Cxkk
    fn rnd_vx_byte(&mut self, x: u8, kk: u8) {
        let rand_byte: u8 = random_range(0..=255);
        self.v[x as usize] = rand_byte & kk;
    }

    fn ld_vx_dt(&mut self, x: u8) {
        self.v[x as usize] = self.delay;
    }

    fn ld_dt_vx(&mut self, x: u8) {
        self.delay = self.v[x as usize];
    }

    fn ld_st_vx(&mut self, x: u8) {
        self.sound = self.v[x as usize];
    }

    fn add_i_vx(&mut self, x: u8) {
        let vx = self.v[x as usize] as u16; 
        self.i += vx;
    }

    fn ld_f_vx(&mut self, x: u8) {
        self.i = (self.v[x as usize] * 5) as u16;
    }
}

struct keyboard {
    keys: [bool; 16]
}

struct Chip8 {
    memory: [u8; 4096],
    cpu: CPU,
    keyboard: keyboard,
    display: [u8; 64 * 32],
    // timers: timers,
    // sound: sound
}

impl Chip8 {
    // 0x00E0
    fn cls(&mut self) {
        self.display.fill(0);
    }

    // Ex9E
    fn skp_vx(&mut self, x: u8) {
        if self.keyboard.keys[self.cpu.v[x as usize] as usize] == true {
            self.cpu.program_counter += 2;
        }
    }

    // ExA1
    fn sknp_vx(&mut self, x: u8) {
        if self.keyboard.keys[self.cpu.v[x as usize] as usize] == false {
            self.cpu.program_counter += 2;
        }
    }

    fn ld_vx_k(&mut self, x: u8) {
        let original = self.keyboard.keys;

        let mut i = 0;
        while i < self.keyboard.keys.len() {
            if self.keyboard.keys[i] != original[i] {
                self.cpu.v[x as usize] = i as u8;
                return;
            }
            i += 1;
        }

        self.cpu.program_counter -= 2;
    }

    fn ld_i_vx(&mut self, x: u8) {
        let mut index = 0;
        let mut addr = self.cpu.i; 

        while index <= x {
            self.memory[addr as usize] = self.cpu.v[index as usize];
            index += 1;
            addr += 1;
        }
    }

    fn ld_vx_i(&mut self, x: u8) {
        let mut index = 0;
        let mut addr = self.cpu.i;

        while index <= x {
            self.cpu.v[index as usize] = self.memory[addr as usize];
            index += 1;
            addr += 1;
        } 
    }

    fn ld_b_vx(&mut self, x: u8) {
        let three_digits = self.cpu.v[x as usize].to_string();
        let three_digit_vec: Vec<char> = three_digits.chars().collect();

        self.memory[self.cpu.i as usize] = three_digit_vec[0] as u8;
        self.memory[self.cpu.i as usize + 1] = three_digit_vec[1] as u8;
        self.memory[self.cpu.i as usize + 2] = three_digit_vec[2] as u8;
    }

    // Dxyn
    fn drw_vx_vy_nibble(&mut self, x: u8, y: u8, n: u8) {
        self.cpu.v[15] = 0;

        for index in 0..n {
            let mut sprite = self.memory[(self.cpu.i + (index as u16)) as usize];
            let row = (self.cpu.v[y as usize] + index) % 32;

            for index in 0..8 {
                let b = (sprite & 0x80) >> 7;
                let col = (self.cpu.v[x as usize] + index) % 64;
                let offset = row * 64 + col;

                if b == 1 {
                    if self.display[offset as usize] != 0 {
                        self.display[offset as usize] = 0;
                        self.cpu.v[15] = 1;
                    } else {
                        self.display[offset as usize] = 1;
                    }
                }
                sprite <<= 1;
            }
        }
    }

    // fetch
    fn fetch(&mut self) {
        let first_half = self.memory[self.cpu.program_counter as usize];
        let second_half = self.memory[self.cpu.program_counter as usize + 1];

        let string_instruction = format!("{}{}", first_half, second_half);
        let instruction: u16 = string_instruction.parse().unwrap();

        match instruction & 0xF000 {
            0x0000 => {
                if (instruction & 0xFF) as u8 == 0xE0 {
                    self.cls();
                } else {
                    self.cpu.ret();
                }
            },
            0x1000 => {
                let nnn = instruction & 0xFFF;
                self.cpu.jp_addr(nnn);
            },
            0x2000 => {
                let nnn = instruction & 0xFFF;
                self.cpu.call_addr(nnn);
            },
            0x3000 => {
                let x: u8 = ((instruction & 0x0F00) >> 8) as u8;
                let kk: u8 = (instruction & 0x00FF) as u8;

                self.cpu.se_vx_byte(kk, x);
            },
            0x4000 => {
                let x: u8 = ((instruction & 0x0F00) >> 8) as u8;
                let kk: u8 = (instruction & 0x00FF) as u8;

                self.cpu.sne_vx_byte(kk, x);
            },
            0x5000 => {
                let x: u8 = ((instruction & 0x0F00) >> 8) as u8;
                let y: u8 = ((instruction & 0x00F0) >> 4) as u8;

                self.cpu.se_vx_vy(x, y);
            },
            0x6000 => {
                let x: u8 = ((instruction & 0x0F00) >> 8) as u8;
                let kk: u8 = (instruction & 0x00FF) as u8;

                self.cpu.ld_vx_byte(x, kk);
            },
            0x7000 => {
                let x: u8 = ((instruction & 0x0F00) >> 8) as u8;
                let kk: u8 = (instruction & 0x00FF) as u8;

                self.cpu.add_vx_byte(x, kk);
            },
            0x8000 => {
                let x: u8 = ((instruction & 0x0F00) >> 8) as u8;
                let y: u8 = ((instruction & 0x00F0) >> 4) as u8;
                match instruction & 0x000F {
                    0 => {self.cpu.ld_vx_vy(x, y);},
                    1 => {self.cpu.or_vx_vy(x, y);},
                    2 => {self.cpu.and_vx_vy(x, y);},
                    3 => {self.cpu.xor_vx_vy(x, y);},
                    4 => {self.cpu.add_vx_vy(x, y);},
                    5 => {self.cpu.sub_vx_vy(x, y);},
                    6 => {self.cpu.shr_vx_vy(x, y);},
                    7 => {self.cpu.subn_vx_vy(x, y);},
                    0xE => {self.cpu.shl_vx_vy(x);}, 
                    _ => {panic!("Not an opcode")}
                }
            },
            0x9000 => {
                let x: u8 = ((instruction & 0x0F00) >> 8) as u8;
                let y: u8 = ((instruction & 0x00F0) >> 4) as u8; 

                self.cpu.sne_vx_vy(x, y);
            },
            0xA000 => {
                let nnn = instruction & 0xFFF;

                self.cpu.ld_i_addr(nnn);
            },
            0xB000 => {
                let nnn = instruction & 0xFFF;

                self.cpu.jp_v0_addr(nnn);
            },
            0xC000 => {
                let x: u8 = ((instruction & 0x0F00) >> 8) as u8;
                let kk: u8 = (instruction & 0x00FF) as u8;

                self.cpu.rnd_vx_byte(x, kk);
            },
            0xD000 => {
                let x: u8 = ((instruction & 0x0F00) >> 8) as u8;
                let y: u8 = ((instruction & 0x00F0) >> 4) as u8; 
                let n: u8 = (instruction & 0x000F) as u8;

                self.drw_vx_vy_nibble(x, y, n);
            },
            0xE000 => {
                let x: u8 = ((instruction & 0x0F00) >> 8) as u8;
                match instruction & 0x00FF {
                    0x9E => {self.skp_vx(x);},
                    0xA1 => {self.sknp_vx(x);}
                    _ => {panic!("Not an opcode")}
                }
            },
            0xF000 => {
                
            }

        }
        self.cpu.program_counter += 2;
    }
}