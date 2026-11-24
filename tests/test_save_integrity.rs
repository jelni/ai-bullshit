use snake_game::game::{Difficulty, EggType, Game, GameMode, InGameUpgrade, SaveState, Theme};
use snake_game::snake::Point;

fn base_game(width: u16, height: u16) -> Game {
    let mut game = Game::new(width, height, false, 'x', Theme::Classic, Difficulty::Normal);
    game.xp_to_next_level = u32::MAX;
    game
}

fn save_to_temp(game: &Game, name: &str) -> (std::path::PathBuf, String) {
    let path = std::env::temp_dir().join(format!(
        "snake_save_integrity_{}_{}.json",
        name,
        std::process::id()
    ));
    game.save_game_to_file(path.to_str().unwrap());
    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("save file must exist after save_game_to_file: {e}"));
    (path, json)
}

#[test]
fn test_save_succeeds_with_eggs_on_board() {
    let mut game = base_game(20, 20);
    game.eggs_on_board.insert(Point { x: 3, y: 4 }, EggType::Common);
    game.eggs_on_board.insert(Point { x: 7, y: 8 }, EggType::Legendary);

    let (path, json) = save_to_temp(&game, "eggs");
    let state: SaveState = serde_json::from_str(&json).expect("saved JSON must parse as SaveState");
    let _ = std::fs::remove_file(&path);

    assert_eq!(state.eggs_on_board, game.eggs_on_board);
    assert_eq!(state.eggs_on_board.len(), 2);
}

#[test]
fn test_save_succeeds_with_painted_tiles() {
    let mut game = base_game(20, 20);
    game.mode = GameMode::TurfWar;
    game.reset();
    game.painted_tiles.insert(Point { x: 3, y: 3 }, 1);
    game.painted_tiles.insert(Point { x: 4, y: 3 }, 2);

    let (path, json) = save_to_temp(&game, "painted");
    let state: SaveState = serde_json::from_str(&json).expect("saved JSON must parse as SaveState");
    let _ = std::fs::remove_file(&path);

    assert_eq!(state.painted_tiles, game.painted_tiles);
    assert_eq!(state.painted_tiles.len(), 2);
}

#[test]
fn test_save_succeeds_with_dungeon_grid() {
    let mut game = base_game(20, 20);
    game.mode = GameMode::DungeonCrawler;
    game.reset();
    assert!(
        !game.dungeon_grid.is_empty(),
        "DungeonCrawler reset should generate a dungeon grid"
    );

    let (path, json) = save_to_temp(&game, "dungeon");
    let state: SaveState = serde_json::from_str(&json).expect("saved JSON must parse as SaveState");
    let _ = std::fs::remove_file(&path);

    assert_eq!(state.dungeon_grid.len(), game.dungeon_grid.len());
    assert_eq!(
        state.dungeon_grid.keys().collect::<std::collections::HashSet<_>>(),
        game.dungeon_grid.keys().collect::<std::collections::HashSet<_>>()
    );
    let (coords, room) = game.dungeon_grid.iter().next().unwrap();
    let loaded = state.dungeon_grid.get(coords).unwrap();
    assert_eq!(loaded.r_type, room.r_type);
    assert_eq!(loaded.cleared, room.cleared);
}

#[test]
fn test_save_succeeds_with_in_game_upgrades() {
    let mut game = base_game(20, 20);
    game.in_game_upgrades.insert(InGameUpgrade::Multishot, 2);

    let (path, json) = save_to_temp(&game, "upgrades");
    let state: SaveState = serde_json::from_str(&json).expect("saved JSON must parse as SaveState");
    let _ = std::fs::remove_file(&path);

    assert_eq!(state.in_game_upgrades, game.in_game_upgrades);
}

#[test]
fn test_saved_state_reserializes() {
    let mut game = base_game(20, 20);
    game.eggs_on_board.insert(Point { x: 3, y: 4 }, EggType::Rare);
    game.painted_tiles.insert(Point { x: 5, y: 5 }, 3);

    let (path, json) = save_to_temp(&game, "reserialize");
    let state: SaveState = serde_json::from_str(&json).unwrap();
    let _ = std::fs::remove_file(&path);

    let json2 = serde_json::to_string(&state).expect("a parsed SaveState must re-serialize");
    let state2: SaveState = serde_json::from_str(&json2).unwrap();
    assert_eq!(state2.eggs_on_board, state.eggs_on_board);
    assert_eq!(state2.painted_tiles, state.painted_tiles);
}

#[test]
fn test_legacy_empty_object_maps_still_load() {
    let mut game = base_game(20, 20);
    game.mode = GameMode::TurfWar;
    game.reset();
    game.painted_tiles.insert(Point { x: 3, y: 3 }, 1);
    game.eggs_on_board.insert(Point { x: 6, y: 6 }, EggType::Common);

    let (path, json) = save_to_temp(&game, "legacy");
    let _ = std::fs::remove_file(&path);

    // Simulate a legacy save written before the fix, where these maps were
    // serialized as (necessarily empty) JSON objects
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    for field in ["painted_tiles", "eggs_on_board", "dungeon_grid"] {
        value[field] = serde_json::json!({});
    }
    let legacy = serde_json::to_string(&value).unwrap();
    let state: SaveState =
        serde_json::from_str(&legacy).expect("legacy saves with empty object maps must load");

    assert!(state.painted_tiles.is_empty());
    assert!(state.eggs_on_board.is_empty());
    assert!(state.dungeon_grid.is_empty());
}
