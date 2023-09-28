use std::ops::*;

use sdl2::rect::Point;
use crate::WINDOW_DIMENSIONS;

//A 2-dimenstional vector, designed to work with SDL2
#[derive(Clone, Copy, Debug)]
pub struct Vec2
{
    x: f64,
    y: f64,
}

impl Vec2
{
    pub fn new(x: f64, y: f64) -> Vec2
    {
        Vec2{x, y}
    }

    pub fn zero() -> Vec2
    {
        Vec2{x: 0.0, y: 0.0}
    }

    pub fn x(&self) -> f64
    {
        self.x
    }

    pub fn y(&self) -> f64
    {
        self.y
    }

    pub fn len_squared(&self) -> f64
    {
        ((self.x as f64).powf(2.0_f64)+(self.y as f64).powf(2.0_f64)) as f64
    }
    pub fn len(&self) -> f64
    {
        (self.len_squared() as f64).powf(0.5_f64)
    }
    pub fn dot(fst: &Vec2, snd: &Vec2) -> f64
    {
        fst.x*snd.x+fst.y*snd.y
    }
    pub fn perpendicular(&self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }
    pub fn polar(&self) -> (f64, f64) {
        let mag = self.len();
        let angle = match self.x {
            x if x >= 0.0 => (self.y/self.x).atan(),
            x if x < 0.0 => (self.y/self.x).atan()+std::f64::consts::PI,
            _ => unreachable!(),
        };
        (mag, angle)
    }
    pub fn from_polar(mag: f64, angle: f64) -> Vec2 {
        Vec2{ x: angle.cos()*mag, y: angle.sin()*mag }
    }
    pub fn rotate(&self, axis: &Vec2, angle: f64) -> Vec2 {
        let relative_pos = *self-*axis;
        let x = relative_pos.x();
        let y = relative_pos.y();
        let cos = angle.cos();
        let sin = angle.sin();
        *axis+Vec2{x: x*cos-y*sin, y: x*sin+y*cos}

    }
}

impl Add for Vec2
{
    type Output = Vec2;

    fn add(self, other: Self) -> Self::Output
    {
        Vec2{x: self.x+other.x, y: self.y+other.y}
    }
}

impl Sub for Vec2
{
    type Output = Vec2;

    fn sub(self, other: Self) -> Self::Output
    {
        Vec2{x: self.x-other.x, y: self.y-other.y}
    }
}

impl Mul<f64> for Vec2
{
    type Output = Vec2;

    fn mul(self, other: f64) -> Self::Output
    {
        Vec2{x: self.x*other, y: self.y*other}
    }
}

impl Div<f64> for Vec2
{
    type Output = Vec2;

    fn div(self, other: f64) -> Self::Output
    {
        Vec2{x: self.x/other, y: self.y/other}
    }
}

impl AddAssign for Vec2
{
    fn add_assign(&mut self, other: Self)
    {
        *self = *self + other
    }
}

impl PartialEq for Vec2
{
    fn eq(&self, other: &Self) -> bool
    {
        ((self.x-other.x).abs() < 1.0e-6) && ((self.y-other.y).abs() < 1.0e-6)
    }
}

impl From<Vec2> for Point
{
    fn from(vec2: Vec2) -> Point
    {
        let x = vec2.x;
        let y = vec2.y;
        let x = (x + (WINDOW_DIMENSIONS.0/2) as f64) as i32;
        let y = (-y + (WINDOW_DIMENSIONS.1/2) as f64) as i32;
        Point::new(x, y)
    }
}

impl From<Point> for Vec2
{
    fn from(point: Point) -> Vec2
    {
        let x = point.x;
        let y = point.y;
        let x = (x - (WINDOW_DIMENSIONS.0/2) as i32) as f64;
        let y = (-y + (WINDOW_DIMENSIONS.1/2) as i32) as f64;
        Vec2::new(x, y)
    }
}

