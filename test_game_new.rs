use snake_game::*;

fn main() {
    let mut game =
        game::Game::new(20, 20, false, 'x', game::Theme::Classic, game::Difficulty::Normal);
    println!("Obstacles: {:?}", game.obstacles.len());
}
