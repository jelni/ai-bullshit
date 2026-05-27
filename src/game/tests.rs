use super::*;
#[cfg(test)]
#[expect(clippy::module_inception, reason = "Allowing module inception")]
mod tests {
    use super::*;
    use std::{fs::File, io::Write};
    #[test]
    fn test_generate_dungeon_obstacles() {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let width = 20;
        let height = 20;
        let obstacles = Game::generate_dungeon_obstacles(width, height, &mut rng);
        assert!(!obstacles.is_empty(), "Dungeon generation should create obstacles (walls)");
        let start_x = width / 2;
        let start_y = height / 2;
        for dy in -2..=2 {
            for dx in -2..=2 {
                let cx = i32::from(start_x) + dx;
                let cy = i32::from(start_y) + dy;
                if cx > 0 && cx < i32::from(width - 1) && cy > 0 && cy < i32::from(height - 1) {
                    assert!(
                        !obstacles.contains(&Point {
                            x: u16::try_from(cx).unwrap_or(0),
                            y: u16::try_from(cy).unwrap_or(0)
                        }),
                        "Center area should be free of obstacles in dungeon mode"
                    );
                }
            }
        }
    }
    #[test]
    fn test_decade_challenge_determinism() {
        let mut game1 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game1.mode = GameMode::DecadeChallenge;
        game1.reset();
        let mut game2 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game2.mode = GameMode::DecadeChallenge;
        game2.reset();
        assert_eq!(game1.food, game2.food);
        assert_eq!(game1.obstacles, game2.obstacles);
    }
    #[test]
    fn test_century_challenge_determinism() {
        let mut game1 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game1.mode = GameMode::CenturyChallenge;
        game1.reset();
        let mut game2 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game2.mode = GameMode::CenturyChallenge;
        game2.reset();
        assert_eq!(game1.food, game2.food);
        assert_eq!(game1.obstacles, game2.obstacles);
    }
    #[test]
    fn test_millennium_challenge_determinism() {
        let mut game1 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game1.mode = GameMode::MillenniumChallenge;
        game1.reset();
        let mut game2 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game2.mode = GameMode::MillenniumChallenge;
        game2.reset();
        assert_eq!(game1.food, game2.food);
        assert_eq!(game1.obstacles, game2.obstacles);
    }
    #[test]
    fn test_yearly_challenge_determinism() {
        let mut game1 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game1.mode = GameMode::YearlyChallenge;
        game1.reset();
        let mut game2 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game2.mode = GameMode::YearlyChallenge;
        game2.reset();
        assert_eq!(game1.food, game2.food);
        assert_eq!(game1.obstacles, game2.obstacles);
        for _ in 0..5 {
            let next_food = game1.food;
            game1.snake.move_to(next_food, true);
            game1.process_food_collision(next_food, false);
            game2.snake.move_to(next_food, true);
            game2.process_food_collision(next_food, false);
            assert_eq!(game1.food, game2.food, "Food generation drifted");
            assert_eq!(game1.obstacles, game2.obstacles, "Obstacles generation drifted");
        }
    }
    #[test]
    fn test_generate_cave_obstacles() {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let width = 20;
        let height = 20;
        let obstacles = Game::generate_cave_obstacles(width, height, &mut rng);
        assert!(!obstacles.is_empty(), "Cave generation should create obstacles");
        let start_x = width / 2;
        let start_y = height / 2;
        for dy in -3..=3 {
            for dx in -3..=3 {
                let cx = i32::from(start_x) + dx;
                let cy = i32::from(start_y) + dy;
                if cx > 0 && cx < i32::from(width - 1) && cy > 0 && cy < i32::from(height - 1) {
                    assert!(
                        !obstacles.contains(&Point {
                            x: u16::try_from(cx).unwrap_or(0),
                            y: u16::try_from(cy).unwrap_or(0)
                        }),
                        "Center area should be free of obstacles"
                    );
                }
            }
        }
    }
    #[test]
    fn test_portal_teleportation() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.snake = crate::snake::Snake::new(crate::snake::Point {
            x: 10,
            y: 10,
        });
        game.snake.direction = crate::snake::Direction::Right;
        let p1 = crate::snake::Point {
            x: 11,
            y: 10,
        };
        let p2 = crate::snake::Point {
            x: 5,
            y: 5,
        };
        game.portals = Some((p1, p2));
        let (final_head1, _final_head2, hit_wall1, _hit_wall2) = game.calculate_final_heads();
        assert_eq!(final_head1, p2);
        assert!(!hit_wall1);
    }
    #[test]
    fn test_save_and_load_settings() {
        let file_path = "savegame_test_settings.json";
        let _ = std::fs::remove_file(file_path);
        let mut game1 =
            Game::new(20, 20, true, '@', crate::game::Theme::Neon, crate::game::Difficulty::Hard);
        game1.snake.body.clear();
        game1.snake.body.push_back(Point {
            x: 10,
            y: 10,
        });
        game1.food = Point {
            x: 5,
            y: 5,
        };
        game1.obstacles.clear();
        game1.save_game_to_file(file_path);
        let mut game2 = Game::new(
            20,
            20,
            false,
            '█',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Easy,
        );
        let success = game2.load_game_from_file(file_path);
        assert!(success);
        assert_eq!(game2.difficulty, crate::game::Difficulty::Hard);
        assert_eq!(game2.theme, crate::game::Theme::Neon);
        assert!(game2.wrap_mode);
        assert_eq!(game2.skin, '@');
        let _ = std::fs::remove_file(file_path);
    }
    #[test]
    fn test_save_and_load_high_scores() {
        let file_path = "highscore_test.txt";
        let _ = std::fs::remove_file(file_path);
        let mut game = Game::new(
            20,
            20,
            false,
            '#',
            crate::game::Theme::Dark,
            crate::game::Difficulty::Normal,
        );
        game.high_scores.clear();
        game.save_high_score_to_file(file_path, "Alice".to_string(), 100);
        game.save_high_score_to_file(file_path, "Bob".to_string(), 200);
        game.save_high_score_to_file(file_path, "Charlie".to_string(), 50);
        let loaded_scores = Game::load_high_scores_from_file(file_path);
        assert_eq!(loaded_scores.len(), 3);
        assert_eq!(loaded_scores[0], ("Bob".to_string(), 200));
        assert_eq!(loaded_scores[1], ("Alice".to_string(), 100));
        assert_eq!(loaded_scores[2], ("Charlie".to_string(), 50));
        let _ = std::fs::remove_file(file_path);
    }
    #[test]
    fn test_save_and_load_auto_pilot() {
        let mut game = Game::new(20, 20, false, '#', Theme::Dark, Difficulty::Normal);
        game.auto_pilot = true;
        let file_path = "savegame_test_autopilot.json";
        game.save_game_to_file(file_path);
        let mut new_game = Game::new(20, 20, false, '#', Theme::Dark, Difficulty::Normal);
        assert!(!new_game.auto_pilot);
        let loaded = new_game.load_game_from_file(file_path);
        assert!(loaded);
        assert!(new_game.auto_pilot);
        let _ = std::fs::remove_file(file_path);
    }
    #[test]
    fn test_reset_clears_power_up() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.power_up = Some(PowerUp {
            p_type: PowerUpType::SpeedBoost,
            location: crate::snake::Point {
                x: 5,
                y: 5,
            },
            activation_time: None,
        });
        game.reset();
        assert!(game.power_up.is_none(), "Power-up should be cleared on reset");
    }
    #[test]
    fn test_load_game_dos_protection() {
        let file_path = "savegame_test_dos.json";
        let mut file = File::create(file_path).expect("Failed to create dos test file");
        let data = vec![b'a'; 2 * 1024 * 1024];
        file.write_all(&data).expect("Failed to write to dos test file");
        let mut game = Game::new(
            20,
            20,
            false,
            '#',
            crate::game::Theme::Dark,
            crate::game::Difficulty::Normal,
        );
        let loaded = game.load_game_from_file(file_path);
        assert!(!loaded);
        let _ = std::fs::remove_file(file_path);
    }
    #[test]
    fn test_reset_clears_bot_flags() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.auto_pilot = true;
        game.used_bot_this_session = true;
        game.reset();
        assert!(
            !game.used_bot_this_session && !game.auto_pilot,
            "Bot flags should be cleared on reset"
        );
    }
    #[test]
    fn test_calculate_autopilot_move_to_food() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.obstacles.clear();
        game.snake = crate::snake::Snake::new(crate::snake::Point {
            x: 10,
            y: 10,
        });
        game.food = crate::snake::Point {
            x: 10,
            y: 8,
        };
        let next_move = game.calculate_autopilot_move();
        assert_eq!(next_move, Some(crate::snake::Direction::Up));
    }
    #[test]
    fn test_apply_magnet() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.snake = crate::snake::Snake::new(crate::snake::Point {
            x: 10,
            y: 10,
        });
        game.food = crate::snake::Point {
            x: 10,
            y: 15,
        };
        game.obstacles.clear();
        game.power_up = Some(PowerUp {
            p_type: PowerUpType::Magnet,
            location: crate::snake::Point {
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
        for _ in 0..100 {
            game.apply_magnet();
            if game.food.y < 15 {
                break;
            }
        }
        assert!(game.food.y < 15, "Food should have moved closer to the snake");
    }
    #[test]
    fn test_generate_maze_obstacles() {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let width = 21;
        let height = 21;
        let obstacles = Game::generate_maze_obstacles(width, height, &mut rng);
        assert!(!obstacles.is_empty(), "Maze generation should create obstacles");
        let start_x = width / 2;
        let start_y = height / 2;
        for dy in -2..=2 {
            for dx in -2..=2 {
                let cx = i32::from(start_x) + dx;
                let cy = i32::from(start_y) + dy;
                if cx > 0 && cx <= i32::from(width - 2) && cy > 0 && cy <= i32::from(height - 2) {
                    assert!(
                        !obstacles.contains(&Point {
                            x: u16::try_from(cx).unwrap_or(0),
                            y: u16::try_from(cy).unwrap_or(0)
                        }),
                        "Center area should be free of obstacles"
                    );
                }
            }
        }
    }
    #[test]
    fn test_bfs_pathfind() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.obstacles.clear();
        for x in 8..=12 {
            game.obstacles.insert(Point {
                x,
                y: 10,
            });
        }
        let start = Point {
            x: 10,
            y: 5,
        };
        let target = Point {
            x: 10,
            y: 15,
        };
        let dir = game.bfs_pathfind(start, target);
        assert!(dir.is_some(), "BFS should find a path around the wall");
        let mut current = start;
        let mut reached = false;
        for _ in 0..100 {
            if current == target {
                reached = true;
                break;
            }
            if let Some(next_dir) = game.bfs_pathfind(current, target) {
                current = Game::calculate_next_head_dir(current, next_dir);
                assert!(!game.obstacles.contains(&current), "Path should not hit obstacles");
            } else {
                break;
            }
        }
        assert!(reached, "Following BFS should reach target");
    }
    #[test]
    fn test_daily_challenge_determinism() {
        let mut game1 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game1.mode = GameMode::DailyChallenge;
        game1.reset();
        let mut game2 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game2.mode = GameMode::DailyChallenge;
        game2.reset();
        assert_eq!(game1.food, game2.food);
        assert_eq!(game1.obstacles, game2.obstacles);
        for _ in 0..5 {
            let next_food = game1.food;
            game1.snake.move_to(next_food, true);
            game1.process_food_collision(next_food, false);
            game2.snake.move_to(next_food, true);
            game2.process_food_collision(next_food, false);
            assert_eq!(game1.food, game2.food, "Food generation drifted");
            assert_eq!(game1.obstacles, game2.obstacles, "Obstacles generation drifted");
        }
    }
    #[test]
    fn test_weekly_challenge_determinism() {
        let mut game1 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game1.mode = GameMode::WeeklyChallenge;
        game1.reset();
        let mut game2 = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game2.mode = GameMode::WeeklyChallenge;
        game2.reset();
        assert_eq!(game1.food, game2.food);
        assert_eq!(game1.obstacles, game2.obstacles);
        for _ in 0..5 {
            let next_food = game1.food;
            game1.snake.move_to(next_food, true);
            game1.process_food_collision(next_food, false);
            game2.snake.move_to(next_food, true);
            game2.process_food_collision(next_food, false);
            assert_eq!(game1.food, game2.food, "Food generation drifted");
            assert_eq!(game1.obstacles, game2.obstacles, "Obstacles generation drifted");
        }
    }
    #[test]
    fn test_upgrades() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.stats.upgrade_extra_lives = 2;
        game.reset();
        assert_eq!(game.lives, 5);
        game.stats.upgrade_powerup_duration = 3;
        assert_eq!(game.powerup_duration(), 8);
        game.stats.upgrade_laser_capacity = 2;
        game.lasers.clear();
        for _ in 0..10 {
            game.shoot_laser(1);
        }
        let active_lasers = game.lasers.iter().filter(|l| l.player == 1).count();
        assert_eq!(active_lasers, 5);
        game.stats.upgrade_coin_multiplier = 5;
        let initial_coins = game.stats.coins;
        let p = game.food;
        game.snake.move_to(p, true);
        game.process_food_collision(p, false);
        assert_eq!(game.stats.coins - initial_coins, 4);
    }
    #[test]
    fn test_weather_tornado_shifts_food() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.state = GameState::Playing;
        game.weather = Weather::Tornado;
        game.food = crate::snake::Point {
            x: 5,
            y: 5,
        };
        let initial_food = game.food;
        game.snake.direction_queue.push_back(crate::snake::Direction::Down);
        let mut shifted = false;
        for _ in 0..10000 {
            game.weather = Weather::Tornado;
            game.snake.direction_queue.push_back(crate::snake::Direction::Down);
            game.snake = crate::snake::Snake::new(crate::snake::Point {
                x: 10,
                y: 10,
            });
            game.update();
            if game.food != initial_food
                && game.food
                    != (crate::snake::Point {
                        x: 0,
                        y: 0,
                    })
            {
                shifted = true;
                break;
            }
        }
        assert!(shifted, "Food should have shifted due to Tornado effect");
    }
    #[test]
    fn test_weather_random_transition() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.state = GameState::Playing;
        game.weather = Weather::Snow;
        assert_eq!(
            game.weather,
            Weather::Snow,
            "Weather state should be mutable and hold correctly"
        );
    }
    #[test]
    fn test_lightning_column_strike() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.state = GameState::Playing;
        game.weather = Weather::Storm;
        game.rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut struck = false;
        for _ in 0..10000 {
            game.weather = Weather::Storm;
            let old_lives = game.lives;
            game.update();
            game.lives = old_lives;
            game.state = GameState::Playing;
            game.weather = Weather::Storm;
            if game.lightning_column.is_some() {
                struck = true;
                break;
            }
        }
        assert!(struck, "Lightning should strike during a storm");
    }
    #[test]
    fn test_calculate_autopilot_avoids_opponent() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.obstacles.clear();
        game.mode = GameMode::BotVsBot;
        game.snake = crate::snake::Snake::new(crate::snake::Point {
            x: 5,
            y: 5,
        });
        game.snake.direction = crate::snake::Direction::Right;
        game.food = crate::snake::Point {
            x: 9,
            y: 5,
        };
        let mut p2 = crate::snake::Snake::new(crate::snake::Point {
            x: 6,
            y: 5,
        });
        p2.direction = crate::snake::Direction::Down;
        game.player2 = Some(p2);
        let next_move = game.calculate_autopilot_move();
        assert!(
            next_move == Some(crate::snake::Direction::Up)
                || next_move == Some(crate::snake::Direction::Down)
        );
    }
    #[test]
    fn test_calculate_autopilot_avoids_boss() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.obstacles.clear();
        game.snake = crate::snake::Snake::new(crate::snake::Point {
            x: 5,
            y: 5,
        });
        game.snake.direction = crate::snake::Direction::Right;
        game.food = crate::snake::Point {
            x: 9,
            y: 5,
        };
        game.bosses.push(Boss {
            position: crate::snake::Point {
                x: 6,
                y: 5,
            },
            health: 10,
            max_health: 10,
            move_timer: 0,
            shoot_timer: 0,
            kind: BossType::Shooter,
            state_timer: 0,
        });
        let next_move = game.calculate_autopilot_move();
        assert!(
            next_move == Some(crate::snake::Direction::Up)
                || next_move == Some(crate::snake::Direction::Down)
        );
    }
    #[test]
    fn test_calculate_autopilot_avoids_laser() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.obstacles.clear();
        game.snake = crate::snake::Snake::new(crate::snake::Point {
            x: 5,
            y: 5,
        });
        game.snake.direction = crate::snake::Direction::Right;
        game.food = crate::snake::Point {
            x: 9,
            y: 5,
        };
        game.lasers.push(Laser {
            position: crate::snake::Point {
                x: 6,
                y: 5,
            },
            direction: crate::snake::Direction::Left,
            player: 0,
        });
        let next_move = game.calculate_autopilot_move();
        assert!(
            next_move == Some(crate::snake::Direction::Up)
                || next_move == Some(crate::snake::Direction::Down)
        );
    }
    #[test]
    fn test_elo_calculation() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.stats.player_elo = 1000;
        game.stats.bot_elo = 1000;
        game.update_elo(false, true);
        assert_eq!(game.stats.player_elo, 1000);
        assert_eq!(game.stats.bot_elo, 1000);
        game.update_elo(true, false);
        assert!(game.stats.player_elo > 1000);
        assert!(game.stats.bot_elo < 1000);
        let p_elo_after_win = game.stats.player_elo;
        let b_elo_after_loss = game.stats.bot_elo;
        game.update_elo(false, false);
        assert!(game.stats.player_elo < p_elo_after_win);
        assert!(game.stats.bot_elo > b_elo_after_loss);
    }
    #[test]
    fn test_sprint_mechanic() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.state = GameState::Playing;
        game.snake = crate::snake::Snake::new(Point {
            x: 5,
            y: 5,
        });
        game.snake.direction = crate::snake::Direction::Right;
        game.obstacles.clear();
        game.food = Point {
            x: 1,
            y: 1,
        };
        game.update();
        assert_eq!(
            game.snake.head(),
            Point {
                x: 6,
                y: 5
            }
        );
        game.is_sprinting = true;
        game.update();
        assert_eq!(
            game.snake.head(),
            Point {
                x: 8,
                y: 5
            }
        );
    }
    #[test]
    fn test_diamond_extra_life() {
        let mut game_normal = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        let base_lives = game_normal.lives;
        game_normal.skin = '💎';
        game_normal.reset();
        assert_eq!(game_normal.lives, base_lives + 1);
    }
    #[test]
    fn test_bitcoin_doubles_coins() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        let initial_coins = game.stats.coins;
        game.process_food_collision(
            Point {
                x: 5,
                y: 5,
            },
            false,
        );
        assert_eq!(game.stats.coins - initial_coins, 2);
        let initial_coins_btc = game.stats.coins;
        game.skin = '₿';
        game.process_food_collision(
            Point {
                x: 5,
                y: 5,
            },
            false,
        );
        assert_eq!(game.stats.coins - initial_coins_btc, 8);
    }
    #[test]
    fn test_gorilla_smashes_walls() {
        let mut game = Game::new(
            20,
            20,
            false,
            '🦍',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.obstacles.clear();
        let target = Point {
            x: 5,
            y: 5,
        };
        game.obstacles.insert(target);
        game.snake = crate::snake::Snake::new(Point {
            x: 5,
            y: 6,
        });
        game.snake.direction = crate::snake::Direction::Up;
        game.state = GameState::Playing;
        let mut old_lives = game.lives;
        game.update();
        assert_eq!(game.lives, old_lives, "Gorilla should not lose life on wall");
        assert!(!game.obstacles.contains(&target), "Gorilla should smash the wall");
        let mut game_normal = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game_normal.obstacles.clear();
        game_normal.obstacles.insert(target);
        game_normal.snake = crate::snake::Snake::new(Point {
            x: 5,
            y: 6,
        });
        game_normal.snake.direction = crate::snake::Direction::Up;
        game_normal.state = GameState::Playing;
        old_lives = game_normal.lives;
        game_normal.update();
        assert_eq!(game_normal.lives, old_lives - 1, "Normal skin should lose life on wall");
        assert!(game_normal.obstacles.contains(&target), "Normal skin should not smash wall");
    }
    #[test]
    fn test_calculate_autopilot_uses_portals() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.obstacles.clear();
        game.snake = crate::snake::Snake::new(crate::snake::Point {
            x: 2,
            y: 2,
        });
        game.snake.direction = crate::snake::Direction::Down;
        game.food = crate::snake::Point {
            x: 18,
            y: 18,
        };
        let p1 = crate::snake::Point {
            x: 3,
            y: 2,
        };
        let p2 = crate::snake::Point {
            x: 17,
            y: 18,
        };
        game.portals = Some((p1, p2));
        game.obstacles.clear();
        let next_move = game.calculate_autopilot_move();
        assert_eq!(next_move, Some(crate::snake::Direction::Right));
    }
    #[test]
    fn test_calculate_autopilot_targets_goblin() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.obstacles.clear();
        game.food = Point {
            x: 1,
            y: 1,
        };
        game.snake = crate::snake::Snake::new(Point {
            x: 10,
            y: 10,
        });
        game.snake.direction = crate::snake::Direction::Right;
        let goblin_pos = Point {
            x: 10,
            y: 5,
        };
        game.goblin = Some(Goblin {
            position: goblin_pos,
            move_timer: 0,
            food_eaten: 0,
        });
        let next_move = game.calculate_autopilot_move();
        assert_eq!(next_move, Some(crate::snake::Direction::Up));
    }
    #[test]
    fn test_goblin_steals_food_and_escapes() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.obstacles.clear();
        game.food = Point {
            x: 10,
            y: 10,
        };
        game.goblin = Some(Goblin {
            position: Point {
                x: 9,
                y: 10,
            },
            move_timer: 1,
            food_eaten: 2,
        });
        game.state = GameState::Playing;
        game.snake = crate::snake::Snake::new(Point {
            x: 1,
            y: 1,
        });
        game.update();
        assert!(game.goblin.is_none(), "Goblin should have stolen food and despawned");
    }
    #[test]
    fn test_snake_catches_goblin() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        game.obstacles.clear();
        let initial_score = game.score;
        let initial_coins = game.stats.coins;
        let goblin_pos = Point {
            x: 10,
            y: 10,
        };
        game.goblin = Some(Goblin {
            position: goblin_pos,
            move_timer: 0,
            food_eaten: 0,
        });
        game.snake = crate::snake::Snake::new(Point {
            x: 10,
            y: 9,
        });
        game.snake.direction = crate::snake::Direction::Down;
        game.state = GameState::Playing;
        game.update();
        assert!(game.goblin.is_none(), "Goblin should be caught and despawned");
        assert_eq!(game.score, initial_score + 500, "Should get 500 score for catching goblin");
        assert_eq!(
            game.stats.coins,
            initial_coins + 500,
            "Should get 500 coins for catching goblin"
        );
    }
    #[test]
    fn test_laser_hits_goblin() {
        let mut game = Game::new(
            20,
            20,
            false,
            'x',
            crate::game::Theme::Classic,
            crate::game::Difficulty::Normal,
        );
        let initial_score = game.score;
        let goblin_pos = Point {
            x: 10,
            y: 10,
        };
        game.goblin = Some(Goblin {
            position: goblin_pos,
            move_timer: 0,
            food_eaten: 0,
        });
        game.lasers.push(Laser {
            position: Point {
                x: 9,
                y: 10,
            },
            direction: crate::snake::Direction::Right,
            player: 1,
        });
        game.state = GameState::Playing;
        game.snake = crate::snake::Snake::new(Point {
            x: 1,
            y: 1,
        });
        game.update();
        assert!(game.goblin.is_none(), "Goblin should be hit by laser and despawned");
        assert_eq!(game.score, initial_score + 500, "Should get score for shooting goblin");
    }
    #[test]
    fn test_weather_sandstorm_shifts_food() {
        let mut game = Game::new(20, 20, false, '#', Theme::Dark, Difficulty::Normal);
        game.weather = Weather::Sandstorm;
        game.food = Point { x: 10, y: 10 };
        let initial_food = game.food;

        // Run updates to see if Sandstorm moves the food
        let mut moved = false;
        for _ in 0..2000 {
            game.state = GameState::Playing; // Ensure playing state
            game.update();
            game.weather = Weather::Sandstorm; // ensure it stays sandstorm
            if game.food != initial_food {
                moved = true;
                break;
            }
        }

        assert!(moved, "Sandstorm should randomly shift food");
    }

    #[test]
    fn test_weather_earthquake_affects_obstacles() {
        let mut game = Game::new(20, 20, false, '#', Theme::Dark, Difficulty::Normal);
        game.weather = Weather::Earthquake;

        // clear and add some specific obstacles
        game.obstacles.clear();
        game.obstacles.insert(Point { x: 5, y: 5 });
        game.obstacles.insert(Point { x: 6, y: 6 });
        let initial_obs_count = game.obstacles.len();

        // Run updates to see if Earthquake changes obstacles
        let mut changed = false;
        for _ in 0..2000 {
            game.state = GameState::Playing; // Ensure playing state
            game.update();
            game.weather = Weather::Earthquake; // ensure it stays earthquake
            if game.obstacles.len() != initial_obs_count || !game.obstacles.contains(&Point { x: 5, y: 5 }) {
                changed = true;
                break;
            }
        }

        assert!(changed, "Earthquake should randomly destroy or spawn obstacles");
    }

}
#[cfg(test)]
    mod evolution_tests {
    use super::*;
    use crate::game::{Difficulty, GameMode, Theme};
    #[test]
    fn test_evolve_game_of_life() {
        let mut game = Game::new(20, 20, false, 'x', Theme::Classic, Difficulty::Normal);
        game.mode = GameMode::Evolution;
        game.obstacles.clear();
        game.obstacles.insert(Point {
            x: 2,
            y: 1,
        });
        game.obstacles.insert(Point {
            x: 3,
            y: 2,
        });
        game.obstacles.insert(Point {
            x: 1,
            y: 3,
        });
        game.obstacles.insert(Point {
            x: 2,
            y: 3,
        });
        game.obstacles.insert(Point {
            x: 3,
            y: 3,
        });
        game.snake = crate::snake::Snake::new(Point {
            x: 10,
            y: 10,
        });
        game.player2 = None;
        game.food = Point {
            x: 15,
            y: 15,
        };
        game.bonus_food = None;
        game.power_up = None;
        game.evolve_game_of_life();
        assert!(!game.obstacles.contains(&Point {
            x: 2,
            y: 1
        }));
        assert!(game.obstacles.contains(&Point {
            x: 1,
            y: 2
        }));
        assert!(game.obstacles.contains(&Point {
            x: 3,
            y: 2
        }));
        assert!(game.obstacles.contains(&Point {
            x: 2,
            y: 3
        }));
        assert!(game.obstacles.contains(&Point {
            x: 3,
            y: 3
        }));
        assert!(game.obstacles.contains(&Point {
            x: 2,
            y: 4
        }));
    }
}

