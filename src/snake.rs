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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_initialization() {
        let start = Point { x: 5, y: 5 };
        let snake = Snake::new(start);

        assert_eq!(snake.body.len(), 3);
        assert_eq!(snake.head(), start);
        assert_eq!(snake.direction, Direction::Up);
        assert_eq!(snake.next_direction, None);
        assert_eq!(snake.body[0], Point { x: 5, y: 5 });
        assert_eq!(snake.body[1], Point { x: 5, y: 6 });
        assert_eq!(snake.body[2], Point { x: 5, y: 7 });
    }

    #[test]
    fn test_snake_move_without_growth() {
        let start = Point { x: 5, y: 5 };
        let mut snake = Snake::new(start);

        let new_head = Point { x: 5, y: 4 };
        snake.move_to(new_head, false);

        assert_eq!(snake.body.len(), 3);
        assert_eq!(snake.head(), new_head);
        assert_eq!(snake.body[0], Point { x: 5, y: 4 });
        assert_eq!(snake.body[1], Point { x: 5, y: 5 });
        assert_eq!(snake.body[2], Point { x: 5, y: 6 });
    }

    #[test]
    fn test_snake_move_with_growth() {
        let start = Point { x: 5, y: 5 };
        let mut snake = Snake::new(start);

        let new_head = Point { x: 5, y: 4 };
        snake.move_to(new_head, true);

        assert_eq!(snake.body.len(), 4);
        assert_eq!(snake.head(), new_head);
        assert_eq!(snake.body[0], Point { x: 5, y: 4 });
        assert_eq!(snake.body[1], Point { x: 5, y: 5 });
        assert_eq!(snake.body[2], Point { x: 5, y: 6 });
        assert_eq!(snake.body[3], Point { x: 5, y: 7 });
    }
}
