use snake_game::game::{Game, GameMode, Theme, Difficulty, generate_flow_field};
use snake_game::snake::{Point, Direction, Snake};

#[test]
fn test_flow_field_avoids_obstacles() {
    let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
    game.obstacles.clear();

    // Create a wall
    for x in 8..=12 {
        game.obstacles.insert(Point { x, y: 10 });
    }

    let target = Point { x: 10, y: 15 };
    let flow_field = generate_flow_field(&game, &[target]);

    // Verify that the start point (10, 5) points around the wall and not through it
    let start = Point { x: 10, y: 5 };

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
        game.obstacles.insert(Point { x, y: 10 });
    }

    game.portals = Some((Point { x: 5, y: 5 }, Point { x: 5, y: 15 }));

    let target = Point { x: 5, y: 17 };
    let flow_field = generate_flow_field(&game, &[target]);

    let start = Point { x: 5, y: 3 };
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
                println!("Step {}: pos {:?} dir {:?}", steps, final_p, d);
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
    game.food = Point { x: 1, y: 1 };
    game.update();

    assert_ne!(initial_targets, game.flow_field_targets, "Flow field targets should update");
}