#[test]
fn test_massive_multiplayer_mode_spawns_bots() {
    let mut game = Game::new(
        40,
        40,
        false,
        '#',
        crate::game::Theme::Dark,
        crate::game::Difficulty::Normal,
    );
    game.mode = GameMode::MassiveMultiplayer;
    game.reset();

    // Verify 50 bots were spawned successfully
    assert_eq!(game.bots.len(), 50);
    assert_eq!(game.bots_autopilot_paths.len(), 50);

    // Verify autopilot works for massive multiplayer
    game.auto_pilot = true;
    game.update();
}

#[test]
fn test_bot_dies_when_hitting_wall() {
    let mut game = Game::new(
        20,
        20,
        false,
        '#',
        crate::game::Theme::Dark,
        crate::game::Difficulty::Normal,
    );
    game.mode = GameMode::MassiveMultiplayer;
    game.reset();

    // Force bot position to hit wall
    let mut bot = game.bots[0].clone();
    bot.body.clear();
    bot.body.push_front(Point { x: 1, y: 1 });
    bot.direction = Direction::Up;
    bot.direction_queue.push_back(Direction::Up);

    game.bots[0] = bot;

    let bot_count_before = game.bots.len();
    game.update(); // Tick should calculate next pos (1, 0), which is a wall (hit_wall)

    assert!(game.bots.len() < bot_count_before);
}
