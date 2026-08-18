import re

with open("src/main.rs", "r") as f:
    content = f.read()

# Settings Left (1565) -> 0, 1, 2, 3
content = content.replace("2 => {\n                let themes", "1 => {\n                let themes")
content = content.replace("3 => game.wrap_mode", "2 => game.wrap_mode")
content = content.replace("4 => {\n                let skins", "3 => {\n                let skins")

# Settings Right (1593) -> 0, 1, 2, 3
content = content.replace(
"""            match game.settings_selection {
                0 => {
                    game.difficulty = game.difficulty.next();
                    game.update_high_scores();
                },
                1 => {
                    let themes = &game.stats.unlocked_themes;
                    let current_idx = themes.iter().position(|&t| t == game.theme).unwrap_or(0);
                    let next_idx = (current_idx + 1) % themes.len();
                    game.theme = themes[next_idx];
                },
                2 => game.wrap_mode = !game.wrap_mode,
                3 => {
                    let skins = &game.stats.unlocked_skins;
                    let current_idx = skins.iter().position(|&c| c == game.skin).unwrap_or(0);
                    let next_idx = (current_idx + 1) % skins.len();
                    game.skin = skins[next_idx];
                },""",
"""            match game.settings_selection {
                0 => {
                    game.difficulty = game.difficulty.next();
                    game.update_high_scores();
                },
                1 => {
                    let themes = &game.stats.unlocked_themes;
                    let current_idx = themes.iter().position(|&t| t == game.theme).unwrap_or(0);
                    let next_idx = (current_idx + 1) % themes.len();
                    game.theme = themes[next_idx];
                },
                2 => game.wrap_mode = !game.wrap_mode,
                3 => {
                    let skins = &game.stats.unlocked_skins;
                    let current_idx = skins.iter().position(|&c| c == game.skin).unwrap_or(0);
                    let next_idx = (current_idx + 1) % skins.len();
                    game.skin = skins[next_idx];
                },"""
)


with open("src/main.rs", "w") as f:
    f.write(content)
