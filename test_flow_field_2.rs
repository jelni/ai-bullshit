use std::collections::{HashMap, VecDeque};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

pub fn calculate_next_head_dir(head: Point, dir: Direction) -> Point {
    match dir {
        Direction::Up => Point {
            x: head.x,
            y: head.y.saturating_sub(1),
        },
        Direction::Down => Point {
            x: head.x,
            y: head.y.saturating_add(1),
        },
        Direction::Left => Point {
            x: head.x.saturating_sub(1),
            y: head.y,
        },
        Direction::Right => Point {
            x: head.x.saturating_add(1),
            y: head.y,
        },
    }
}

fn main() {
    let mut flow_field = HashMap::new();
    let mut queue = VecDeque::new();
    let targets = vec![Point { x: 2, y: 2 }];

    for &t in &targets {
        queue.push_back((t, 0_u16));
    }

    let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];

    while let Some((curr, dist)) = queue.pop_front() {
        for &d in &dirs {
            let opposite = match d {
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
            };

            let final_prev = calculate_next_head_dir(curr, opposite);

            if final_prev.x <= 0 || final_prev.x >= 19 || final_prev.y <= 0 || final_prev.y >= 19 {
                continue;
            }

            if !flow_field.contains_key(&final_prev) && !targets.contains(&final_prev) {
                // Simplified checks
                flow_field.insert(final_prev, d);
                queue.push_back((final_prev, dist + 1));
            }
        }
    }

    println!("Flow field entry for (10,10): {:?}", flow_field.get(&Point { x: 10, y: 10 }));
}
