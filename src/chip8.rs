use rand::prelude::*;

use crate::opcode::Opcode;
use crate::util::byte_to_bits;
use crate::graphics::Screen;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;


static SPRITES: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0],
    [0x20, 0x60, 0x20, 0x20, 0x70],
    [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    [0x90, 0x90, 0xF0, 0x10, 0x10],
    [0xF0, 0x80, 0xF0, 0x10, 0xF0],
    [0xF0, 0x80, 0xF0, 0x90, 0xF0],
    [0xF0, 0x10, 0x20, 0x40, 0x40],
    [0xF0, 0x90, 0xF0, 0x90, 0xF0],
    [0xF0, 0x90, 0xF0, 0x10, 0xF0],
    [0xF0, 0x80, 0x80, 0x80, 0xF0],
    [0xE0, 0x90, 0xE0, 0x90, 0xE0],
    [0xF0, 0x80, 0x80, 0x80, 0xF0],
    [0xE0, 0x90, 0x90, 0x90, 0xE0],
    [0xF0, 0x80, 0xF0, 0x80, 0xF0],
    [0xF0, 0x80, 0xF0, 0x80, 0x80]
];

#[allow(non_snake_case)]
pub struct Chip8 {
    memory: [u8; 4096],
    stack: [u16; 16],
    V: [u8; 16], // CPU registers
    I: u16, // Index register
    pc: u16, // Program counter
    sp: u8, // Stack pointer
    opcode: Opcode,
    delay_timer: u8,
    sound_timer: u8,
    rng: ThreadRng,
    screen: Screen,
    draw_flag: bool,
    event_pump: EventPump,
    keyboard: [bool; 16]
}


impl Chip8 {
    pub fn new() -> Chip8 {
        println!("Initializing emulator.");

        let sdl_context = sdl2::init().unwrap();

        Chip8 {
            memory: Chip8::init_memory(),
            stack: [0; 16],
            V: [0; 16],
            I: 0,
            pc: 0x200,
            sp: 0,
            opcode: Opcode::UNDEFINED,
            delay_timer: 0,
            sound_timer: 0,
            rng: rand::thread_rng(),
            screen: Screen::new(&sdl_context),
            draw_flag: false,
            event_pump: sdl_context.event_pump().unwrap(),
            keyboard: [false; 16]
        }
    }

    fn init_memory() -> [u8; 4096] {
        let mut memory = [0; 4096];

        println!("Loading font into memory.");

        // Load the 16 sprites, each 5 bytes long into the array
        for s in 0..16 {
            for i in 0..5 {
                memory[s*5+i] = SPRITES[s][i];
            }
        }

        memory
    }

    pub fn load_game(&mut self, path: &str) {
        println!("Loading ROM.");

        let f = File::open(path).unwrap();
        let reader = BufReader::new(f);

        let mut index = 0;
        for byte in reader.bytes() {
            let byte = byte.unwrap();
            self.memory[0x200+index] = byte;

            index += 1;
        }

        println!("{} bytes loaded ({})", index-1, path);
    }

