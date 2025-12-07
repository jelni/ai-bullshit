use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Serialize, Deserialize)]
pub struct Snake {
    pub body: VecDeque<Point>,
    pub direction: Direction,
    pub next_direction: Option<Direction>,
}

impl Snake {
    pub fn new(start: Point) -> Self {
        let mut body = VecDeque::new();
        // Head
        body.push_back(start);
        // Body segments below head (since we face UP)
        body.push_back(Point { x: start.x, y: start.y + 1 });
        body.push_back(Point { x: start.x, y: start.y + 2 });
        Self {
            body,
            direction: Direction::Up,
            next_direction: None,
        }
    }

    pub fn head(&self) -> Point {
        *self.body.front().unwrap()
    }

    pub fn move_to(&mut self, new_head: Point, grow: bool) {
        self.body.push_front(new_head);
        if !grow {
            self.body.pop_back();
        }
    }
}
