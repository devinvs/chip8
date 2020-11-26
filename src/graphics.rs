use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::Sdl;
use sdl2::rect::Rect;
use sdl2::pixels::Color;

use std::ops::{Index, IndexMut};


pub struct Screen {
    screen: [[u8; 64]; 32],
    canvas: Canvas<Window>,
    pixel_size: u32
}

impl Screen {
    pub fn new(sdl_context: &Sdl) -> Screen {
        
        let video_subsystem = sdl_context.video().unwrap();

        let (width, height) = (1280, 640);
        let window = video_subsystem.window("Chip-8 emulator", width, height)
            .position_centered()
            .build()
            .unwrap();
        
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        Screen {
            screen: [[0; 64]; 32],
            canvas,
            pixel_size: width / 64
        }
    }

    pub fn draw(&mut self) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        self.canvas.set_draw_color(Color::WHITE);
        for row in 0..32 {
            for col in 0..64 {
                if self.screen[row][col] == 1 {
                    self.canvas.fill_rect(Rect::new(
                        col as i32 * self.pixel_size as i32,
                        row as i32 * self.pixel_size as i32,
                        self.pixel_size,
                        self.pixel_size
                    )).unwrap();
                }
            }
        }

        self.canvas.present();
    }
}


impl Index<usize> for Screen {
    type Output = [u8; 64];

    fn index(&self, index: usize) -> &[u8; 64] {
        &self.screen[index]
    }
}

impl IndexMut<usize> for Screen {

    fn index_mut(&mut self, index: usize) -> &mut [u8; 64] {
        &mut self.screen[index]
    }
}