    pub fn handle_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    println!("Shutting down!");
                    std::process::exit(0);
                },
                Event::KeyDown { keycode: Some(key), ..} => {
                    let key = match key {
                        Keycode::Num1 => 0x0,
                        Keycode::Num2 => 0x1,
                        Keycode::Num3 => 0x2,
                        Keycode::Num4 => 0x3,
                        Keycode::Q => 0x4,
                        Keycode::W => 0x5,
                        Keycode::E => 0x6,
                        Keycode::R => 0x7,
                        Keycode::A => 0x8,
                        Keycode::S => 0x9,
                        Keycode::D => 0xA,
                        Keycode::F => 0xB,
                        Keycode::Z => 0xC,
                        Keycode::X => 0xD,
                        Keycode::C => 0xE,
                        Keycode::V => 0xF,
                        _ => {continue;}
                    };

                    self.keyboard[key] = true;
                }
                Event::KeyUp { keycode: Some(key), ..} => {
                    let key = match key {
                        Keycode::Num1 => 0x0,
                        Keycode::Num2 => 0x1,
                        Keycode::Num3 => 0x2,
                        Keycode::Num4 => 0x3,
                        Keycode::Q => 0x4,
                        Keycode::W => 0x5,
                        Keycode::E => 0x6,
                        Keycode::R => 0x7,
                        Keycode::A => 0x8,
                        Keycode::S => 0x9,
                        Keycode::D => 0xA,
                        Keycode::F => 0xB,
                        Keycode::Z => 0xC,
                        Keycode::X => 0xD,
                        Keycode::C => 0xE,
                        Keycode::V => 0xF,
                        _ => {continue;}
                    };

                    self.keyboard[key] = false;
                }
                _ => {}
            }
        }
    }

    pub fn tick(&mut self) {
        // Fetch Opcode
        let upper =  self.memory[self.pc as usize] as u16;
        let lower = self.memory[self.pc as usize + 1] as u16;
        let bytes = upper << 8 | lower;

        self.opcode = Opcode::from_bytes(bytes);
        
        // Execute Opcode
        self.execute_opcode();

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer-=1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!");
            }
            self.sound_timer -= 1;
        }

        if self.draw_flag {
            self.screen.draw();
            self.draw_flag = false;
        }
    }

    fn execute_opcode(&mut self) {
        self.pc += 2;

        match self.opcode {
            Opcode::SYS(_nnn) => {
                // Only implemented on original machines
            }
            Opcode::CLS => {
                // Clear the display
                for row in 0..32 {
                    for col in 0..64 {
                        self.screen[row][col] = 0;
                    }
                }
                self.draw_flag = true;
            }
            Opcode::RET => {
                // Return from subroutine
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }
            Opcode::JP(nnn) => {
                // Jump to location nnn
                self.pc = nnn;
            }
            Opcode::CALL(nnn) => {
                // Call subroutine at nnn
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = nnn;
            }
            Opcode::SE(x, kk) => {
                // Skip next instruction if Vx == kk
                if self.V[x] == kk {
                    self.pc += 2;
                }
            }
            Opcode::SNE(x, kk) => {
                // Skip next instruction if Vx != kk
                if self.V[x] != kk {
                    self.pc += 2;
                }
            }
            Opcode::SE_V(x, y) => {
                // Skip next instruction if Vx == Vy
                if self.V[x] == self.V[y] {
                    self.pc += 2;
                }
            }
            Opcode::LD(x, kk) => {
                // Set Vx = kk
                self.V[x] = kk;
            }
            Opcode::ADD(x, kk) => {
                // Set Vx = Vx + kk
                self.V[x] = (self.V[x] as u16 + kk as u16) as u8;
            }
            Opcode::LD_V(x, y) => {
                // Set Vx = Vy
                self.V[x] = self.V[y];
            }
            Opcode::OR(x, y) => {
                // Set Vx = Vx OR Vy
                self.V[x] = self.V[x] | self.V[y];
            }
            Opcode::AND(x, y) => {
                // Set Vx = Vx AND Vy
                self.V[x] = self.V[x] & self.V[y];
            }
            Opcode::XOR(x, y) => {
                // Set Vx = Vx XOR Vy
                self.V[x] = self.V[x] ^ self.V[y];
            }
            Opcode::ADD_V(x, y) => {
                // Set Vx = Vx + Vy, set VF = carry
                let sum = self.V[x] as u16 + self.V[y] as u16;
                self.V[0xF] = if sum > 255 { 1 } else { 0 };

                self.V[x] = sum as u8;
            }
            Opcode::SUB(x, y) => {
                self.V[0xF] = if self.V[x] > self.V[y] { 1 } else { 0 };

                self.V[x] = (self.V[x] as i8 - self.V[y] as i8) as u8;
            }
            Opcode::SHR(x, _y) => {
                // Set Vx = Vx SHR 1
                self.V[0xF] = self.V[x] & 0b0001;
                self.V[x] = self.V[x] >> 1;
            }
            Opcode::SUBN(x, y) => {
                // Set Vx = Vy - Vx, set VF = NOT borrow
                self.V[0xF] = if self.V[y] > self.V[x] { 1 } else { 0 };
                self.V[x] = (self.V[y] as i8 - self.V[x] as i8) as u8;
            }
            Opcode::SHL(x, _y) => {
                // Set Vx = Vx SHL 1
                self.V[0xF] = (self.V[x] & 0b10000000) >> 7;
                self.V[x] = self.V[x] << 1;
            }
            Opcode::SNE_V(x, y) => {
                // Skip next instruction if Vx != Vy
                if self.V[x] != self.V[y] {
                    self.pc += 2;
                }
            }
            Opcode::LD_I(nnn) => {
                // Set I = nnn
                self.I = nnn;
            }
            Opcode::JP_V(nnn) => {
                // Jump to location nnn + V0
                self.pc = (nnn as u32 + self.V[0] as u32) as u16;
            }
            Opcode::RND(x, kk) => {
                // Set Vx = random byte AND kk
                let rand: u8 = self.rng.gen();
                self.V[x] = rand & kk;
            }
            Opcode::DRW(x, y, n) => {
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                self.V[0xF] = 0;
                for row in 0..n as u16 {
                    let byte = self.memory[(row+self.I) as usize];
                    let bits = byte_to_bits(byte);
                    let screen_y = ((self.V[y] as u16 +row) as usize) % 32;

                    for col in 0..8 {
                        let screen_x = (self.V[x] as usize + col) % 64;
                        if self.screen[screen_y][screen_x] & bits[col] == 1 {
                            self.V[0xF] = 1
                        }

                        self.screen[screen_y][screen_x] ^= bits[col]
                    }
                }

                self.draw_flag = true;
            }
            Opcode::SKP(x) => {
                // Skip next instruction if key with the value of Vx is pressed
                if self.keyboard[self.V[x] as usize] {
                    self.pc += 2;
                }
            }
            Opcode::SKNP(x) => {
                // Skip next instruction if key with the value of Vx is not pressed
                if !self.keyboard[self.V[x] as usize] {
                    self.pc += 2;
                }
            }
            Opcode::LD_V_DT(x) => {
                // Set Vx = delay timer value
                self.V[x] = self.delay_timer;
            }
            Opcode::LD_K(x) => {
                // Wait for a key press, store the value of the key in Vx
                let mut initial= self.keyboard;
                let mut keypressed = false;

                loop {
                    self.handle_events();

                    for i in 0..16 {
                        if initial[i] == true && self.keyboard[i] == false {
                            initial[i] = self.keyboard[i];
                        } else if initial[i] == false && self.keyboard[i] == true {
                            keypressed = true;
                            self.V[x] = i as u8;
                            break;
                        }
                    }

                    if keypressed {
                        break;
                    } else {
                        std::thread::sleep(std::time::Duration::from_secs_f64(1.0/60.0));
                    }
                }
                
            }
            Opcode::LD_DT_V(x) => {
                // Set delay timer = Vx
                self.delay_timer = self.V[x];
            }
            Opcode::LD_ST(x) => {
                // Set sound timer = Vx
                self.sound_timer = self.V[x];
            }
            Opcode::ADD_I(x) => {
                // Set I = I + Vx
                self.I = (self.I as u32 + self.V[x] as u32) as u16;
            }
            Opcode::LD_F(x) => {
                // Set I = location of sprite for digit Vx
                self.I = (self.V[x] * 5) as u16;
            }
            Opcode::LD_B(x) => {
                // Store BCD representation of Vx in memory locations I, I+1, and I+2
                let mut num = self.V[x];

                let ones = num % 10;
                num /= 10;

                let tens = num % 10;
                num /= 10;

                let hundreds = num;

                self.memory[self.I as usize] = hundreds;
                self.memory[self.I as usize +1] = tens;
                self.memory[self.I as usize +2] = ones;

            }
            Opcode::LD_I_V(x) => {
                // Store registers V0 though Vx in memory starting at location I
                for i in 0..x+1 {
                    self.memory[self.I as usize+i] = self.V[i];
                }
            }
            Opcode::LD_V_I(x) => {
                // Read registers V0 through Vx from memory starting at location I
                for i in 0..x+1 {
                    self.V[i] = self.memory[self.I as usize+i];
                }
            }
            Opcode::UNDEFINED => {
                eprintln!("Unrecognized opcode!");
            }
        }
    }
}


