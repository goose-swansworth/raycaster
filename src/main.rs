#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::time::Instant;
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

const DELTAMOVE: f64 = 100000.0;
const MAP_TILE_SIZE: u32 = 10;

fn draw_tile(frame: &mut [u8], pos_x: usize, pos_y: usize, width: usize, color: [u8; 4]) {
    let row_len: usize = WIDTH.try_into().unwrap();
    for i in 0..width {
        let start = ((i + pos_y) * row_len + pos_x) * 4;
        let end = start + 4 * width;
        for pixel in frame[start..end].chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }
}

fn draw_scanline(frame: &mut [u8], pos_x: usize, width: usize, length: usize, color: [u8; 4]) {
    let row_len = WIDTH as usize;
    for i in 0..length {
        let start = ((i + (HEIGHT as usize - length) / 2) * row_len + pos_x - width / 2) * 4;
        let end = start + 4 * width;
        for pixel in frame[start..end].chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        } 
    }
}

struct Map {
    map_array: Array2D<char>,
    width: u32,
    height: u32,
    screen_x: u32,
    screen_y: u32,
    tile_size: u32,
    player_x: f64,
    player_y: f64,
}

impl Map {
    fn init(map_str: String, screen_x: u32, screen_y: u32, tile_size: u32) -> Map {
        let rows: Vec<_> = map_str
            .lines()
            .map(|line| line.trim_start().chars().collect())
            .collect();
        let map_array = Array2D::from_rows(&rows);
        Map {
            map_array: Array2D::from_rows(&rows),
            width: map_array.num_columns() as u32,
            height: map_array.num_rows() as u32,
            screen_x,
            screen_y,
            tile_size,
            player_x: (map_array.num_columns() as f64) / 2.0,
            player_y: 1.0,
        }
    }

    fn in_moveable(&self, new_x: f64, new_y: f64) -> bool {
        let (i, j) = (new_y as usize, new_x as usize);
        if i >= self.height as usize || j >= self.width as usize {
            return false;
        }
        self.map_array[(i, j)] == '_'
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
        draw_tile(
            frame,
            (self.player_x * self.tile_size as f64) as usize + self.screen_x as usize,
            (self.player_y * self.tile_size as f64) as usize + self.screen_y as usize,
            1,
            WHITE,
        );
    }
}

struct Game {
    map: Map,
}

impl Game {
    fn init(map_str: String, map_x: u32, map_y: u32, map_tile_size: u32) -> Game {
        let map = Map::init(map_str, map_x, map_y, map_tile_size);
        Game { map }
    }
}

fn draw_frame(frame: &mut [u8], scene: &Game) {
    scene.map.draw(frame);
    scene.map.draw_player_on_map(frame);
    for i in 0..100 {
        draw_scanline(frame, 1 + i, 1, i + 1, RED);
    }
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
        "rrrrr
         r___r
         r_b_r
         r___r
         rrrrr",
    );

    let mut scene = Game::init(map_str, 0, (HEIGHT-1) - 5 * MAP_TILE_SIZE, MAP_TILE_SIZE);
    
    
    event_loop.run(move |event, _, control_flow| {
        let last_frame = Instant::now();
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

        let next_frame_time = last_frame.elapsed();
        
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            
            let move_amount = next_frame_time.as_secs_f64() * DELTAMOVE;
            //println!("FPS: {}", 1.0 / move_amount);

            if input.key_held(VirtualKeyCode::Up) {
                let (new_x, new_y) = (scene.map.player_x, scene.map.player_y + move_amount);
                if scene.map.in_moveable(new_x, new_y) {
                    scene.map.player_y = new_y;
                    println!("Player ({}, {})", scene.map.player_x, scene.map.player_y);
                }
            }

            if input.key_held(VirtualKeyCode::Down) {
                let (new_x, new_y) = (scene.map.player_x, scene.map.player_y - move_amount);
                if scene.map.in_moveable(new_x, new_y) {
                    scene.map.player_y = new_y;
                    println!("Player ({}, {})", scene.map.player_x, scene.map.player_y);
                }
            }

            if input.key_held(VirtualKeyCode::Left) {
                let (new_x, new_y) = (scene.map.player_x - move_amount, scene.map.player_y);
                if scene.map.in_moveable(new_x, new_y) {
                    scene.map.player_x = new_x;
                    println!("Player ({}, {})", scene.map.player_x, scene.map.player_y);
                }
            }

            if input.key_held(VirtualKeyCode::Right) {
                let (new_x, new_y) = (scene.map.player_x + move_amount, scene.map.player_y);
                if scene.map.in_moveable(new_x, new_y) {
                    scene.map.player_x = new_x;
                    println!("Player ({}, {})", scene.map.player_x, scene.map.player_y);
                }
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
