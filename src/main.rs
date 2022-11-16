#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::usize;

use array2d::Array2D;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 900;
const HEIGHT: u32 = 450;

const RED: [u8; 4] = [0xdd, 0x40, 0x3a, 0xff];
const GREEN: [u8; 4] = [0x69, 0x7a, 0x21, 0xff];
const BLUE: [u8; 4] = [0x05, 0x29, 0x9e, 0xff];
const GREY: [u8; 4] = [0x3e, 0x42, 0x4b, 0xff];
const WHITE: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

fn draw_tile(frame: &mut [u8], pos_x: usize, pos_y: usize, width: usize, color: [u8; 4]) {
    let row_len: usize = WIDTH.try_into().unwrap();
    for i in 1..width + 1 {
        let start = ((i + pos_y) * row_len + pos_x) * 4;
        let end = start + 4 * width;
        for pixel in frame[start..end].chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }
}

struct Map {
    map_array: Array2D<char>,
    width: u16,
    height: u16,
    screen_x: u32,
    screen_y: u32,
    tile_size: u8,
    player_x: f64,
    player_y: f64,
}

impl Map {
    fn init(map_str: String, screen_x: u32, screen_y: u32, tile_size: u8) -> Map {
        let rows: Vec<_> = map_str
            .lines()
            .map(|line| line.trim_start().chars().collect())
            .collect();
        let map_array = Array2D::from_rows(&rows);
        Map {
            map_array: Array2D::from_rows(&rows),
            width: map_array.num_columns() as u16,
            height: map_array.num_rows() as u16,
            screen_x,
            screen_y,
            tile_size,
            player_x: (map_array.num_columns() / 2) as f64,
            player_y: 1.0,
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        //Draw minimap
        for (i, row_iter) in self.map_array.rows_iter().enumerate() {
            for (j, tile) in row_iter.enumerate() {
                let pos_x: usize = (self.screen_x + (self.tile_size as usize * j) as u32)
                    .try_into()
                    .unwrap();
                let pos_y: usize = (self.screen_y + (self.tile_size as usize * i) as u32)
                    .try_into()
                    .unwrap();
                let color = if *tile == 'r' {
                    RED
                } else if *tile == 'g' {
                    GREEN
                } else if *tile == 'b' {
                    BLUE
                } else {
                    GREY
                };
                draw_tile(frame, pos_x, pos_y, self.tile_size as usize, color);
            }
        }
    }

    fn draw_player_on_map(&self, frame: &mut [u8]) {
        //Draw player dot
        let p_screen_x = self.screen_x
            + (self.player_x as u32 * self.tile_size as u32) as u32
            + ((self.width / 2) * self.tile_size as u16) as u32;
        let p_screen_y = self.screen_y + self.height as u32 * self.tile_size as u32
            - self.player_y as u32
            - self.tile_size as u32;
        draw_tile(frame, p_screen_x as usize, p_screen_y as usize, 1, WHITE);
    }
}

struct Game {
    map: Map,
}

impl Game {
    fn init(map_str: String, map_x: u32, map_y: u32, map_tile_size: u8) -> Game {
        let map = Map::init(map_str, map_x, map_y, map_tile_size);
        Game { map }
    }
}

fn draw_frame(frame: &mut [u8], scene: &Game) {
    scene.map.draw_player_on_map(frame);
    println!("Player ({}, {})", scene.map.player_x, scene.map.player_y);
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let map_str: String = String::from(
        "rrrrrrrrrrrr
         r__________r
         r__b____g__r
         r__________r
         r__________r
         r__g____b__r
         r__________r
         rrrrrrrrrrrr",
    );

    let mut scene = Game::init(map_str, 0, 0, 20);

    scene.map.draw(pixels.get_frame_mut());

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            draw_frame(pixels.get_frame_mut(), &scene);
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        //let next_frame = last_frame.elapsed();
        //println!("FPS {}", (1 / next_frame.as_nanos()) / 10 ^ 9);

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_held(VirtualKeyCode::Up) {
                scene.map.player_y = scene.map.player_y + 0.1;
            }

            if input.key_held(VirtualKeyCode::Down) {
                scene.map.player_y = scene.map.player_y - 0.1;
            }

            if input.key_held(VirtualKeyCode::Left) {
                scene.map.player_x = scene.map.player_x - 0.1;
            }

            if input.key_held(VirtualKeyCode::Right) {
                scene.map.player_x = scene.map.player_x + 0.1;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            window.request_redraw();
        }

        //window.request_redraw()
    });
}
