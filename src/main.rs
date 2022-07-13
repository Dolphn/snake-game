use image::{ImageBuffer, Rgba};
use pixels::{Pixels, SurfaceTexture};
use rand::Rng;
use std::time;
use std::time::{Duration, Instant};
use winit::{
    dpi::LogicalSize,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WINDOW_HEIGHT_PIXEL: i32 = 10;
const WINDOW_WIDTH_PIXEL: i32 = 20;
const PIXEL_SIZE: u32 = 50;
const WINDOW_HEIGHT: i32 = WINDOW_HEIGHT_PIXEL * PIXEL_SIZE as i32;
const WINDOW_WIDTH: i32 = WINDOW_WIDTH_PIXEL * PIXEL_SIZE as i32;
const SNAKE_COLOR: [u8; 4] = [00, 0xff, 00, 0xff];
const FRUIT_COLOR: [u8; 4] = [0xff, 60, 60, 0xff];
const BACKGROUND_COLOR: [u8; 4] = [0x20, 0x20, 0x20, 0xff];
const STEP_TIME: Duration = Duration::new(0, 80000000);

fn main() {
    println!("Welcome!");
    run();
}

fn draw(frame: &mut [u8], fruit: Pixel, snake: &Vec<Pixel>) {
    //print!("Snake size: {}", snake.len());
    for (_i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        pixel.copy_from_slice(&BACKGROUND_COLOR);
    }

    let mut image = ImageBuffer::<image::Rgba<_>, _>::from_raw(
        WINDOW_WIDTH as u32,
        WINDOW_HEIGHT as u32,
        frame,
    )
    .unwrap();

    for elem in snake {
        let x = elem.position_x;
        let y = elem.position_y;
        // imageproc::drawing::draw_filled_rect(&image, imageproc::rect::Rect::at(x,y).of_size(PIXEL_SIZE, PIXEL_SIZE), Rgba(SNAKE_COLOR));
        imageproc::drawing::draw_filled_rect_mut(
            &mut image,
            imageproc::rect::Rect::at(x * PIXEL_SIZE as i32, y * PIXEL_SIZE as i32)
                .of_size(PIXEL_SIZE, PIXEL_SIZE),
            Rgba(SNAKE_COLOR),
        );
    }
    imageproc::drawing::draw_filled_rect_mut(
        &mut image,
        imageproc::rect::Rect::at(
            fruit.position_x * PIXEL_SIZE as i32,
            fruit.position_y * PIXEL_SIZE as i32,
        )
        .of_size(PIXEL_SIZE, PIXEL_SIZE),
        Rgba(FRUIT_COLOR),
    );
}

fn run() {
    let mut rand = rand::thread_rng();
    let mut snake: Vec<Pixel> = Vec::new();
    let x = rand.gen_range(0..WINDOW_WIDTH_PIXEL);
    let y = rand.gen_range(0..WINDOW_HEIGHT_PIXEL);
    let head = Pixel {
        position_x: x,
        position_y: y,
    };
    snake.push(head);
    let mut dir = (0,0);

    let mut fruit = loop {
        let x = rand.gen_range(0..WINDOW_WIDTH_PIXEL);
        let y = rand.gen_range(0..WINDOW_HEIGHT_PIXEL);
        let fruit = Pixel {
            position_x: x,
            position_y: y,
        };
        if head != fruit {
            break fruit;
        }
    };

    let event_loop = EventLoop::new();
    let log_size = LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let window = WindowBuilder::new()
        .with_title("Snake Game")
        .with_inner_size(log_size)
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, surface_texture).unwrap()
    };

    let mut time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        //*control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                draw(pixels.get_frame(), fruit, &snake);
                pixels.render().unwrap();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::W),
                                ..
                            },
                        ..
                    },
                ..
            } => dir = (0, -1),
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::A),
                                ..
                            },
                        ..
                    },
                ..
            } => dir = (-1, 0),
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::S),
                                ..
                            },
                        ..
                    },
                ..
            } => dir = (0, 1),
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::D),
                                ..
                            },
                        ..
                    },
                ..
            } => dir = (1, 0),
            _ => (),
        }
        let (stamp, res) = timer(time);
        if res {
            fruit = one_step(&mut snake, dir, fruit, control_flow);
            window.request_redraw();
            time = stamp;
        } else {
            *control_flow = ControlFlow::WaitUntil(
                Instant::now() + (STEP_TIME - Instant::now().duration_since(stamp)),
            );
        }
    });
}

fn eat_fruit(snake: &mut Vec<Pixel>) -> Pixel {
    let new_tail = Pixel {
        position_x: snake.first().unwrap().position_x,
        position_y: snake.first().unwrap().position_y,
    };
    snake.insert(1, new_tail);
    let mut rand = rand::thread_rng();

    loop {
        let x = rand.gen_range(0..WINDOW_WIDTH_PIXEL);
        let y = rand.gen_range(0..WINDOW_HEIGHT_PIXEL);
        let pixel = Pixel {
            position_x: x,
            position_y: y,
        };
        if !snake.contains(&pixel) {
            return pixel;
        }
    }
}

fn one_step(snake: &mut Vec<Pixel>, direction: (i32, i32), mut fruit_pos: Pixel, flow: &mut ControlFlow) -> Pixel {
    let new_head_x = snake[0].position_x + direction.0;
    let new_head_y = snake[0].position_y + direction.1;
    if fruit_pos.position_x == new_head_x && fruit_pos.position_y == new_head_y {
        fruit_pos = eat_fruit(snake);

        snake[0].position_x = new_head_x;
        snake[0].position_y = new_head_y;
    } else {
        for no in (1..snake.len()).rev() {
            snake[no].position_x = snake[no - 1].position_x;
            snake[no].position_y = snake[no - 1].position_y;
            if snake[no].position_x == new_head_x && snake[no].position_y == new_head_y {
                kill(flow, "Snake ate itself!");
                //panic!("Snake ate itself!");
                /* println!(
                    "Head: x:{} y:{}, Tail: x:{} y{}",
                    snake[0].position_x,
                    snake[0].position_y,
                    snake[1].position_x,
                    snake[1].position_y
                ); */
            }
        }
        snake[0].position_x = new_head_x;
        snake[0].position_y = new_head_y;
    }
    if new_head_x >= WINDOW_WIDTH_PIXEL
        || new_head_y >= WINDOW_HEIGHT_PIXEL
        || new_head_x < 0
        || new_head_y < 0
    {
        kill(flow, "Snake crawled into wall!");
        //panic!("Snake crawled into wall!")
    }
    fruit_pos
}

#[derive(PartialEq, Clone, Copy)]
pub struct Pixel {
    pub position_x: i32,
    pub position_y: i32,
}

fn timer(last: time::Instant) -> (time::Instant, bool) {
    let now = Instant::now();
    if now.duration_since(last) >= STEP_TIME {
        (now, true)
    } else {
        (last, false)
    }
}

fn kill(flow: &mut ControlFlow, msg: &str) {
    println!("{}",msg);
    *flow = ControlFlow::Exit;
}
/*  let chunked_frame = frame.chunks_exact_mut(4).enumerate();
for elem in snake {
    let begin_x = elem.position_x;
    let begin_y = elem.position_y;
    let mut cur_pos = begin_x *  begin_y;
    for pos_y in 0..PIXEL_SIZE {
        chunked_frame. =
        for pos_x in 0..PIXEL_SIZE {

        }
    }
} */
