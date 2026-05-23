#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AStarState {
    pub f_score: u16,
    pub position: crate::snake::Point,
}
impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .f_score
            .cmp(&self.f_score)
            .then_with(|| other.position.x.cmp(&self.position.x))
            .then_with(|| other.position.y.cmp(&self.position.y))
    }
}
impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
