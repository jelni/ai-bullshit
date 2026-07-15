use rand::SeedableRng;
use snake_game::game::{Boss, BossType, Difficulty, Game, Theme};
use snake_game::snake::{Direction, Point, Snake};

#[test]
fn test_mimic_mimics_decoy() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Mimic only moves if within 3 distance
    let initial_pos = Point {
        x: 7,
        y: 5,
    };
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.bosses.push(Boss {
        position: initial_pos,
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Mimic,
        state_timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;

    game.update();

    assert_ne!(game.bosses[0].position, initial_pos);
}

#[test]
fn test_charger_moves_faster() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Charger Boss
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.bosses.push(Boss {
        position: Point {
            x: 9,
            y: 5,
        },
        health: 10,
        max_health: 10,
        move_timer: 0, // In update, move_timer increments, then >= move_threshold (which is 1)
        shoot_timer: 0,
        kind: BossType::Charger,
        state_timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;

    game.update();
    // Move timer for charger increments by 1. Threshold is (2/2) = 1.
    // So it should move in 1 update call.
    let current_pos = game.bosses[0].position;
    assert_ne!(
        current_pos,
        Point {
            x: 9,
            y: 5
        },
        "Charger boss should have moved"
    );
}

#[test]
fn test_juggernaut_destroys_obstacles() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    let initial_pos = Point {
        x: 10,
        y: 5,
    };
    let obstacle_pos = Point {
        x: 9,
        y: 5,
    };

    game.obstacles.insert(obstacle_pos);

    // Juggernaut Boss
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.bosses.push(Boss {
        position: initial_pos,
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Juggernaut,
        state_timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;

    game.update();

    let current_pos = game.bosses[0].position;
    assert_eq!(current_pos, obstacle_pos, "Juggernaut boss should move into the obstacle");
    assert!(
        !game.obstacles.contains(&obstacle_pos),
        "Juggernaut boss should have destroyed the obstacle"
    );
}

#[test]
fn test_spawner_drops_mines() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Spawner Boss
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.bosses.push(Boss {
        position: Point {
            x: 9,
            y: 5,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 29, // Threshold is 30, so next update it should drop a mine
        kind: BossType::Spawner,
        state_timer: 0,
    });

    game.mines.clear();
    game.state = snake_game::game::GameState::Playing;
    game.update();

    assert_eq!(game.mines.len(), 1, "Spawner boss should drop a mine");
    assert!(
        game.mines.contains(&Point {
            x: 9,
            y: 5
        }),
        "Mine should be dropped at boss position"
    );
}

#[test]
fn test_teleporter_boss_moves() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    let initial_pos = Point {
        x: 9,
        y: 5,
    };
    // Teleporter Boss
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.bosses.push(Boss {
        position: initial_pos,
        health: 10,
        max_health: 10,
        move_timer: 250, // Force teleport threshold to trigger
        shoot_timer: 0,
        kind: BossType::Teleporter,
        state_timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;

    game.update();
    // Move timer hits threshold, boss should teleport to a random empty point.
    let current_pos = game.bosses[0].position;
    assert_ne!(current_pos, initial_pos, "Teleporter boss should have changed position");
}

#[test]
fn test_splitter_boss_splits_on_death() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Up;
    game.state = snake_game::game::GameState::Playing;

    // Splitter Boss with health 1 and max_health 10
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.bosses.push(Boss {
        position: Point {
            x: 10,
            y: 10,
        },
        health: 1,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::Splitter,
        state_timer: 0,
    });

    // Freeze time so the boss doesn't move and laser moves 1 step
    game.power_up = Some(snake_game::game::PowerUp {
        p_type: snake_game::game::PowerUpType::TimeFreeze,
        location: Point {
            x: 1,
            y: 1,
        },
        activation_time: Some(
            web_time::SystemTime::now()
                .duration_since(web_time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ),
    });

    // Laser to hit the boss
    // Time freeze means laser does not move. It just does collision check.
    // So we spawn it exactly on top of the boss!
    game.lasers.push(snake_game::game::Laser {
        position: Point {
            x: 10,
            y: 10,
        },
        direction: Direction::Right,
        player: 1,
    });

    game.update();

    // The boss should split into two children
    assert_eq!(game.bosses.len(), 2, "Splitter boss should split into 2 smaller bosses");
    assert_eq!(game.bosses[0].kind, BossType::Splitter);
    assert_eq!(game.bosses[0].health, 5);
    assert_eq!(game.bosses[0].max_health, 5);
    assert_eq!(game.bosses[1].kind, BossType::Splitter);
    assert_eq!(game.bosses[1].health, 5);
    assert_eq!(game.bosses[1].max_health, 5);
}

#[test]
fn test_necromancer_summons_goblin() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Necromancer Boss
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.bosses.push(Boss {
        position: Point {
            x: 9,
            y: 5,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 44, // Default threshold is 45 in Normal mode, so next update spawns goblin
        kind: BossType::Necromancer,
        state_timer: 0,
    });

    game.goblin = None; // Ensure no goblin exists
    game.state = snake_game::game::GameState::Playing;

    game.update();

    assert!(game.goblin.is_some(), "Necromancer should summon a goblin");
    assert_eq!(
        game.goblin.unwrap().position,
        Point {
            x: 9,
            y: 5
        },
        "Goblin should be summoned at boss position"
    );
}

#[test]
fn test_shadowclone_moves_towards_snake() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // ShadowClone Boss
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.bosses.push(Boss {
        position: Point {
            x: 7,
            y: 7,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 0,
        kind: BossType::ShadowClone,
        state_timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;
    game.obstacles.clear();
    game.update();

    let new_pos = game.bosses[0].position;
    // Expected movement: boss at (7, 7), snake at (5, 5). dx = -1, dy = -1.
    // So new_pos could be (6, 7) or (7, 6) or (6, 6) depending on rng and axis movement.
    // But it should move closer to the snake.
    assert!(new_pos.x < 7 || new_pos.y < 7, "Shadow clone should move towards the snake");
}

#[test]
fn test_gorgon_turns_food_to_stone() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    let food_pos = game.food;

    // Gorgon Boss
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.bosses.push(Boss {
        position: Point {
            x: 9,
            y: 5,
        },
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 44, // Default threshold is 45 in Normal mode, so next update it triggers ability
        kind: BossType::Gorgon,
        state_timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;

    game.update();

    // Gorgon's ability should turn the current food into an obstacle, and a new food should be spawned
    assert!(game.obstacles.contains(&food_pos), "Food should be turned into an obstacle by Gorgon");
    assert_ne!(game.food, food_pos, "A new food should have been spawned");
}

#[test]
fn test_vampire_lord_steals_life() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.snake = Snake::new(Point {
        x: 5,
        y: 5,
    });
    game.snake.direction = Direction::Right;

    // Set initial lives to 3
    game.lives = 3;

    // Place VampireLord adjacent to the snake
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.bosses.push(Boss {
        position: Point {
            x: 6,
            y: 5,
        },
        health: 5,
        max_health: 10,
        move_timer: 1, // Ready to act
        shoot_timer: 0,
        kind: BossType::VampireLord,
        state_timer: 0,
    });

    game.state = snake_game::game::GameState::Playing;

    game.update();

    // Check life drain and teleport
    assert_eq!(game.lives, 2, "Vampire Lord should drain 1 life from the player");
    assert_eq!(game.bosses[0].health, 10, "Vampire Lord should heal itself by 5");
    assert_ne!(
        game.bosses[0].position,
        Point {
            x: 6,
            y: 5
        },
        "Vampire Lord should teleport away after stealing life"
    );
}

#[test]
fn test_alchemist_drops_poison() {
    let mut game = snake_game::game::Game::new(
        20,
        20,
        false,
        'x',
        snake_game::game::Theme::Classic,
        snake_game::game::Difficulty::Normal,
    );

    game.bosses.clear();
    let start_pos = snake_game::snake::Point {
        x: 5,
        y: 5,
    };
    game.rng = rand::rngs::StdRng::seed_from_u64(42);
    game.bosses.push(snake_game::game::Boss {
        position: start_pos,
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 20,
        kind: snake_game::game::BossType::Alchemist,
        state_timer: 0,
    });
    game.snake.move_to(
        snake_game::snake::Point {
            x: 18,
            y: 18,
        },
        false,
    );

    assert!(game.poison_food.is_none());

    game.update();

    // shoot_timer was 20, the threshold is 20, so it will shoot and reset to 0
    // wait, we need to make sure the target position logic isn't messing with update
    // run update multiple times
    game.state = snake_game::game::GameState::Playing;
    for _ in 0..25 {
        game.update();
        if game.poison_food.is_some() {
            break;
        }
    }
    assert!(game.poison_food.is_some());
}

#[test]
fn test_dragon_boss_shoots_lasers() {
    let mut game = snake_game::game::Game::new(
        20,
        20,
        false,
        'x',
        snake_game::game::Theme::Classic,
        snake_game::game::Difficulty::Normal,
    );
    game.bosses.clear();
    let start_pos = snake_game::snake::Point {
        x: 5,
        y: 5,
    };
    game.bosses.push(snake_game::game::Boss {
        position: start_pos,
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 19, // One tick before threshold (20)
        kind: snake_game::game::BossType::Dragon,
        state_timer: 0,
    });
    // Set snake position to the right to control direction
    game.snake = snake_game::snake::Snake::new(snake_game::snake::Point {
        x: 10,
        y: 5,
    });

    let initial_lasers = game.lasers.len();
    game.state = snake_game::game::GameState::Playing;
    game.update();

    assert_eq!(game.lasers.len(), initial_lasers + 3, "Dragon should shoot 3 lasers");
    // Since snake is to the right, dx > 0 and dy = 0, so direction is Right.
    // Based on actual positions printed, the lasers should be at (8,5), (8,6), (8,4)
    // because calculate_next_head_dir computes position depending on speed.
    let expected_positions = vec![
        snake_game::snake::Point {
            x: 8,
            y: 5,
        }, // Middle
        snake_game::snake::Point {
            x: 8,
            y: 6,
        }, // Bottom
        snake_game::snake::Point {
            x: 8,
            y: 4,
        }, // Top
    ];
    let mut actual_positions = Vec::new();
    for i in initial_lasers..game.lasers.len() {
        actual_positions.push(game.lasers[i].position);
    }
    for expected in expected_positions {
        assert!(actual_positions.contains(&expected), "Missing laser at {:?}", expected);
    }
}

#[test]
fn test_mage_boss_spawns_meteor_and_powerup() {
    let mut game = snake_game::game::Game::new(
        20,
        20,
        false,
        'x',
        snake_game::game::Theme::Classic,
        snake_game::game::Difficulty::Normal,
    );
    game.bosses.clear();
    let start_pos = snake_game::snake::Point {
        x: 5,
        y: 5,
    };
    game.bosses.push(snake_game::game::Boss {
        position: start_pos,
        health: 10,
        max_health: 10,
        move_timer: 0,
        shoot_timer: 29, // One tick before threshold (30)
        kind: snake_game::game::BossType::Mage,
        state_timer: 0,
    });
    // Set snake position
    game.snake = snake_game::snake::Snake::new(snake_game::snake::Point {
        x: 10,
        y: 5,
    });

    let initial_meteors = game.meteors.len();
    assert!(game.power_up.is_none());

    game.state = snake_game::game::GameState::Playing;
    game.update();

    assert_eq!(game.meteors.len(), initial_meteors + 1, "Mage should spawn a meteor");
    assert_eq!(
        game.meteors.last().unwrap().position,
        snake_game::snake::Point {
            x: 10,
            y: 6
        },
        "Meteor should be spawned at snake head"
    );

    assert!(game.power_up.is_some(), "Mage should spawn a power up");
    let power_up = game.power_up.unwrap();
    assert!(
        power_up.p_type == snake_game::game::PowerUpType::TimeFreeze,
        "Power up should be TimeFreeze"
    );
}

#[test]
fn test_puffer_boss_moves_and_shoots() {
    let mut game = snake_game::game::Game::new(
        20,
        20,
        false,
        'x',
        snake_game::game::Theme::Classic,
        snake_game::game::Difficulty::Normal,
    );
    game.bosses.clear();
    let start_pos = snake_game::snake::Point {
        x: 5,
        y: 5,
    };
    game.bosses.push(snake_game::game::Boss {
        position: start_pos,
        health: 10,
        max_health: 10,
        move_timer: 2,   // 1 tick before move threshold (3)
        shoot_timer: 29, // 1 tick before shoot threshold (30)
        kind: snake_game::game::BossType::Puffer,
        state_timer: 0,
    });
    // Set snake position to bottom-right to test movement
    game.snake = snake_game::snake::Snake::new(snake_game::snake::Point {
        x: 10,
        y: 10,
    });

    let initial_lasers = game.lasers.len();

    game.state = snake_game::game::GameState::Playing;
    game.update();

    let boss = game.bosses.first().unwrap();
    // It should have moved towards the snake
    assert!(boss.position != start_pos, "Puffer boss should move");

    // It should have shot 4 lasers (Up, Down, Left, Right)
    assert_eq!(game.lasers.len(), initial_lasers + 4, "Puffer boss should shoot 4 lasers");

    // Check lasers originated around the boss's updated position

    // Up is y+1 or y-1? Let's check dirs on the lasers actually spawned
    let dirs: Vec<_> = game.lasers[initial_lasers..].iter().map(|l| l.direction).collect();
    assert!(dirs.contains(&snake_game::snake::Direction::Up));
    assert!(dirs.contains(&snake_game::snake::Direction::Down));
    assert!(dirs.contains(&snake_game::snake::Direction::Left));
    assert!(dirs.contains(&snake_game::snake::Direction::Right));
}
