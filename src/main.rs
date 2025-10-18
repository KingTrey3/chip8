use std::array;

fn main() {
    println!("Hello, world!");
}

struct CPU {
    v: [u8; 16],
    delay: u8,
    sound: u8,
    program_counter: u16,
    stack_pointer: u8,
    stack: [u16; 16]
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