#[test]
fn test_CLS() {
    let mut chip8 = Chip8::new();
    chip8.screen[2][0] = 1;
    chip8.screen[23][23] = 1;

    chip8.opcode = Opcode::CLS;
    chip8.execute_opcode();

    for row in 0..32 {
        for col in 0..64 {
            assert_eq!(chip8.screen[row][col], 0);
        }
    }

    assert!(chip8.draw_flag);
}

#[test]
fn test_CALL_RET() {
    let mut chip8 = Chip8::new();

    chip8.opcode = Opcode::CALL(0x200 + 4);
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 0x200 + 4);
    assert_eq!(chip8.stack[chip8.sp as usize], 0x200 + 2);

    chip8.opcode = Opcode::RET;
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 0x200 + 2);
    assert_eq!(chip8.sp, 0);

}
#[test]
fn test_JP() {
    let mut chip8 = Chip8::new();
    chip8.opcode = Opcode::JP(0x200 + 2);
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 0x200 + 2);
}

#[test]
fn test_SE_true() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 1;

    chip8.opcode = Opcode::SE(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 0x200 + 4);
}

#[test]
fn test_SE_false() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 1;

    chip8.opcode = Opcode::SE(0, 0);
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 0x200 + 2);
}

#[test]
fn test_SNE_true() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 1;

    chip8.opcode = Opcode::SNE(0, 0);
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 0x200 + 4);
}

