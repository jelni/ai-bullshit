use super::{ShopItem, Theme};
pub const AVAILABLE_ITEMS: [(ShopItem, u32); 16] = [
    (ShopItem::Skin('💎'), 100),
    (ShopItem::Skin('👾'), 250),
    (ShopItem::Skin('🐍'), 500),
    (ShopItem::Skin('🚀'), 1000),
    (ShopItem::Skin('🦍'), 2000),
    (ShopItem::Skin('₿'), 5000),
    (ShopItem::Skin('Ξ'), 10_000),
    (ShopItem::Skin('Ð'), 25_000),
    (ShopItem::Theme(Theme::Premium), 5000),
    (ShopItem::Theme(Theme::Cyberpunk), 10_000),
    (ShopItem::Theme(Theme::Rainbow), 25_000),
    (ShopItem::Theme(Theme::Hacker), 50_000),
    (ShopItem::Theme(Theme::Blockchain), 100_000),
    (ShopItem::Theme(Theme::Esports), 250_000),
    (ShopItem::Theme(Theme::Solar), 500_000),
    (ShopItem::Theme(Theme::Metaverse), 1_000_000),
];
