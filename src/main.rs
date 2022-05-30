#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod complex;
use complex::*;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const SCALE_FACTOR: f64 = 300.;
const MAX_ITERATIONS: u16 = 100;

struct Mandelbrot {
    scale: f64,
    offsetX: f64,
    offsetY: f64,
}

fn main() -> Result<(), Error> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(16)
        .build_global()
        .unwrap();

    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Mandelbrot")
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
    let mut mandelbrot = Mandelbrot {
        scale: 1.,
        offsetX: 0.,
        offsetY: 0.,
    };

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            mandelbrot.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            mandelbrot.scale += 0.05;
            mandelbrot.update();
            window.request_redraw();
        }
    });
}

impl Mandelbrot {
    fn update(&mut self) {}

    #[inline]
    fn check_point_inside_set(x: f64, y: f64) -> bool {
        let c = Complex::new(x, y);
        let mut z = Complex::new(0., 0.);
        let mut counter = 0;
        loop {
            z = z * z + c;
            counter += 1;
            if z.magnitude_squared() > 4.0 {
                return false;
            }
            if counter > MAX_ITERATIONS {
                return true;
            }
        }
    }

    #[inline]
    fn get_mandelbrot_set(&self) -> Vec<bool> {
        let set: Vec<bool> = vec![false; (WIDTH * HEIGHT) as usize];
        set.par_iter()
            .enumerate()
            .map(|(i, _)| {
                let w = WIDTH as usize;
                let h = HEIGHT as usize;
                let mut x = (i % w) as f64 - w as f64 / 2.;
                let mut y = (i / w) as f64 - h as f64 / 2.;
                x += self.offsetX;
                y += self.offsetY;

                let scale = self.scale;
                x /= SCALE_FACTOR * scale;
                y /= SCALE_FACTOR * scale;
                Mandelbrot::check_point_inside_set(x, y)
            })
            .collect()
    }

    fn draw(&self, frame: &mut [u8]) {
        let set = self.get_mandelbrot_set();
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba = if set[i] {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x00, 0x00, 0x00, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
