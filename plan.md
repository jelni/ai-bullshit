Wait! It passes again when run in isolation!
This confirms it's a flaky test due to random obstacles being added in `Game::new`.
`Game::new` sets up some random obstacles because of the difficulty and theme and RNG!
In `Game::new`:
```rust
        let mut game = Self { ... };
        game.reset(); // which calls generate_obstacles!
```
Wait, if it's because of `game.obstacles`, we should just clear `game.obstacles` in `tests/test_boss_avoids_black_hole.rs`.
