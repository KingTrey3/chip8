use std::array;
use std::rand::{task_rng, Rng};

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
        let rand_byte: u8 = task_rng().gen_range(0, 255);
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
    // display: display,
    // timers: timers,
    // sound: sound
}

