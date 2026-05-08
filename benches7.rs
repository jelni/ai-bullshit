use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

pub struct Snake {
    pub body: VecDeque<Point>,
}

fn generate_food_baseline(width: u16, height: u16, snake: &Snake, obstacles: &[Point]) -> Point {
    let mut i = 0;
    loop {
        i = (i + 13) % (width * height);
        let x = i % width;
        let y = i / width;
        let p = Point { x: x.max(1).min(width-2), y: y.max(1).min(height-2) };
        if !snake.body.contains(&p) && !obstacles.contains(&p) {
            return p;
        }
    }
}

fn generate_food_optimized(width: u16, height: u16, snake: &Snake, obstacles: &[Point]) -> Point {
    // Determine bounds to avoid unnecessary large allocation
    // Instead of full w*h vector which takes heap allocation, can we just use a small vector?
    // Wait, w and h are u16, so w*h could be up to 65535 * 65535 which is 4 GB!
    // But realistically, terminal size is at most a few hundreds.
    // 500 x 500 = 250,000 bools = 250 KB.
    // 100 x 100 = 10,000 bools = 10 KB.
    let mut occupied = vec![false; (width as usize) * (height as usize)];
    for p in &snake.body {
        if p.x < width && p.y < height {
            occupied[(p.y as usize) * (width as usize) + (p.x as usize)] = true;
        }
    }
    for p in obstacles {
        if p.x < width && p.y < height {
            occupied[(p.y as usize) * (width as usize) + (p.x as usize)] = true;
        }
    }

    let mut i = 0;
    loop {
        i = (i + 13) % (width * height);
        let x = i % width;
        let y = i / width;
        let p = Point { x: x.max(1).min(width-2), y: y.max(1).min(height-2) };
        if !occupied[(p.y as usize) * (width as usize) + (p.x as usize)] {
            return p;
        }
    }
}

fn generate_food_optimized_hashset(width: u16, height: u16, snake: &Snake, obstacles: &[Point]) -> Point {
    // Instead of full grid, use HashSet which scales with snake + obstacles
    // but we need to derive Hash for Point. Since we can't change Point hash derive right now (wait, we can, it's just snake.rs)
    // Actually snake.rs we can edit. Let's see if Hash is derived.
    let p = Point { x: 0, y: 0 };
    //...
    p
}

fn main() {
}
