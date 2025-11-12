pub mod cpu;
pub mod chip8;
use std::time::{Instant, Duration};
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::controller::Button;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::env;
use std::fs;
use cpu::CPU;
use chip8::Chip8;
use crate::chip8::Keyboard;

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

    if contoller_id.is_some() {
        controller_subsystem.open(contoller_id.unwrap()).unwrap();
    }
    
    // let cpu_hz = 600;
    // let cpu_cycle_duration = Duration::from_micros(1_000_000 / cpu_hz);

    // let mut last_cpu_cycle = Instant::now();
    let mut last_timer_update = Instant::now();
    let timer_interval = Duration::from_micros(1_000_000 / 60);

    'running: loop {
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
