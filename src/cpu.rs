use rand::random_range;

pub struct CPU {
    pub v: [u8; 16],
    pub delay: u8,
    pub sound: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub stack: [u16; 16],
    pub i: u16
}

impl CPU {
    pub fn ret(&mut self) {
        // if self.stack_pointer == 0 {
        //     panic!("RET called on empty stack")
        // } else {
        //     self.stack_pointer -= 1;
        //     self.program_counter = self.stack[self.stack_pointer as usize];
        // }
        let subroutine_value = self.pop();
        self.program_counter = subroutine_value;
    }
    // Think about adding a check to ensure nnn is 12 bits
    pub fn jp_addr(&mut self, nnn: u16) {
        self.program_counter = nnn;
    }

    // 2nnn
    // fn call_addr(&mut self, nnn: u16) {        
    //     self.stack[self.stack_pointer as usize] = self.program_counter;
    //     self.stack_pointer += 1;
    //     self.program_counter = nnn;
    // }
    pub fn call_addr(&mut self, nnn: u16) {
        // if self.stack_pointer as usize >= self.stack.len() {
        //     panic!("Stack overflow: tried to CALL when stack is full");
        // }

        // self.stack[self.stack_pointer as usize] = self.program_counter;
        // self.stack_pointer += 1;
        // self.program_counter = nnn;
        
        self.push(self.program_counter);
        self.program_counter = nnn;
    }


    // 3xkk
    pub fn se_vx_byte(&mut self, kk: u8, x: u8) {
        if self.v[x as usize] == kk {
            self.program_counter += 2;
        }
    }

    // 4xkk
    pub fn sne_vx_byte(&mut self, kk: u8, x: u8) {
       if self.v[x as usize] != kk {
            self.program_counter += 2;
        } 
    }

    // 5xy0
    pub fn se_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[x as usize] == self.v[y as usize] {
            self.program_counter += 2;
        }
    }

    // 6xkk
    pub fn ld_vx_byte(&mut self, x: u8, kk: u8) {
        self.v[x as usize] = kk;
    }

    // 7xkk
    // fn add_vx_byte(&mut self, x: u8, kk: u8) {
    //     self.v[x as usize] += kk;
    // }
    pub fn add_vx_byte(&mut self, x: u8, kk: u8) {
        self.v[x as usize] = self.v[x as usize].wrapping_add(kk);
    }

    // 8xy0
    pub fn ld_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[y as usize];
    }

    // 8xy1
    pub fn or_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] |= self.v[y as usize];
        self.v[0xF] = 0
    }

    // 8xy2
    pub fn and_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] &= self.v[y as usize];
        self.v[0xF] = 0
    }

    // 8xy3
    pub fn xor_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] ^= self.v[y as usize];
        self.v[0xF] = 0
    }

    // 8xy4
    pub fn add_vx_vy(&mut self, x: u8, y: u8) {
        let a = self.v[x as usize];
        let b = self.v[y as usize];
        // match a.checked_add(b) {
        //     Some(sum) => {
        //         self.v[x as usize] = sum;
        //         self.v[15] = 0;
        //     },
        //     None => {
        //         self.v[x as usize] += self.v[y as usize];
        //         self.v[15] = 1;
        //     }
        // }
        let (sum, carry) = a.overflowing_add(b);
        self.v[x as usize] = sum;
        self.v[0xF] = if carry {1} else {0}
    }

    // 8xy5
    pub fn sub_vx_vy(&mut self, x: u8, y: u8) {
        // if self.v[x as usize] > self.v[y as usize] {
        //    self.v[15] = 1; 
        // } else {
        //     self.v[15] = 0;
        // }

        // self.v[x as usize] = self.v[x as usize].wrapping_sub(self.v[y as usize]);

        let (current_x, overflow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
        let new_vf = if overflow {0} else {1};
        self.v[x as usize] = current_x;
        self.v[0xF] = new_vf;
    }

    // 8xy6
    pub fn shr_vx_vy(&mut self, x: u8, y: u8) {
        let lsb = self.v[y as usize] & 0b0000_0001;
        // self.v[0xF] = lsb;
        self.v[x as usize] = self.v[y as usize] >> 1;
        self.v[0xF] = lsb;
    }

    // 8xy7
    pub fn subn_vx_vy(&mut self, x: u8, y: u8) {
        // if self.v[y as usize] > self.v[x as usize] {
        //    self.v[15] = 1; 
        // } else {
        //     self.v[15] = 0;
        // }

        // self.v[x as usize] = self.v[y as usize].wrapping_sub(self.v[x as usize]);
        let (current_x, overflow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
        let new_vf = if overflow {0} else {1};
        self.v[x as usize] = current_x;
        self.v[0xF] = new_vf;
    }

    // 8xyE
    pub fn shl_vx_vy(&mut self, x: u8, y: u8) {
        let msb = (self.v[y as usize] >> 7) & 1;

        // if msb == 1 {
        //    self.v[15] = 1; 
        // } else {
        //    self.v[15] = 0; 
        // }

        // self.v[x as usize] *= 2;
        // self.v[0xF] = msb;
        self.v[x as usize] = self.v[y as usize] << 1;
        self.v[0xF] = msb;
    }

    // 9xy0
    pub fn sne_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[y as usize] != self.v[x as usize] {
            self.program_counter += 2;
        }
    }

    // Annn
    pub fn ld_i_addr(&mut self, nnn: u16) {
        self.i = nnn;
    }

    // Bnnn
    pub fn jp_v0_addr(&mut self, nnn: u16) {
        // let twelve_bit = nnn & 0b0000_1111_1111_1111;
        // let shifted = twelve_bit >> 8;
        // let x = shifted & 0b1111;

        // let vx = self.v[x as usize];
        // self.program_counter = nnn + vx as u16;
        let v0 = self.v[0] as u16;
        self.program_counter = v0 + nnn;
    }

    // Cxkk
    pub fn rnd_vx_byte(&mut self, x: u8, kk: u8) {
        let rand_byte: u8 = random_range(0..=255);
        self.v[x as usize] = rand_byte & kk;
    }

    // Fx07
    pub fn ld_vx_dt(&mut self, x: u8) {
        self.v[x as usize] = self.delay;
    }

    // Fx15
    pub fn ld_dt_vx(&mut self, x: u8) {
        println!("setting delay timer");
        self.delay = self.v[x as usize];
        println!("x: {}", x);
        println!("delay timer: {}", self.delay);
    }

    // Fx18
    pub fn ld_st_vx(&mut self, x: u8) {
        self.sound = self.v[x as usize];
    }

    // Fx1E
    pub fn add_i_vx(&mut self, x: u8) {
        let vx = self.v[x as usize] as u16; 
        self.i += vx;
    }

    // Fx29
    // fn ld_f_vx(&mut self, x: u8) {
    //     self.i = (self.v[x as usize] * 5) as u16;
    // }
    pub fn ld_f_vx(&mut self, x: u8) {
        self.i = (self.v[x as usize] as u16) * 5;
    }

    pub fn push(&mut self, instuction: u16) {
        self.stack[self.stack_pointer as usize] = instuction;
        self.stack_pointer += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }
}