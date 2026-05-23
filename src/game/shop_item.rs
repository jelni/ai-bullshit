use super::Theme;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShopItem {
    Skin(char),
    Theme(Theme),
}
