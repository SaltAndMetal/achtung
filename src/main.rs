mod snake;
use snake::generate_snakes;
use snake::add_trails_from_buffer;
use snake::TurnDirection;
use snake::Colour;

mod vec2;

extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::rect::Point;

use std::time::Duration;
use std::thread::sleep;

use std::collections::VecDeque;

const WINDOW_DIMENSIONS: (i32, i32) = (500, 500);

//The amount of time to wait between frames. In milliseconds.
const FRAME_GAP: u64 = 1000/60;

//Speed of the snake. In pixels per frame.
const SNAKE_VELOCITY: f64 = 2.2;

//Amount of time before trails take effect. Used to allow snakes to not run into their own heads.
//In frames. Formula was calculated with math to be big enough always. Bit of a nightmare
//const TRAIL_BUFFER_TIME: u64 = ((1_f64/SNAKE_TURNING_VELOCITY * ((2_f64/SNAKE_TURNING_VELOCITY - SNAKE_RADIUS as f64)/2_f64*SNAKE_RADIUS as f64).acos() + 1_f64)/SNAKE_VELOCITY) as u64;
const TRAIL_BUFFER_TIME: u64 = 1;

//Turning speed of the snake. In radians per second. Multiplied by the velocity to keep turning
//circle radius constant
const SNAKE_TURNING_VELOCITY: f64 = 0.03*SNAKE_VELOCITY;

//Distance between the gaps in the snakes. In frames
//const GAP_FREQUENCY: u32 = 100*SNAKE_VELOCITY as u32;

//Length of the gaps. In frames
//const GAP_LENGTH: u32 = 10*SNAKE_VELOCITY as u32;

const EXIT_CODE_INVALID_CLI_USE: i32 = 64;

//The keys to press to turn each player. In format (left, right).
const TURNING_CODES: [(Scancode, Scancode); PLAYER_MAX] = [(Scancode::Num1, Scancode::Q), (Scancode::Left, Scancode::Right), (Scancode::G, Scancode::H)];

//The maximum number of players supported.
const PLAYER_MAX: usize = 3;

//Initialises the sdl2 window
fn sdl2_init() -> (Canvas<Window>, sdl2::EventPump)
{
    let sdl_context = sdl2::init().expect("Error initialising");

    let video_subsystem = sdl_context.video().unwrap();
    
    let window = video_subsystem.window("Achtung, Die Kurve!", WINDOW_DIMENSIONS.0 as u32, WINDOW_DIMENSIONS.1 as u32)
        .position_centered()
        .build()
        .expect("Error creating window");
    
    let canvas = window.into_canvas().build().expect("Error creating canvas");

    let event_pump = sdl_context.event_pump().expect("Error creating event pump");
    
    (canvas, event_pump)
}

