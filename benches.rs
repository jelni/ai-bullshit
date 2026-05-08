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
    // We will use a predictable PRNG for benchmark
    let mut i = 0;
    loop {
        // Mocking random by a simple counter to make it deterministic
        i = (i + 13) % (width * height);
        let x = i % width;
        let y = i / width;
        let p = Point { x: x.max(1).min(width-2), y: y.max(1).min(height-2) };
        if !snake.body.contains(&p) && !obstacles.contains(&p) {
            return p;
        }
    }
}

fn generate_food_optimized_hashset(width: u16, height: u16, snake: &Snake, obstacles: &[Point]) -> Point {
    // This is just a test to see if Hash is available, wait we need to derive Hash
    let mut i = 0;
    // mock body set
    // ...
    Point { x: 0, y: 0 }
}

fn main() {
    let width = 100;
    let height = 100;
    let mut snake = Snake { body: VecDeque::new() };
    for x in 1..width-2 {
        for y in 1..height-2 {
            snake.body.push_back(Point { x, y });
        }
    }
    // Now the snake occupies almost the whole board!
    // The loop will have to search for a long time.

    let start = std::time::Instant::now();
    for _ in 0..10 {
        let p = generate_food_baseline(width, height, &snake, &[]);
        std::hint::black_box(p);
    }
    println!("Baseline took: {:?}", start.elapsed());
}
