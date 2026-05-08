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

fn generate_food_vec_of_bools(width: u16, height: u16, snake: &Snake, obstacles: &[Point]) -> Point {
    let mut occupied = vec![false; (width as usize) * (height as usize)];
    for p in &snake.body {
        occupied[(p.y as usize) * (width as usize) + (p.x as usize)] = true;
    }
    for p in obstacles {
        occupied[(p.y as usize) * (width as usize) + (p.x as usize)] = true;
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

// Optimization avoiding `vec!` allocation on every call
fn generate_food_bitvec(width: u16, height: u16, snake: &Snake, obstacles: &[Point]) -> Point {
    let mut occupied = vec![0u64; ((width as usize) * (height as usize) + 63) / 64];
    for p in &snake.body {
        let idx = (p.y as usize) * (width as usize) + (p.x as usize);
        occupied[idx / 64] |= 1 << (idx % 64);
    }
    for p in obstacles {
        let idx = (p.y as usize) * (width as usize) + (p.x as usize);
        occupied[idx / 64] |= 1 << (idx % 64);
    }

    let mut i = 0;
    loop {
        i = (i + 13) % (width * height);
        let x = i % width;
        let y = i / width;
        let p = Point { x: x.max(1).min(width-2), y: y.max(1).min(height-2) };
        let idx = (p.y as usize) * (width as usize) + (p.x as usize);
        if occupied[idx / 64] & (1 << (idx % 64)) == 0 {
            return p;
        }
    }
}


fn main() {
    let width = 100;
    let height = 100;
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
        let p = generate_food_vec_of_bools(width, height, &snake, &[]);
        std::hint::black_box(p);
    }
    println!("Vec of bools took: {:?}", start.elapsed());

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let p = generate_food_bitvec(width, height, &snake, &[]);
        std::hint::black_box(p);
    }
    println!("Bitvec took: {:?}", start.elapsed());
}
