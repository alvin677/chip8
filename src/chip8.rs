use std::fs;

pub struct CHIP8 {
    pub memory: [u8; 4096],
    pub vregister: [u8; 16],
    pub index_register: u16,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub stack: [u16; 16],

    pub delay_timer: u8,
    pub sound_timer: u8,

    pub display: [u8; 64 * 32],
    pub keypad: [bool; 16],

    pub debug: bool,
}

impl CHIP8 {
    pub fn new() -> Self {
        return Self {
            memory: [0; 4096], // empty memory
            vregister: [0; 16],
            index_register: 0x0,
            program_counter: 0x200,
            stack_pointer: 0,
            stack: [0; 16],

            delay_timer: 0,
            sound_timer: 0,

            display: [0; 64 * 32], // black screen
            keypad: [false; 16],   // the 16-key hexadecimal keypad

            debug: false,
        };
    }

    pub fn load_fonts(&mut self) {
        const START_ADDRESS: usize = 0x50; // 80 decimal

        // load fonts from 0x50 to 0x9F
        const FONT_SET: [u8; 80] = [
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

        for (i, &byte) in FONT_SET.iter().enumerate() {
            self.memory[START_ADDRESS + i] = byte;
        }
    }

    // load the rom to the starting address (0x200)
    pub fn load_rom(&mut self, rom_file: &str) {
        const START_ADDRESS: usize = 0x200; // 512 decimal

        let data = fs::read(rom_file).unwrap();

        for (i, &byte) in data.iter().enumerate() {
            self.memory[START_ADDRESS + i] = byte;
        }
    }

    pub fn cycle(&mut self) {
        let msb = self.memory[self.program_counter as usize];
        let lsb = self.memory[(self.program_counter + 1) as usize];

        let opcode: u16 = ((msb as u16) << 8) | lsb as u16;

        /*if self.debug {
            println!(
                "address: 0x{:x}, opcode: {:x}",
                self.program_counter, opcode
            );
        }*/

        // process the opcode
        match opcode {
            0x00E0 => {
                // clear the display
                if self.debug {
                    println!("0x{:x} clearing screen", opcode)
                }

                self.display = [0; 64 * 32];
                self.program_counter += 0x02; // increment the counter to the next address (opcodes on the chip8 are 2 bytes)
            }
            0x00EE => {
                // return from a subroutine
                if self.debug {
                    println!("0x{:x} returning from subroutine", opcode);
                }

                self.stack_pointer -= 1;
                self.program_counter = self.stack[self.stack_pointer as usize];
                self.program_counter += 0x02;
            }
            0x1000..=0x1FFF => {
                // jump to location nnn
                // (check if opcode starts with 1 and is within range)
                if self.debug {
                    println!("0x{:x} jumping to location", opcode);
                }

                self.program_counter = opcode & 0x0FFF; // bitwise AND to remove the first nibble
            }
            0x2000..=0x2FFF => {
                // call subroutine at nnn
                if self.debug {
                    println!("0x{:x} calling subroutine", opcode);
                }

                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.stack_pointer += 1;
                self.program_counter = opcode & 0x0FFF;
            }
            0x3000..=0x3FFF => {
                // skip next instruction if Vx == kk
                let reg = ((opcode & 0x0F00) >> 8) as usize;
                let kk = (opcode & 0x00FF) as u8;

                if self.debug {
                    println!(
                        "0x{:x} skipping next instruction if register V{} == {}",
                        opcode, reg, kk
                    );
                }

                if self.vregister[reg] == kk {
                    self.program_counter += 2; // skip next instruction
                }

                self.program_counter += 0x02;
            }
            0x4000..=0x4FFF => {
                // skip next instruction if Vx != kk
                let reg = ((opcode & 0x0F00) >> 8) as usize;
                let kk = (opcode & 0x00FF) as u8;

                if self.debug {
                    println!(
                        "0x{:x} skipping next instruction if register V{} != {}",
                        opcode, reg, kk
                    );
                }

                if self.vregister[reg] != kk {
                    self.program_counter += 2;
                }

                self.program_counter += 0x02;
            }
            0x5000..=0x5FFF => {
                // skip next instruction if Vx == Vy
                let reg_x = ((opcode & 0x0F00) >> 8) as usize;
                let reg_y = ((opcode & 0x00F0) >> 4) as usize;

                if self.debug {
                    println!(
                        "0x{:x} skipping next instruction if register V{} == V{}",
                        opcode, reg_x, reg_y
                    );
                }

                if self.vregister[reg_x] == self.vregister[reg_y] {
                    self.program_counter += 2;
                }

                self.program_counter += 0x02;
            }
            0x6000..=0x6FFF => {
                // put value kk into register Vx
                let reg = ((opcode & 0x0F00) >> 8) as usize;
                let kk = (opcode & 0x00FF) as u8;

                if self.debug {
                    println!("0x{:x} setting register V{} to {}", opcode, reg, kk);
                }

                self.vregister[reg] = kk;
                self.program_counter += 0x02;
            }
            0x7000..=0x7FFF => {
                // set Vx = Vx + kk
                let reg = ((opcode & 0x0F00) >> 8) as usize;
                let kk = (opcode & 0x00FF) as u8;

                if self.debug {
                    println!("0x{:x} adding {} to register V{}", opcode, kk, reg);
                }

                self.vregister[reg] = self.vregister[reg].wrapping_add(kk);
                self.program_counter += 0x02;
            }
            0x8000..=0x8FFF => {
                // 0x8 has multiple variants, handle all here based on the last nibble
                let reg_x = ((opcode & 0x0F00) >> 8) as usize;
                let reg_y = ((opcode & 0x00F0) >> 4) as usize;
                let last_nibble = opcode & 0x000F;

                match last_nibble {
                    0 => self.vregister[reg_x] = self.vregister[reg_y], // set Vx = Vy
                    1 => self.vregister[reg_x] = self.vregister[reg_x] | self.vregister[reg_y], // set Vx = Vx OR Vy
                    2 => self.vregister[reg_x] = self.vregister[reg_x] & self.vregister[reg_y], // set Vx = Vx AND Vy
                    3 => self.vregister[reg_x] = self.vregister[reg_x] ^ self.vregister[reg_y], // set Vx = Vx XOR Vy
                    4 => {
                        let (result, carry) =
                            self.vregister[reg_x].overflowing_add(self.vregister[reg_y]);

                        self.vregister[reg_x] = result;
                        self.vregister[0xF] = if carry { 1 } else { 0 };
                    } // set Vx = Vx + Vy, set VF = carry
                    5 => {
                        self.vregister[0xF] = if self.vregister[reg_x] > self.vregister[reg_y] {
                            1
                        } else {
                            0
                        };

                        self.vregister[reg_x] =
                            self.vregister[reg_x].wrapping_sub(self.vregister[reg_y]);
                    } // set Vx = Vx - Vy, set VF = NOT borrow
                    6 => {
                        let lsb = self.vregister[reg_x] & 0x01; // least-significant bit
                        self.vregister[0xF] = if lsb == 1 { 1 } else { 0 };

                        self.vregister[reg_x] /= 2; // shift right
                    } // set Vx = Vx SHR (shift right) 1
                    7 => {
                        self.vregister[0xF] = if self.vregister[reg_y] > self.vregister[reg_x] {
                            1
                        } else {
                            0
                        };

                        self.vregister[reg_x] =
                            self.vregister[reg_y].wrapping_sub(self.vregister[reg_x]);
                    } // set Vx = Vy - Vx, set VF = NOT borrow
                    0xE => {
                        let msb = (self.vregister[reg_x] & 0x80) >> 7; // most-significant bit

                        self.vregister[0xF] = if msb == 1 { 1 } else { 0 };
                        self.vregister[reg_x] = self.vregister[reg_x].wrapping_mul(2);
                    } // set Vx = Vx SHL (shift left) 1
                    _ => println!("unknown 0x8xxx opcode variant: {}", last_nibble),
                }

                self.program_counter += 0x02;
            }
            0x9000..=0x9FFF => {
                let reg_x = ((opcode & 0x0F00) >> 8) as usize;
                let reg_y = ((opcode & 0x00F0) >> 4) as usize;

                if self.vregister[reg_x] != self.vregister[reg_y] {
                    self.program_counter += 2;
                }

                self.program_counter += 0x02;
            } // skip next instruction if Vx != Vy
            0xA000..=0xAFFF => {
                let nnn = opcode & 0x0FFF;

                self.index_register = nnn;
                self.program_counter += 0x02;
            } // set I = nnn
            0xB000..=0xBFFF => {
                let nnn = opcode & 0x0FFF;

                self.program_counter = nnn + self.vregister[0x0] as u16;
            } // jump to location nnn + V0
            0xC000..=0xCFFF => {
                let reg = ((opcode & 0x0F00) >> 8) as usize;
                let kk = (opcode & 0x00FF) as u8;
                let random_byte: u8 = rand::random();

                self.vregister[reg] = random_byte & kk;
                self.program_counter += 0x02;
            } // set Vx = random byte AND kk
            0xD000..=0xDFFF => {
                // sprites are 8 bits wide and n-bytes tall (+1 on the y-axis)
                let n = opcode & 0x000F;
                let x = self.vregister[((opcode & 0x0F00) >> 8) as usize] as usize;
                let y = self.vregister[((opcode & 0x00F0) >> 4) as usize] as usize;

                self.vregister[0xF] = 0;

                let mut reading_bytes = Vec::new();

                // read n amount of bytes starting from the index (I) register
                // and push them to the reading_bytes vector
                for i in 0..n {
                    reading_bytes.push(self.memory[(self.index_register + i) as usize]);
                }

                // now go through each bit in the bytes
                for (row, byte) in reading_bytes.iter().enumerate() {
                    for col in 0..8 {
                        let bit = (byte >> (7 - col)) & 0x01; // extract the bits from each byte each iteration

                        // if the bit is on
                        // then figure out the index equivalent to (x, y) on the screen and XOR with 1
                        if bit == 1 {
                            let pixel_index = (x + col) + (y + row) * 64;
                            self.display[pixel_index] ^= 1;
                        }
                    }
                }

                self.program_counter += 0x02;
            } // display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            0xE000..=0xEFFF => {
                let reg = ((opcode & 0x0F00) >> 8) as usize;
                let last_nibbles = opcode & 0x00FF;

                match last_nibbles {
                    0x9E => {
                        if self.keypad[self.vregister[reg] as usize] == true {
                            self.program_counter += 2;
                        }
                    } // skip next instruction if key with the value of Vx is pressed
                    0xA1 => {
                        if self.keypad[self.vregister[reg] as usize] == false {
                            self.program_counter += 2;
                        }
                    } // skip next instruction if key with the value of Vx is not pressed
                    _ => println!("unknown last two nibbles of 0xExxx"),
                }

                self.program_counter += 0x02;
            }
            0xF000..=0xFFFF => {
                let reg = ((opcode & 0x0F00) >> 8) as usize;
                let last_nibbles = opcode & 0x00FF;

                match last_nibbles {
                    0x07 => {
                        self.vregister[reg] = self.delay_timer;
                    } // set Vx = delay timer value
                    0x0A => {
                        let mut key: Option<u8> = None;

                        // attempt to find a held key
                        while key.is_none() {
                            for (i, &k) in self.keypad.iter().enumerate() {
                                if k {
                                    key = Some(i as u8);
                                    break; // break out early
                                }
                            }
                        }

                        self.vregister[reg] = key.unwrap();
                    } // halt the program and wait for a key press, store the value of the key in Vx
                    0x15 => {
                        self.delay_timer = self.vregister[reg];
                    } // set delay timer = Vx
                    0x18 => {
                        self.sound_timer = self.vregister[reg];
                    } // set the sound timer = Vx
                    0x1E => {
                        self.index_register = self.index_register + (self.vregister[reg] as u16);
                    } // set I = I + Vx
                    0x29 => {
                        let font_start = 0x50; // where the fonts start in memory
                        let font_size = 5; // 5 bytes wide

                        // set index register to where the digit stored in Vx starts
                        // (where the fonts start + which digit * fonts size to jump to the correct one)
                        self.index_register =
                            (font_start + (self.vregister[reg] as usize) * font_size) as u16;
                    } // set I = location of sprite for digit Vx
                    0x33 => {
                        let value = self.vregister[reg];

                        self.memory[self.index_register as usize] = value / 100;
                        self.memory[(self.index_register + 1) as usize] = (value / 10) % 10;
                        self.memory[(self.index_register + 2) as usize] = value % 10;
                    } // store BCD representation of Vx in memory locations I, I+1 and I+2
                    0x55 => {
                        let range = reg as usize;

                        // loop and include Vx register itself
                        for i in 0..=range {
                            self.memory[self.index_register as usize + (i as usize)] =
                                self.vregister[i as usize];
                        }
                    } // store registers V0 through Vx in memory starting at location I
                    0x65 => {
                        let range = reg as usize;

                        // loop and include Vx register itself
                        for i in 0..=range {
                            self.vregister[i as usize] =
                                self.memory[(self.index_register as usize) + (i as usize)];
                        }
                    } // read registers V0 through Vx from memory starting at location I
                    _ => println!("unknown last two nibbles of 0xFxxx"),
                }

                self.program_counter += 0x02;
            }
            _ => {
                if self.debug {
                    println!("opcode 0x{:x} not yet implemented", opcode)
                }
            }
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        println!("PC: {:04X}, Opcode: {:04X}", self.program_counter, opcode);
    }
}