#[test]
fn test_SNE_false() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 1;

    chip8.opcode = Opcode::SNE(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 0x200 + 2);
}

#[test]
fn test_SE_V() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 1;
    chip8.V[1] = 1;

    chip8.opcode = Opcode::SE_V(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 0x200 + 4);
}

#[test]
fn test_SE_V_false() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 0;
    chip8.V[1] = 1;

    chip8.opcode = Opcode::SE_V(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 0x200 + 2);
}

#[test]
fn test_LD() {
    let mut chip8 = Chip8::new();
    
    chip8.opcode = Opcode::LD(0, 0x4f);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 0x4f);
}

#[test]
fn test_ADD() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 13;

    chip8.opcode = Opcode::ADD(0, 17);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 30);
}

#[test]
fn test_ADD_overflow() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 15;

    chip8.opcode = Opcode::ADD(0, 255);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 14);
}

#[test]
fn test_LD_V() {
    let mut chip8 = Chip8::new();
    chip8.V[1] = 0x6a;

    chip8.opcode = Opcode::LD_V(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 0x6a);
}

#[test]
fn test_OR() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 0b00001111;
    chip8.V[1] = 0b10101010;

    chip8.opcode = Opcode::OR(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 0b10101111);
}

#[test]
fn test_AND() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 0b00001111;
    chip8.V[1] = 0b10101010;

    chip8.opcode = Opcode::AND(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 0b00001010);
}

#[test]
fn test_XOR() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 0b00001111;
    chip8.V[1] = 0b10101010;

    chip8.opcode = Opcode::XOR(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 0b10100101);
}

#[test]
fn test_ADD_V() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 5;
    chip8.V[1] = 7;

    chip8.opcode = Opcode::ADD_V(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 12);
    assert_eq!(chip8.V[0xF], 0);
}

#[test]
fn test_ADD_V_overflow() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 5;
    chip8.V[1] = 255;

    chip8.opcode = Opcode::ADD_V(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 4);
    assert_eq!(chip8.V[0xF], 1);
}

#[test]
fn test_SUB() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 12;
    chip8.V[1] = 7;

    chip8.opcode = Opcode::SUB(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 5);
    assert_eq!(chip8.V[0xF], 1);
}

#[test]
fn test_SUB_negative() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 5;
    chip8.V[1] = 12;

    chip8.opcode = Opcode::SUB(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 249);
    assert_eq!(chip8.V[0xF], 0);
}

#[test]
fn test_SHR() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 0b01001100;

    chip8.opcode = Opcode::SHR(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 0b00100110);
    assert_eq!(chip8.V[0xF], 0);
}

#[test]
fn test_SHR_loss() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 0b01001101;

    chip8.opcode = Opcode::SHR(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 0b00100110);
    assert_eq!(chip8.V[0xF], 1);
}

#[test]
fn test_SUBN() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 5;
    chip8.V[1] = 12;

    chip8.opcode = Opcode::SUBN(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 7);
    assert_eq!(chip8.V[0xF], 1);
}

#[test]
fn test_SUBN_overflow() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 12;
    chip8.V[1] = 5;

    chip8.opcode = Opcode::SUBN(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 249);
    assert_eq!(chip8.V[0xF], 0);
}

#[test]
fn test_SHL() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 0b01001100;

    chip8.opcode = Opcode::SHL(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 0b10011000);
    assert_eq!(chip8.V[0xF], 0);
}

#[test]
fn test_SHL_loss() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 0b11001100;

    chip8.opcode = Opcode::SHL(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 0b10011000);
    assert_eq!(chip8.V[0xF], 1);
}

#[test]
fn test_SNE_V() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 0x3b;
    chip8.V[1] = 0x20;

    chip8.opcode = Opcode::SNE_V(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 0x200 + 4);
}

#[test]
fn test_SNE_V_false() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 0x3b;
    chip8.V[1] = 0x3b;

    chip8.opcode = Opcode::SNE_V(0, 1);
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 0x200 + 2);
}

#[test]
fn test_LD_I() {
    let mut chip8 = Chip8::new();
    
    chip8.opcode = Opcode::LD_I(0xff55);
    chip8.execute_opcode();

    assert_eq!(chip8.I, 0xff55);
}

#[test]
fn test_JP_V() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 20;

    chip8.opcode = Opcode::JP_V(101);
    chip8.execute_opcode();

    assert_eq!(chip8.pc, 121);
}

