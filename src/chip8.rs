use crate::{cpu::CPU};

pub struct Chip8 {
    pub memory: [u8; 4096],
    pub cpu: CPU,
    pub keyboard: Keyboard,
    pub display: [u8; 64 * 32],
    pub draw_flag: bool,
    pub waiting_for_key: bool,
    pub wait_for_release_key: u8
    // timers: timers,
    // sound: sound
}

pub struct Keyboard {
    pub keys: [bool; 16]
}

impl Chip8 {
    // 0x00E0
    fn cls(&mut self) {
        self.display.fill(0);
        self.draw_flag = true;
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

    // Fx0A
    // fn ld_vx_k(&mut self, x: u8) {
    //     let original = self.keyboard.keys;

    //     let mut i = 0;
    //     while i < self.keyboard.keys.len() {
    //         if self.keyboard.keys[i] != original[i] {
    //             self.cpu.v[x as usize] = i as u8;
    //             return;
    //         }
    //         i += 1;
    //     }

    //     self.cpu.program_counter -= 2;
    // }

    fn ld_vx_k(&mut self, x: u8) {
        println!("Waiting for {} to be pressed", x);
        if self.wait_for_release_key != 50 {
            if !self.keyboard.keys[self.wait_for_release_key as usize] {
                self.cpu.v[x as usize] = self.wait_for_release_key as u8;
                self.waiting_for_key = false;
                self.wait_for_release_key = 50;
                return;
            }
        }
        for (i, &pressed) in self.keyboard.keys.iter().enumerate() {
            if pressed {
                self.wait_for_release_key = i as u8;
                // self.cpu.v[x as usize] = i as u8;
                // self.waiting_for_key = false;
                return;
            }
        }
        self.waiting_for_key = true;

        // self.cpu.program_counter -= 2;
        return;
    }

    // Fx55
    fn ld_i_vx(&mut self, x: u8) {
        let mut index = 0;
        let mut addr = self.cpu.i; 

        while index <= x {
            self.memory[addr as usize] = self.cpu.v[index as usize];
            index += 1;
            addr += 1;
        }
        
        self.cpu.i += x as u16 + 1;
    }

    // Fx65
    fn ld_vx_i(&mut self, x: u8) {
        let mut index = 0;
        let mut addr = self.cpu.i;

        while index <= x {
            self.cpu.v[index as usize] = self.memory[addr as usize];
            index += 1;
            addr += 1;
        }
        
        self.cpu.i += x as u16 + 1; 
    }

    // Fx33
    // fn ld_b_vx(&mut self, x: u8) {
    //     let three_digits = self.cpu.v[x as usize].to_string();
    //     let three_digit_vec: Vec<char> = three_digits.chars().collect();

    //     self.memory[self.cpu.i as usize] = three_digit_vec[0] as u8;
    //     self.memory[self.cpu.i as usize + 1] = three_digit_vec[1] as u8;
    //     self.memory[self.cpu.i as usize + 2] = three_digit_vec[2] as u8;
    // }
    fn ld_b_vx(&mut self, x: u8) {
        let value = self.cpu.v[x as usize];
        self.memory[self.cpu.i as usize]     = value / 100;          // hundreds
        self.memory[self.cpu.i as usize + 1] = (value / 10) % 10;    // tens
        self.memory[self.cpu.i as usize + 2] = value % 10;           // units
    }


    // Dxyn
    fn drw_vx_vy_nibble(&mut self, x: u8, y: u8, n: u8) {
        let x_coord = self.cpu.v[x as usize] as u16;
        let y_coord = self.cpu.v[y as usize] as u16;
        let num_rows = n as u16;
        let mut flipped = 0;

        for y_line in 0..num_rows {
            let addr = self.cpu.i + y_line as u16;
            let pixels = self.memory[addr as usize];

            for x_line in 0..8 {
                if (pixels & (0b1000_0000 >> x_line)) != 0 {
                    let x = (x_coord + x_line) as usize % 64;
                    let y = (y_coord + y_line) as usize % 32;

                    let idx = x + 64 * y;
                    flipped |= self.display[idx];
                    self.display[idx] ^= 1;                    
                }
            }
        }
        if flipped == 1 {
            self.cpu.v[0xF] = 1;
        }
        else {
            self.cpu.v[0xF] = 0;
        }
        self.draw_flag = true;
    }

    // fetch
    pub fn fetch(&mut self) {
        let first_half = self.memory[self.cpu.program_counter as usize];
        let second_half = self.memory[self.cpu.program_counter as usize + 1];

        let instruction: u16 = ((first_half as u16) << 8) | (second_half as u16);
        println!("Instruction in decimal: {:?}", instruction);

        match instruction & 0xF000 {
            0x0000 => {
                match instruction & 0x00FF {
                    0xE0 => {self.cls();},
                    0xEE => {
                        self.cpu.ret();
                        
                    },
                    0x00 => {},
                    _ => {panic!("Not an opcode: {:#06X}", instruction)}
                }
            },
            0x1000 => {
                let nnn = instruction & 0xFFF;
                self.cpu.jp_addr(nnn);
                return;
            },
            0x2000 => {
                let nnn = instruction & 0xFFF;
                self.cpu.call_addr(nnn);
                // jumped = true;
                return;
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
                    0xE => {self.cpu.shl_vx_vy(x, y);}, 
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
                return;
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
                // println!("made it to xF000 branch");
                let x: u8 = ((instruction & 0x0F00) >> 8) as u8;
                // println!("x from xF000: {}",x);
                // println!("instruction & 0x00FF: {}", instruction & 0x00FF);
                match instruction & 0x00FF {
                    0x07 => {self.cpu.ld_vx_dt(x);},
                    0x0A => {self.ld_vx_k(x);},
                    0x15 => {self.cpu.ld_dt_vx(x);},
                    0x18 => {self.cpu.ld_st_vx(x);},
                    0x1E => {self.cpu.add_i_vx(x);},
                    0x29 => {self.cpu.ld_f_vx(x);},
                    0x33 => {self.ld_b_vx(x);},
                    0x55 => {self.ld_i_vx(x);},
                    0x65 => {self.ld_vx_i(x);},
                    _ => {panic!("Not an opcode")}
                }
            },
            _ => {panic!("Not an opcode")}
        }
        // if !jumped {
            if !self.waiting_for_key {
            self.cpu.program_counter += 2;
            // }
        }
    }

    pub fn load_sprites(&mut self, sprites: [u8; 80]) {
        let mut i = 0;

        while i < sprites.len() {
            self.memory[i] = sprites[i];
            i += 1;
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        if rom.len() + 0x200 > self.memory.len() {
            panic!("ROM too large to fit in memory!");
        }
        for (offset, &byte) in rom.iter().enumerate() {
            self.memory[0x200 + offset] = byte;
        }
    }
}