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

    fn call_addr(&mut self, nnn: u16) {        
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = nnn;
    }

    fn se_vx_byte(&mut self, kk: u8, x: u8) {
        if self.v[x as usize] == kk {
            self.program_counter += 2;
        }
    }

    fn sne_vx_byte(&mut self, kk: u8, x: u8) {
       if self.v[x as usize] != kk {
            self.program_counter += 2;
        } 
    }

    fn se_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[x as usize] == self.v[y as usize] {
            self.program_counter += 2;
        }
    }

    fn ld_vx_byte(&mut self, x: u8, kk: u8) {
        self.v[x as usize] = kk;
    }

    fn add_vx_byte(&mut self, x: u8, kk: u8) {
        self.v[x as usize] += kk;
    }

    fn ld_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[y as usize];
    }

    fn or_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] |= self.v[y as usize];
    }

    fn and_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] &= self.v[y as usize];
    }

    fn xor_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] ^= self.v[y as usize];
    }

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

    fn sub_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[x as usize] > self.v[y as usize] {
           self.v[15] = 1; 
        } else {
            self.v[15] = 0;
        }

        self.v[x as usize] -= self.v[y as usize]; 
    }

    fn shr_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[x as usize] % 2 == 0 {
           self.v[15] = 0;
        } else {
           self.v[15] = 1; 
        }

        self.v[x as usize] /= 2;
    }

    fn subn_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[y as usize] > self.v[x as usize] {
           self.v[15] = 1; 
        } else {
            self.v[15] = 0;
        }

        self.v[x as usize] = self.v[y as usize] - self.v[x as usize];
    }

    fn shl_vx_vy(&mut self, x: u8) {
        let msb = (self.v[x as usize] >> 7) & 1;

        if msb == 1 {
           self.v[15] = 1; 
        } else {
           self.v[15] = 0; 
        }

        self.v[x as usize] *= 2;
    }

    fn sne_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[y as usize] != self.v[x as usize] {
            self.program_counter += 2;
        }
    }

    fn ld_i_addr(&mut self, nnn: u16) {
        self.i = nnn;
    }

    fn jp_v0_addr(&mut self, nnn: u16) {
        let v0 = self.v[0] as u16;
        self.program_counter = nnn + v0;
    }

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
    fn cls(&mut self) {
        self.display.fill(0);
    }

    fn skp_vx(&mut self, x: u8) {
        if self.keyboard.keys[self.cpu.v[x as usize] as usize] == true {
            self.cpu.program_counter += 2;
        }
    }

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
}