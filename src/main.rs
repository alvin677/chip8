use minifb::{Key, Window, WindowOptions};

mod chip8;
use chip8::CHIP8;

fn main() {
    // initialize the cpu
    let mut chip8 = CHIP8::new();
    chip8.debug = false;

    // get cli game argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("first argument should be a game!");
        return;
    }

    let game = &args[1];
    println!("{}", game);

    // load rom to cpu memory
    chip8.load_fonts();
    chip8.load_rom(game);
    println!("{:x?}", chip8.memory);

    const SCALE_WIDTH: usize = 1024; // scaled 16 times
    const SCALE_HEIGHT: usize = 512;

    const CHIP8_WIDTH: usize = 64; // original
    const CHIP8_HEIGHT: usize = 32;

    const SCALE: usize = 16;

    let mut window = Window::new(
        "Test - ESC to exit",
        SCALE_WIDTH,
        SCALE_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.set_target_fps(240);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // we use chip8.display to calculate and scale and store in the buffer variable
        let mut buffer: Vec<u32> = vec![0xFF000000; SCALE_WIDTH * SCALE_HEIGHT];

        for y in 0..CHIP8_HEIGHT {
            for x in 0..CHIP8_WIDTH {
                let pixel = chip8.display[y * CHIP8_WIDTH + x];
                if pixel == 1 {
                    for dy in 0..SCALE {
                        for dx in 0..SCALE {
                            let buffer_index = (y * SCALE + dy) * SCALE_WIDTH + (x * SCALE + dx);
                            buffer[buffer_index] = 0xFFFFFFFF;
                        }
                    }
                }
            }
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, SCALE_WIDTH, SCALE_HEIGHT)
            .unwrap();

        chip8.keypad = [false; 16]; // clear keys
        let keys = window.get_keys();
        for key in keys {
            match key {
                Key::Key1 => {
                    chip8.keypad[0x1] = true;
                }
                Key::Key2 => {
                    chip8.keypad[0x2] = true;
                }
                Key::Key3 => {
                    chip8.keypad[0x3] = true;
                }
                Key::Key4 => {
                    chip8.keypad[0x4] = true;
                }
                Key::Key5 => {
                    chip8.keypad[0x5] = true;
                }
                Key::Key6 => {
                    chip8.keypad[0x6] = true;
                }
                Key::Key7 => {
                    chip8.keypad[0x7] = true;
                }
                Key::Key8 => {
                    chip8.keypad[0x8] = true;
                }
                Key::Key9 => {
                    chip8.keypad[0x9] = true;
                }
                Key::Key0 => {
                    chip8.keypad[0x0] = true;
                }
                Key::A => {
                    chip8.keypad[0xA] = true;
                }
                Key::B => {
                    chip8.keypad[0xB] = true;
                }
                Key::C => {
                    chip8.keypad[0xC] = true;
                }
                Key::D => {
                    chip8.keypad[0xD] = true;
                }
                Key::E => {
                    chip8.keypad[0xE] = true;
                }
                Key::F => {
                    chip8.keypad[0xF] = true;
                }
                _ => {}
            }
        }

        chip8.cycle();
    }
}
