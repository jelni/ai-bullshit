use snake_game::game::{Difficulty, Game, GameMode, Theme, generate_flow_field};
use snake_game::snake::Point;

#[test]
fn test_flow_field_avoids_obstacles() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();

    // Create a wall
    for x in 8..=12 {
        game.obstacles.insert(Point {
            x,
            y: 10,
        });
    }

    let target = Point {
        x: 10,
        y: 15,
    };
    let flow_field = generate_flow_field(&game, &[target]);

    // Verify that the start point (10, 5) points around the wall and not through it
    let start = Point {
        x: 10,
        y: 5,
    };

    // Follow the flow field to see if we reach the target
    let mut current = start;
    let mut reached = false;

    for _ in 0..100 {
        if current == target {
            reached = true;
            break;
        }
        if let Some(&d) = flow_field.get(&current) {
            current = Game::calculate_next_head_dir(current, d);
            assert!(!game.obstacles.contains(&current), "Flow field directed into an obstacle!");
        } else {
            break;
        }
    }

    assert!(reached, "Following flow field did not reach target");
}

#[test]
fn test_flow_field_handles_portals() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();

    for x in 0..20 {
        game.obstacles.insert(Point {
            x,
            y: 10,
        });
    }

    game.portals = Some((
        Point {
            x: 5,
            y: 5,
        },
        Point {
            x: 5,
            y: 15,
        },
    ));

    let target = Point {
        x: 5,
        y: 17,
    };
    let flow_field = generate_flow_field(&game, &[target]);

    let start = Point {
        x: 5,
        y: 3,
    };
    let mut current = start;
    let mut reached = false;
    let mut steps = 0;

    for _ in 0..50 {
        if current == target {
            reached = true;
            break;
        }
        if let Some(&d) = flow_field.get(&current) {
            let next_p = Game::calculate_next_head_dir(current, d);
            if let Some(final_p) = game.get_final_p(next_p) {
                current = final_p;
                steps += 1;
                println!("Step {steps}: pos {final_p:?} dir {d:?}");
            } else {
                break;
            }
        } else {
            break;
        }
    }

    assert!(reached, "Flow field should traverse portals");
    assert!(steps < 10, "Flow field path should be short due to portal usage");
}

#[test]
fn test_flow_field_updates_in_game() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.mode = GameMode::MassiveMultiplayer;
    game.obstacles.clear();
    game.reset(); // Should set auto_pilot and spawn bots

    game.update();

    assert!(game.flow_field.is_some(), "Flow field should be generated for MassiveMultiplayer");
    let initial_targets = game.flow_field_targets.clone();

    // Change food
    game.food = Point {
        x: 1,
        y: 1,
    };
    game.update();

    assert_ne!(initial_targets, game.flow_field_targets, "Flow field targets should update");
}

#[test]
fn test_flow_field_avoids_entities() {
    use snake_game::game::{Boss, BossType, Laser};
    use snake_game::snake::Direction;
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();

    // Create a boss at (10, 10)
    game.bosses.push(Boss {
        position: Point {
            x: 10,
            y: 10,
        },
        health: 10,
        max_health: 10,
        kind: BossType::Juggernaut,
        move_timer: 0,
        shoot_timer: 0,
        state_timer: 0,
    });

    // Create a laser at (10, 11)
    game.lasers.push(Laser {
        position: Point {
            x: 10,
            y: 11,
        },
        direction: Direction::Down,
        player: 3,
    });

    let target = Point {
        x: 10,
        y: 15,
    };
    let flow_field = generate_flow_field(&game, &[target]);

    // Ensure the flow field does not route through the boss
    let boss_point = Point {
        x: 10,
        y: 10,
    };
    let laser_point = Point {
        x: 10,
        y: 11,
    };

    // The flow field should not have an entry mapping from the boss or laser positions
    // since they act as static obstacles.
    assert!(!flow_field.contains_key(&boss_point), "Flow field should not route through a boss");
    assert!(!flow_field.contains_key(&laser_point), "Flow field should not route through a laser");

    // Check path routes around it
    let start = Point {
        x: 10,
        y: 5,
    };
    let mut current = start;
    let mut reached = false;

    for _ in 0..100 {
        if current == target {
            reached = true;
            break;
        }
        if let Some(&d) = flow_field.get(&current) {
            current = Game::calculate_next_head_dir(current, d);
            assert_ne!(current, boss_point, "Flow field directed into a boss!");
            assert_ne!(current, laser_point, "Flow field directed into a laser!");
        } else {
            break;
        }
    }

    assert!(reached, "Following flow field did not reach target while avoiding entities");
}