#[test]
fn test_RND() {
    let mut chip8 = Chip8::new();

    chip8.opcode = Opcode::RND(0, 0xFF);
    chip8.execute_opcode();

    assert!(chip8.V[0] != 0);
}

#[test]
fn test_DRW() {
    let mut chip8 = Chip8::new();

    chip8.memory[0x200]   = 0b00011000;
    chip8.memory[0x200+1] = 0b00011000;
    chip8.memory[0x200+2] = 0b11111111;
    chip8.memory[0x200+3] = 0b11111111;
    chip8.memory[0x200+4] = 0b00011000;
    chip8.memory[0x200+5] = 0b00011000;
    chip8.I = 0x200;

    chip8.V[0] = 0;
    chip8.V[1] = 0;
    chip8.opcode = Opcode::DRW(0, 1, 6);
    chip8.execute_opcode();

    chip8.V[0] = 56;
    chip8.V[1] = 0;
    chip8.opcode = Opcode::DRW(0, 1, 6);
    chip8.execute_opcode();

    chip8.V[0] = 0;
    chip8.V[1] = 26;
    chip8.opcode = Opcode::DRW(0, 1, 6);
    chip8.execute_opcode();

    chip8.V[0] = 56;
    chip8.V[1] = 26;
    chip8.opcode = Opcode::DRW(0, 1, 6);
    chip8.execute_opcode();
    
    chip8.screen.draw();

    std::thread::sleep(std::time::Duration::from_secs_f32(3.0));
}

#[test]
fn test_LD_V_DT() {
    let mut chip8 = Chip8::new();
    chip8.delay_timer = 12;

    chip8.opcode = Opcode::LD_V_DT(0);
    chip8.execute_opcode();

    assert_eq!(chip8.V[0], 12);
}

#[test]
fn test_LD_DT_V() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 0xfa;
    
    chip8.opcode = Opcode::LD_DT_V(0);
    chip8.execute_opcode();

    assert_eq!(chip8.delay_timer, 0xfa);
}

#[test]
fn test_LD_ST() {
    let mut chip8 = Chip8::new();
    chip8.V[0] = 42;

    chip8.opcode = Opcode::LD_ST(0);
    chip8.execute_opcode();

    assert_eq!(chip8.sound_timer, 42);
}

#[test]
fn test_ADD_I() {
    let mut chip8 = Chip8::new();
    chip8.I = 17;
    chip8.V[0] = 10;

    chip8.opcode = Opcode::ADD_I(0);
    chip8.execute_opcode();

    assert_eq!(chip8.I, 27);
}

#[test]
fn test_ADD_I_overflow() {
    let mut chip8 = Chip8::new();
    chip8.I = 17;
    chip8.V[0] = 255;

    chip8.opcode = Opcode::ADD_I(0);
    chip8.execute_opcode();

    assert_eq!(chip8.I, 272);
}

#[test]
fn test_LD_F() {
    let mut chip8 = Chip8::new();

    for i in 0..0xF {
        chip8.V[0] = i;
        chip8.opcode = Opcode::LD_F(0);
        chip8.execute_opcode();

        assert_eq!(chip8.I, i as u16*5);
    }
}

#[test]
fn test_LD_B() {
    let mut chip8 = Chip8::new();
    chip8.opcode = Opcode::LD(0, 0xF3);
    chip8.execute_opcode();

    chip8.opcode = Opcode::LD_B(0);
    chip8.execute_opcode();

    assert_eq!(chip8.memory[0], 2);
    assert_eq!(chip8.memory[1], 4);
    assert_eq!(chip8.memory[2], 3);
}

#[test]
fn test_LD_I_V() {
    let mut chip8 = Chip8::new();
    chip8.V = [0,1,6,4,3,6,7,3,2,7,5,4,4,6,3,2];

    chip8.opcode = Opcode::LD_I_V(0xF);
    chip8.execute_opcode();

    assert_eq!(
        chip8.memory[chip8.I as usize..chip8.I as usize+0xF+1],
        [0,1,6,4,3,6,7,3,2,7,5,4,4,6,3,2]
    );
}

#[test]
fn test_LD_V_I() {
    let mut chip8 = Chip8::new();
    chip8.I = 0x200;
    let vals = [0,1,6,4,3,6,7,3,2,7,5,4,4,6,3,2];

    for (i, val) in vals.iter().enumerate() {
        chip8.memory[chip8.I as usize + i] = *val;
    }

    chip8.opcode = Opcode::LD_V_I(0xF);
    chip8.execute_opcode();

    assert_eq!(chip8.V, vals);
}