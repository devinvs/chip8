use chip8::chip8::Chip8;

fn main() {
    let mut emu = Chip8::new();
    emu.load_game("tetris.rom");

    loop {
        emu.handle_events();
        emu.tick();
        std::thread::sleep(std::time::Duration::from_secs_f64(1.0/60.0));
    }
}
