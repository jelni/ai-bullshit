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

// Another method without allocating dynamic vec: just iterating over empty spaces
// To keep exactly the original logic (random selection via RNG), we must use boolean grid
// Let's re-verify the vec![false; w*h] approach. It allocs every time. We can just change generate_obstacles and generate_food.

fn generate_food_vec_bool(width: u16, height: u16, snake: &Snake, obstacles: &[Point]) -> Point {
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

fn main() {
    let width = 50;
    let height = 50;
    let mut snake = Snake { body: VecDeque::new() };
    for x in 1..width-3 {
        for y in 1..height-3 {
            snake.body.push_back(Point { x, y });
        }
    }

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let p = generate_food_baseline(width, height, &snake, &[]);
        std::hint::black_box(p);
    }
    println!("Baseline took: {:?}", start.elapsed());

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let p = generate_food_vec_bool(width, height, &snake, &[]);
        std::hint::black_box(p);
    }
    println!("Optimized took: {:?}", start.elapsed());
}
