#![allow(dead_code)]

use crate::vec2::Vec2;
use std::collections::VecDeque;

use crate::{WINDOW_DIMENSIONS, SNAKE_VELOCITY, SNAKE_TURNING_VELOCITY, TRAIL_BUFFER_TIME};

extern crate rand;
use rand::thread_rng;
use rand::Rng;

use sdl2::rect::Point;

//Radius of the drawn snake. In pixels. Is i32 due to usage in display function.
pub const SNAKE_RADIUS: i32 = 3;

//A snake. Orientation is in radians, and is counter-clockwise from the right direction
#[derive(Debug)]
pub struct Snake {
    position: Vec2,
    orientation: f64,
    colour: Colour,
}
impl Snake {
    pub fn new(position: Vec2, orientation: f64, colour: Colour) -> Snake {
        Snake{ position, orientation, colour }
    }
    #[inline]
    pub fn colour(&self) -> Colour {
        self.colour
    }
    #[inline]
    pub fn position(&self) -> Vec2 {
        self.position
    }

    //Moves the snake by one frame
    pub fn translate(&mut self) {
        let delta_x = self.orientation.cos()*SNAKE_VELOCITY;
        let delta_y = self.orientation.sin()*SNAKE_VELOCITY;

        self.position += Vec2::new(delta_x, delta_y);
    }

    //Turns the snake by one frame
    pub fn turn(&mut self, direction: TurnDirection) {
        let direction = match direction {
            TurnDirection::Left => 1_f64,
            TurnDirection::Right => -1_f64,
        };
        self.orientation += direction*SNAKE_TURNING_VELOCITY;
    }
    pub fn draw(&self) -> Vec<(Point, Colour)> {
        let mut ret_val = Vec::new();

        //Defines out a bounding rectangle, then checks if every pixel in there is within snake
        //radius to draw circle.
        for x in -SNAKE_RADIUS..SNAKE_RADIUS {
            for y in -SNAKE_RADIUS..SNAKE_RADIUS {
                let distance = Vec2::new(x as f64, y as f64).len();
                if distance < SNAKE_RADIUS as f64 {
                    let vec2_position = self.position + Vec2::new(x as f64, y as f64);
                    ret_val.push((Point::from(vec2_position), self.colour));
                }
            }
        }
        ret_val
    }

    //Adds trails to the bitmap. Also returns a list of points for easier rendering. This is a bit
    //inneficient memory wise but saves loads of time on rendering trails
    pub fn add_trail(&self, trails: &mut Vec<Vec<Option<Colour>>>) {
        let points = self.draw();
        for (point, _) in points {
            let x = point.x();
            let y = point.y();

            //If trail is out of bounds, don't add it
            if x < 0 || y < 0 || x >= WINDOW_DIMENSIONS.0 || y >= WINDOW_DIMENSIONS.1 {
                continue;
            }
            match trails[x as usize][y as usize] {
                Some(_) => (),
                None => trails[x as usize][y as usize] = Some(self.colour),
            }
        }
    }

    //Adds trails to the bitmap. Also returns a list of points for easier rendering. This is a bit
    //inneficient memory wise but saves loads of time on rendering trails
    pub fn add_trail_to_queue(&self, frame: u64, trail_queue: &mut VecDeque<(Point, Colour, u64)>) {
        let points = self.draw();
        for (point, colour) in points {
            let x = point.x();
            let y = point.y();

            //If trail is out of bounds, don't add it
            if x < 0 || y < 0 || x >= WINDOW_DIMENSIONS.0 || y >= WINDOW_DIMENSIONS.1 {
                continue;
            }
            trail_queue.push_back((point, colour, frame));
        }
    }

    //Detects if a snake has hit a trail.
    pub fn detect_trail_hit(&self, trails: &Vec<Vec<Option<Colour>>>) -> bool {
        let position = Point::from(self.position);
        if position.x() < 0 || position.y() < 0 || position.x() >= WINDOW_DIMENSIONS.0 || position.y() >= WINDOW_DIMENSIONS.1 {
            return false;
        }
        if let Some(_) = trails[position.x() as usize][position.y() as usize] {
            true
        } 
        else {
            false
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TurnDirection {
    Left,
    Right,
}

//Has a brightness between 0 and 255.
#[derive(Clone, Copy, Debug)]
pub enum Colour {
    Red(u8),
    Green(u8),
    Blue(u8),
}
impl std::fmt::Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::Red(i) => format!("Red({})", i),
            Self::Green(i) => format!("Green({})", i),
            Self::Blue(i) => format!("Blue({})", i),
        };
        write!(f, "{}", string)
    }
}
impl Colour {
    pub fn set_brightness(&mut self, brightness: u8) {
        *self = match self {
            Self::Red(_) => Self::Red(brightness),
            Self::Green(_) => Self::Green(brightness),
            Self::Blue(_) => Self::Blue(brightness),
        }
    }
}
impl From<Colour> for sdl2::pixels::Color {
    fn from(colour: Colour) -> Self {
        match colour {
            Colour::Red(brightness) => sdl2::pixels::Color::RGB(brightness, 0, 0),
            Colour::Green(brightness) => sdl2::pixels::Color::RGB(0, brightness, 0),
            Colour::Blue(brightness) => sdl2::pixels::Color::RGB(0, 0, brightness),
        }
        
    }
}
const COLOURS: [Colour; 3] = [Colour::Red(255), Colour::Green(255), Colour::Blue(255)];

//Randomly generates the requested number of snakes, in the center of the screen
pub fn generate_snakes(count: usize) -> Vec<Snake> {
    let mut rng = thread_rng();
    let mut x: f64;
    let mut y: f64;
    let mut orientation: f64;
    let mut snakes = Vec::with_capacity(count);
    for i in 0..count {
        let corner1 = Vec2::from(Point::new(WINDOW_DIMENSIONS.0/4, WINDOW_DIMENSIONS.1/4));
        let corner2 = Vec2::from(Point::new((WINDOW_DIMENSIONS.0 as f64 *(3_f64/4_f64)) as i32, (WINDOW_DIMENSIONS.1 as f64 * (3_f64/4_f64)) as i32));
        x = rng.gen_range(corner1.x()..corner2.x());
        y = rng.gen_range(corner2.y()..corner1.y());
        orientation = rng.gen_range(0.0..std::f64::consts::TAU);
        snakes.push(Snake::new(Vec2::new(x, y), orientation, COLOURS[i]));
    }
    snakes
}

//Adds trails to the bitmap from the trail queue if it has been long enough. Also returns a list of points for easier rendering. This is a bit inneficient memory wise but saves loads of time on rendering trails
pub fn add_trails_from_buffer(frame: u64, trail_queue: &mut VecDeque<(Point, Colour, u64)>, trails: &mut Vec<Vec<Option<Colour>>>) {
    loop {
        //Checks if it is too early. Elements are ordered chronologically, so if one is too early,
        //all after it are also. Also aborts if trail queue is empty.
        match trail_queue.front() {
            None => break,
            Some((_, _, x)) if x + TRAIL_BUFFER_TIME > frame => break,
            _ => (),
        }
        let (point, colour, _) = trail_queue.pop_front().unwrap();
        match trails[point.x() as usize][point.y() as usize] {
            Some(_) => (),
            None => trails[point.x() as usize][point.y() as usize] = Some(colour),
        }
    }
}
