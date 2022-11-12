#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::usize;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 900;
const HEIGHT: u32 = 450;

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
    let red = [0xff, 0x00, 0x00, 0xff];
    let green = [0x00, 0xff, 0x00, 0xff];
    let blue = [0x00, 0x00, 0xff, 0xff];

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            draw_tile(pixels.get_frame_mut(), 100, 10, 10, red);
            draw_tile(pixels.get_frame_mut(), 200, 50, 20, green);
            draw_tile(pixels.get_frame_mut(), 100, 60, 15, blue);
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            window.request_redraw();
        }
    });
}
