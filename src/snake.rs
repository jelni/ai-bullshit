use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Serialize, Deserialize)]
pub struct Snake {
    pub body: VecDeque<Point>,
    #[serde(skip)]
    pub body_map: HashMap<Point, usize>,
    pub direction: Direction,
    #[serde(default)]
    pub direction_queue: std::collections::VecDeque<Direction>,
}

impl Snake {
    pub fn new(start: Point) -> Self {
        let mut body = VecDeque::new();
        // Head
        body.push_back(start);
        // Body segments below head (since we face UP)
        body.push_back(Point {
            x: start.x,
            y: start.y + 1,
        });
        body.push_back(Point {
            x: start.x,
            y: start.y + 2,
        });

        let mut body_map = HashMap::with_capacity(3);
        for p in &body {
            *body_map.entry(*p).or_insert(0) += 1;
        }

        Self {
            body,
            body_map,
            direction: Direction::Up,
            direction_queue: std::collections::VecDeque::new(),
        }
    }

    pub fn head(&self) -> Point {
        *self.body.front().expect("Snake must have a head")
    }

    pub fn move_to(&mut self, new_head: Point, grow: bool) {
        self.body.push_front(new_head);
        *self.body_map.entry(new_head).or_insert(0) += 1;
        #[expect(clippy::collapsible_if, reason = "stable rust")]
        if !grow {
            if let Some(tail) = self.body.pop_back() {
                if let Some(count) = self.body_map.get_mut(&tail) {
                    *count -= 1;
                    if *count == 0 {
                        self.body_map.remove(&tail);
                    }
                }
            }
        }
    }

    pub fn rebuild_map(&mut self) {
        self.body_map.clear();
        for p in &self.body {
            *self.body_map.entry(*p).or_insert(0) += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_new() {
        let start = Point { x: 5, y: 5 };
        let snake = Snake::new(start);

        assert_eq!(snake.body.len(), 3);
        assert_eq!(snake.body[0], start);
        assert_eq!(
            snake.body[1],
            Point {
                x: start.x,
                y: start.y + 1
            }
        );
        assert_eq!(
            snake.body[2],
            Point {
                x: start.x,
                y: start.y + 2
            }
        );
        assert_eq!(snake.direction, Direction::Up);
        assert!(snake.direction_queue.is_empty());
    }

    #[test]
    fn test_snake_new_origin() {
        let start = Point { x: 0, y: 0 };
        let snake = Snake::new(start);

        assert_eq!(snake.body.len(), 3);
        assert_eq!(snake.body[0], start);
        assert_eq!(snake.body[1], Point { x: 0, y: 1 });
        assert_eq!(snake.body[2], Point { x: 0, y: 2 });
        assert_eq!(snake.direction, Direction::Up);
        assert!(snake.direction_queue.is_empty());
    }

    #[test]
    fn test_snake_new_large_coordinates() {
        let start = Point {
            x: u16::MAX - 2,
            y: u16::MAX - 2,
        };
        let snake = Snake::new(start);

        assert_eq!(snake.body.len(), 3);
        assert_eq!(snake.body[0], start);
        assert_eq!(
            snake.body[1],
            Point {
                x: u16::MAX - 2,
                y: u16::MAX - 1
            }
        );
        assert_eq!(
            snake.body[2],
            Point {
                x: u16::MAX - 2,
                y: u16::MAX
            }
        );
        assert_eq!(snake.direction, Direction::Up);
        assert!(snake.direction_queue.is_empty());
    }
}