fn main() {

    //Reads number of players from command line, giving appropiate errors for invalid inputs
    let args: Vec<String> = std::env::args().collect();
    let players;
    match args.len() {
        1 => { println!("Too few arguments. Please provide number of players."); std::process::exit(EXIT_CODE_INVALID_CLI_USE); },
        2 => { players = args[1].clone(); },
        _ => { println!("Too many arguments."); std::process::exit(EXIT_CODE_INVALID_CLI_USE); },
    }
    let players = match players.parse::<usize>() {
        Ok(n) if n < 2 => { println!("Player count must 2 or greater"); std::process::exit(EXIT_CODE_INVALID_CLI_USE); },
        Ok(n) => n,
        Err(_) => { println!("Player count must be a number 2 or greater"); std::process::exit(EXIT_CODE_INVALID_CLI_USE); },
    };

    let (mut canvas, mut event_pump) = sdl2_init();
    let mut scores = vec![0; players];

    //Outer loop
    'running: loop {
        let mut snakes = generate_snakes(players);
    
        //Bitmap of the screen with trails marked with their colour, and no trail being None.
        let mut trails: Vec<Vec<Option<Colour>>> = vec![vec![None; WINDOW_DIMENSIONS.0 as usize]; WINDOW_DIMENSIONS.1 as usize];
    
        //Queue for trails before they are added to the list of trails. Done in order to prevent snakes
        //from hitting their own heads. Holds the count of the frame it was comitted.
        let mut trail_queue: VecDeque<(Point, Colour, u64)> = VecDeque::new();
    
        //The direction each snake is being turned this frame. None for no turn.
        let mut directions: [Option<TurnDirection>; PLAYER_MAX] = [None; PLAYER_MAX];
    
        //Initialise canvas
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();
    
        let mut frame_count = 0_u64;

        let mut paused = false;

        let mut dead_snakes = vec![false; players];
    
        //Loop for one game
        'game: loop {
            //Detects directions each snake is being turned this frame
            for player in 0..directions.len() {
                let left = event_pump.keyboard_state().is_scancode_pressed(TURNING_CODES[player].0);
                let right = event_pump.keyboard_state().is_scancode_pressed(TURNING_CODES[player].1);
                directions[player] = match (left, right) {
                    (true, true) => None,
                    (true, false) => Some(TurnDirection::Left),
                    (false, true) => Some(TurnDirection::Right),
                    (false, false) => None,
                };
            }
    
            //Handle all events
            for event in event_pump.poll_iter() {
                match event {
    
                    Event::Quit{..} => break 'running,
                    Event::KeyDown{scancode, ..} => if let Some(Scancode::Space) = scancode { paused = !paused },
    
                    _ => (),
                }
            }
            if paused {
                //Render scores
                for (i, (score, snake)) in scores.iter().zip(snakes.iter()).enumerate() {
                    canvas.set_draw_color(snake.colour());
                    for j in 0..*score {
                        let rect = sdl2::rect::Rect::new((j*5) as i32, i as i32*30, 5, 30);
                        canvas.fill_rect(rect).expect("Error drawing scores");
                    }
                }

                //Display snake
                for snake in &snakes {
                    display_points(snake.draw(), &mut canvas);
                }

                canvas.present();

                sleep(Duration::from_millis(FRAME_GAP));
                continue 'game;
            }
            //Move snakes by 1 frame
            for snake in &mut snakes {
                snake.translate();
            }
    
            //Turn snakes by one frame if requested
            for (i, snake_direction) in directions.iter().enumerate() {
                if i >= players {
                    break;
                }
                if !dead_snakes[i] {
                    match snake_direction {
                    None => (),
                    Some(turn_direction) => snakes[i].turn(*turn_direction),
                    }
                }
            }

            //Render scores
            for (i, (score, snake)) in scores.iter().zip(snakes.iter()).enumerate() {
                canvas.set_draw_color(snake.colour());
                for j in 0..*score {
                    let rect = sdl2::rect::Rect::new((j*5) as i32, i as i32*30, 5, 30);
                    canvas.fill_rect(rect).expect("Error drawing scores");
                }
            }

            for (i, snake) in snakes.iter().enumerate() {
                if dead_snakes[i] {
                    continue;
                }
                //Add snake trails to queue
                snake.add_trail_to_queue(frame_count, &mut trail_queue);

                //Display snake
                display_points(snake.draw(), &mut canvas);

                //Detect collisions
                if snake.detect_trail_hit(&trails) {
                    dead_snakes[i] = true;
                }
                //Detect if snake hits wall
                let position = Point::from(snake.position());
                let x = position.x();
                let y = position.y();
                if x < 0 || y < 0 || x >= WINDOW_DIMENSIONS.0 || y >= WINDOW_DIMENSIONS.1 {
                    dead_snakes[i] = true;
                }
            }

            let mut alive_count = 0;
            for dead in &dead_snakes {
                if !dead {
                    alive_count += 1;
                }
            }
            if alive_count <= 1 {
                for i in 0..snakes.len() {
                    if !dead_snakes[i] {
                        scores[i] += 1;
                    }
                }
                sleep(Duration::from_secs(1));
                break 'game;
            }
    
            //Add trails from buffer if enough time has elapsed
            add_trails_from_buffer(frame_count, &mut trail_queue, &mut trails);

    
            canvas.present();
    
            frame_count += 1;
            sleep(Duration::from_millis(FRAME_GAP));
    
        }
    }
}

fn display_points(points: Vec<(Point, Colour)>, canvas: &mut Canvas<Window>) {
    for (point, colour) in points {
        canvas.set_draw_color(colour);
        canvas.draw_point(point).expect(&format!("Error drawing point: {:?}", point));
    }
}
