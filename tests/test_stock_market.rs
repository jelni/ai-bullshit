use snake_game::game::Game;

#[test]
fn test_stock_market_price_fluctuations_and_bounds() {
    let mut game = Game::new(
        40,
        20,
        false,
        'O',
        snake_game::game::Theme::Dark,
        snake_game::game::Difficulty::Normal,
    );

    // Simulate enough ticks/updates to ensure stock market events trigger and clamp logic is hit.
    // update_stock_market has a 1% chance (rng.gen_bool(0.01)) of triggering per call.
    // Calling it 10000 times will almost certainly trigger it many times.
    for _ in 0..10_000 {
        game.update_stock_market();
    }

    // Verify all stock prices are within bounds [5, 2000].
    for stock in [
        snake_game::game::Stock::SnakeCorp,
        snake_game::game::Stock::GoblinInc,
        snake_game::game::Stock::BossDynamics,
        snake_game::game::Stock::LaserTech,
    ] {
        let price = game.stats.stock_prices.get(&stock).copied().unwrap_or(100);
        assert!(price >= 5, "Price {} for {:?} dropped below minimum 5", price, stock);
        assert!(price <= 2000, "Price {} for {:?} exceeded maximum 2000", price, stock);
    }
}
