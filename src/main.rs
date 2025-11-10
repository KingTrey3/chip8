use std::time::{Instant, Duration};
use rand::random_range;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::controller::Button;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::env;
use std::fs;

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
    let args: Vec<String> = env::args().collect();
    let rom_file_path = &args[1];
    let rom_bytes = fs::read(rom_file_path)
        .expect("Should have been able to read the file");

    let mut chip8: Chip8 = Chip8 { 
        memory: [0; 4096], 
        cpu: CPU { 
            v: [0; 16], 
            delay: 0, 
            sound: 0, 
            program_counter: 0x200, 
            stack_pointer: 0, 
            stack: [0; 16], 
            i: 0 }, 
        keyboard: Keyboard { keys: [false; 16] }, 
        display: [0; 64 * 32],
        draw_flag: false,
        waiting_for_key: false,
        wait_for_release_key: 50,
    };

    chip8.load_sprites(SPRITES);
    chip8.load_rom(rom_bytes);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25
        }
    }).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let window = video_subsystem.window("Chip 8 Window", 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let controller_subsystem = sdl_context.game_controller().unwrap();
    let mut num_of_joysticks = None;
    let mut contoller_id = None;

    match controller_subsystem.num_joysticks() {
        Ok(num) => {num_of_joysticks = Some(num)},
        Err(e) => println!("{}", e)
    }

    for id in 0..num_of_joysticks.unwrap() {
        if controller_subsystem.is_game_controller(id) {
            match controller_subsystem.open(id) {
                Ok(controller) => {
                    println!("Opened controller: {}", controller.name());
                    contoller_id = Some(id);
                },
                Err(e) => {
                    println!("{}", e)
                }
            }
        }
    }

    // let controller = controller_subsystem.open(contoller_id.unwrap()).unwrap();
    
    // let cpu_hz = 600;
    // let cpu_cycle_duration = Duration::from_micros(1_000_000 / cpu_hz);

    // let mut last_cpu_cycle = Instant::now();
    let mut last_timer_update = Instant::now();
    let timer_interval = Duration::from_micros(1_000_000 / 60);

    'running: loop {
        // let controller = controller_subsystem.open(contoller_id.unwrap()).unwrap();
        let now = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown {keycode: Some(Keycode::Num1), .. } => {
                    println!("1 was pressed");
                    chip8.keyboard.keys[1] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::Num1), .. } => {
                    chip8.keyboard.keys[1] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::Num2), .. } => {
                    chip8.keyboard.keys[2] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::Num2), .. } => {
                    chip8.keyboard.keys[2] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::Num3), .. } => {
                    chip8.keyboard.keys[3] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::Num3), .. } => {
                    chip8.keyboard.keys[3] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::Num4), .. } => {
                    chip8.keyboard.keys[0xC] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::Num4), .. } => {
                    chip8.keyboard.keys[0xC] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::Q), .. } => {
                    chip8.keyboard.keys[4] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::Q), .. } => {
                    chip8.keyboard.keys[4] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::W), .. } => {
                    chip8.keyboard.keys[5] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::W), .. } => {
                    chip8.keyboard.keys[5] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::E), .. } => {
                    chip8.keyboard.keys[6] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::E), .. } => {
                    chip8.keyboard.keys[6] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::R), .. } => {
                    chip8.keyboard.keys[0xD] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::R), .. } => {
                    chip8.keyboard.keys[0xD] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::A), .. } => {
                    chip8.keyboard.keys[7] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::A), .. } => {
                    chip8.keyboard.keys[7] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::S), .. } => {
                    chip8.keyboard.keys[8] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::S), .. } => {
                    chip8.keyboard.keys[8] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::D), .. } => {
                    chip8.keyboard.keys[9] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::D), .. } => {
                    chip8.keyboard.keys[9] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::F), .. } => {
                    chip8.keyboard.keys[0xE] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::F), .. } => {
                    chip8.keyboard.keys[0xE] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::Z), .. } => {
                    chip8.keyboard.keys[0xA] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::Z), .. } => {
                    chip8.keyboard.keys[0xA] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::X), .. } => {
                    chip8.keyboard.keys[0] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::X), .. } => {
                    chip8.keyboard.keys[0] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::C), .. } => {
                    chip8.keyboard.keys[0xB] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::C), .. } => {
                    chip8.keyboard.keys[0xB] = false;
                },
                Event::KeyDown {keycode: Some(Keycode::V), .. } => {
                    chip8.keyboard.keys[0xF] = true;
                },
                Event::KeyUp {keycode: Some(Keycode::V), .. } => {
                    chip8.keyboard.keys[0xF] = false;
                },
                // controller inputs
                Event::ControllerButtonDown {button, .. } if button == Button::A => {
                    chip8.keyboard.keys[4] = true;
                },
                Event::ControllerButtonDown {button, .. } if button == Button::DPadLeft => {
                    chip8.keyboard.keys[5] = true;
                },
                Event::ControllerButtonDown {button, .. } if button == Button::DPadRight => {
                    chip8.keyboard.keys[6] = true;
                },
                Event::ControllerButtonDown {button, .. } if button == Button::DPadDown => {
                    chip8.keyboard.keys[7] = true;
                },
                Event::ControllerButtonUp {button, .. } if button == Button::A => {
                    chip8.keyboard.keys[4] = false;
                },
                Event::ControllerButtonUp {button, .. } if button == Button::DPadLeft => {
                    chip8.keyboard.keys[5] = false;
                },
                Event::ControllerButtonUp {button, .. } if button == Button::DPadRight => {
                    chip8.keyboard.keys[6] = false;
                },
                Event::ControllerButtonUp {button, .. } if button == Button::DPadDown => {
                    chip8.keyboard.keys[7] = false;
                },
                _ => {},
            }
        }

        if chip8.draw_flag {
                canvas.set_draw_color(sdl2::pixels::Color::BLACK);
                canvas.clear();
                canvas.set_draw_color(sdl2::pixels::Color::WHITE);
                let scale = 10;

                for y in 0..32 {
                    for x in 0..64 {
                        let index = y * 64 + x;
                        if chip8.display[index] != 0 {
                            let _ = canvas.fill_rect(sdl2::rect::Rect::new(
                                (x * scale) as i32,
                                (y * scale) as i32,
                                scale as u32,
                                scale as u32,
                            ));
                        } 
                    }
                }
                chip8.draw_flag = false;
                canvas.present();
            }

        // let now = Instant::now();
        // while now.duration_since(last_cpu_cycle) >= cpu_cycle_duration {
        // if !chip8.waiting_for_key {
        //     chip8.fetch();
        //     last_cpu_cycle += cpu_cycle_duration;
        //     if chip8.draw_flag {
        //         canvas.set_draw_color(sdl2::pixels::Color::BLACK);
        //         canvas.clear();
        //         canvas.set_draw_color(sdl2::pixels::Color::WHITE);
        //         let scale = 10;

        //         for y in 0..32 {
        //             for x in 0..64 {
        //                 let index = y * 64 + x;
        //                 if chip8.display[index] != 0 {
        //                     let _ = canvas.fill_rect(sdl2::rect::Rect::new(
        //                         (x * scale) as i32,
        //                         (y * scale) as i32,
        //                         scale as u32,
        //                         scale as u32,
        //                     ));
        //                 } 
        //             }
        //         }
        //         chip8.draw_flag = false;
        //         println!("About to draw");
        //         canvas.present();
        //     }
        // }
        // }

        for _ in 0..10 {
        // if !chip8.waiting_for_key {
            chip8.fetch();
            // } else {
            //     println!("Waiting");
            // }
        }        

              
        println!("delay timer {}", chip8.cpu.delay);
        
        // let now = Instant::now();
        if now.duration_since(last_timer_update) >= timer_interval {
            if chip8.cpu.delay > 0 { chip8.cpu.delay -= 1; }
            if chip8.cpu.sound > 0 { 
                chip8.cpu.sound -= 1;
                device.resume(); 
            } else {
                device.pause();
            }
            last_timer_update = now;
            // last_timer_update += timer_interval;
        }
        std::thread::sleep(Duration::from_millis(16));
    }
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
    fn jp_addr(&mut self, nnn: u16) {
        self.program_counter = nnn;
    }

    // 2nnn
    // fn call_addr(&mut self, nnn: u16) {        
    //     self.stack[self.stack_pointer as usize] = self.program_counter;
    //     self.stack_pointer += 1;
    //     self.program_counter = nnn;
    // }
    fn call_addr(&mut self, nnn: u16) {
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
    // fn add_vx_byte(&mut self, x: u8, kk: u8) {
    //     self.v[x as usize] += kk;
    // }
    fn add_vx_byte(&mut self, x: u8, kk: u8) {
        self.v[x as usize] = self.v[x as usize].wrapping_add(kk);
    }

    // 8xy0
    fn ld_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[y as usize];
    }

    // 8xy1
    fn or_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] |= self.v[y as usize];
        self.v[0xF] = 0
    }

    // 8xy2
    fn and_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] &= self.v[y as usize];
        self.v[0xF] = 0
    }

    // 8xy3
    fn xor_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] ^= self.v[y as usize];
        self.v[0xF] = 0
    }

    // 8xy4
    fn add_vx_vy(&mut self, x: u8, y: u8) {
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
    fn sub_vx_vy(&mut self, x: u8, y: u8) {
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
    fn shr_vx_vy(&mut self, x: u8, y: u8) {
        let lsb = self.v[y as usize] & 0b0000_0001;
        // self.v[0xF] = lsb;
        self.v[x as usize] = self.v[y as usize] >> 1;
        self.v[0xF] = lsb;
    }

    // 8xy7
    fn subn_vx_vy(&mut self, x: u8, y: u8) {
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
    fn shl_vx_vy(&mut self, x: u8, y: u8) {
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
        // let twelve_bit = nnn & 0b0000_1111_1111_1111;
        // let shifted = twelve_bit >> 8;
        // let x = shifted & 0b1111;

        // let vx = self.v[x as usize];
        // self.program_counter = nnn + vx as u16;
        let v0 = self.v[0] as u16;
        self.program_counter = v0 + nnn;
    }

    // Cxkk
    fn rnd_vx_byte(&mut self, x: u8, kk: u8) {
        let rand_byte: u8 = random_range(0..=255);
        self.v[x as usize] = rand_byte & kk;
    }

    // Fx07
    fn ld_vx_dt(&mut self, x: u8) {
        self.v[x as usize] = self.delay;
    }

    // Fx15
    fn ld_dt_vx(&mut self, x: u8) {
        println!("setting delay timer");
        self.delay = self.v[x as usize];
        println!("x: {}", x);
        println!("delay timer: {}", self.delay);
    }

    // Fx18
    fn ld_st_vx(&mut self, x: u8) {
        self.sound = self.v[x as usize];
    }

    // Fx1E
    fn add_i_vx(&mut self, x: u8) {
        let vx = self.v[x as usize] as u16; 
        self.i += vx;
    }

    // Fx29
    // fn ld_f_vx(&mut self, x: u8) {
    //     self.i = (self.v[x as usize] * 5) as u16;
    // }
    fn ld_f_vx(&mut self, x: u8) {
        self.i = (self.v[x as usize] as u16) * 5;
    }

    fn push(&mut self, instuction: u16) {
        self.stack[self.stack_pointer as usize] = instuction;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }
}

struct Keyboard {
    keys: [bool; 16]
}

struct Chip8 {
    memory: [u8; 4096],
    cpu: CPU,
    keyboard: Keyboard,
    display: [u8; 64 * 32],
    draw_flag: bool,
    waiting_for_key: bool,
    wait_for_release_key: u8
    // timers: timers,
    // sound: sound
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
        // self.cpu.v[15] = 0;
        // let mut any_pixel_on = false;

        // for index in 0..n {
        //     let mut sprite = self.memory[(self.cpu.i + (index as u16)) as usize];
        //     let row = (self.cpu.v[y as usize] + index) % 32;

        //     for index in 0..8 {
        //         let b = (sprite & 0x80) >> 7;
        //         let col = (self.cpu.v[x as usize] + index) % 64;
        //         let offset = (row as usize) * 64 + (col as usize);

        //         if b == 1 {
        //             if self.display[offset as usize] != 0 {
        //                 self.display[offset as usize] = 0;
        //                 self.cpu.v[15] = 1;
        //             } else {
        //                 self.display[offset as usize] = 1;
        //                 any_pixel_on = true;
        //             }
        //         }
        //         sprite <<= 1;
        //     }
        // }
        // println!("DRW: drew sprite at ({}, {}), any_pixel_on={}", self.cpu.v[x as usize], self.cpu.v[y as usize], any_pixel_on);
        // self.draw_flag = true;

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
    fn fetch(&mut self) {
        let first_half = self.memory[self.cpu.program_counter as usize];
        let second_half = self.memory[self.cpu.program_counter as usize + 1];

        let instruction: u16 = ((first_half as u16) << 8) | (second_half as u16);
        println!("Instruction in decimal: {:?}", instruction);
        // let mut jumped = false;

        match instruction & 0xF000 {
            0x0000 => {
                match instruction & 0x00FF {
                    0xE0 => {self.cls();},
                    0xEE => {
                        self.cpu.ret();
                        // jumped = true;
                        
                    },
                    0x00 => {},
                    _ => {panic!("Not an opcode: {:#06X}", instruction)}
                }
            },
            0x1000 => {
                let nnn = instruction & 0xFFF;
                self.cpu.jp_addr(nnn);
                // jumped = true;
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

    fn load_sprites(&mut self, sprites: [u8; 80]) {
        let mut i = 0;

        while i < sprites.len() {
            self.memory[i] = sprites[i];
            i += 1;
        }
    }

    // fn load_rom(&mut self, rom: Vec<u8>) {
    //     let mut i = 512;

    //     for byte in rom {
    //         self.memory[i] = byte;
    //          i += 1;
    //     }
    // }

    fn load_rom(&mut self, rom: Vec<u8>) {
    if rom.len() + 0x200 > self.memory.len() {
        panic!("ROM too large to fit in memory!");
    }
    for (offset, &byte) in rom.iter().enumerate() {
        self.memory[0x200 + offset] = byte;
    }
}

}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                - self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
