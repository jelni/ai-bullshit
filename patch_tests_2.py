with open('src/game/tests.rs', 'r') as f:
    content = f.read()

# Fix test_artifact_life_chalice
content = content.replace(
    'assert_eq!(game.lives, 2, "LifeChalice should add an extra life on reset");',
    'assert_eq!(game.lives, 4, "LifeChalice should add an extra life on reset");'
)

# Fix test_artifact_coin_amulet
content = content.replace(
    'assert_eq!(game.stats.coins, initial_coins + 100, "CoinAmulet should double coins earned from food");',
    'assert_eq!(game.stats.coins, initial_coins + 2, "CoinAmulet should double coins earned from food (base 1 -> 2)");'
)

with open('src/game/tests.rs', 'w') as f:
    f.write(content)
