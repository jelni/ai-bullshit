use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DungeonRoomType {
    Start,
    Normal,
    Boss,
    Treasure,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[expect(clippy::struct_excessive_bools)]
pub struct DungeonRoom {
    pub r_type: DungeonRoomType,
    pub cleared: bool,
    pub north_door: bool,
    pub south_door: bool,
    pub east_door: bool,
    pub west_door: bool,
}

impl DungeonRoom {
    #[must_use]
    pub const fn new(r_type: DungeonRoomType) -> Self {
        Self {
            r_type,
            cleared: false,
            north_door: false,
            south_door: false,
            east_door: false,
            west_door: false,
        }
    }
}

pub fn generate_dungeon(
    level: u32,
    rng: &mut rand::rngs::StdRng,
) -> HashMap<(i32, i32), DungeonRoom> {
    let mut grid = HashMap::new();
    let num_rooms = 5 + (level * 2);

    let start_pos = (0, 0);
    grid.insert(start_pos, DungeonRoom::new(DungeonRoomType::Start));

    let mut current_rooms = vec![start_pos];
    let mut num_generated = 1;

    let dirs = [(0, -1), (0, 1), (1, 0), (-1, 0)];

    while num_generated < num_rooms && !current_rooms.is_empty() {
        let idx = rng.gen_range(0..current_rooms.len());
        let pos = current_rooms[idx];

        let available_dirs: Vec<(i32, i32)> = dirs
            .iter()
            .copied()
            .filter(|&(dx, dy)| !grid.contains_key(&(pos.0 + dx, pos.1 + dy)))
            .collect();

        if available_dirs.is_empty() {
            current_rooms.remove(idx);
            continue;
        }

        let dir = available_dirs[rng.gen_range(0..available_dirs.len())];
        let new_pos = (pos.0 + dir.0, pos.1 + dir.1);

        grid.insert(new_pos, DungeonRoom::new(DungeonRoomType::Normal));
        current_rooms.push(new_pos);

        // Connect the rooms
        let mut room1 = grid.get(&pos).unwrap().clone();
        let mut room2 = grid.get(&new_pos).unwrap().clone();

        match dir {
            (0, -1) => {
                room1.north_door = true;
                room2.south_door = true;
            },
            (0, 1) => {
                room1.south_door = true;
                room2.north_door = true;
            },
            (1, 0) => {
                room1.east_door = true;
                room2.west_door = true;
            },
            (-1, 0) => {
                room1.west_door = true;
                room2.east_door = true;
            },
            _ => {},
        }

        grid.insert(pos, room1);
        grid.insert(new_pos, room2);

        num_generated += 1;
    }

    // Designate Boss and Treasure rooms (furthest from start)
    let mut sorted_by_dist: Vec<((i32, i32), u32)> =
        grid.keys().map(|&(x, y)| ((x, y), x.unsigned_abs() + y.unsigned_abs())).collect();
    sorted_by_dist.sort_by_key(|&(_, d)| std::cmp::Reverse(d));

    if let Some(&((bx, by), _)) = sorted_by_dist.first()
        && let Some(r) = grid.get_mut(&(bx, by))
    {
        r.r_type = DungeonRoomType::Boss;
    }

    if sorted_by_dist.len() > 1
        && let Some(&((tx, ty), _)) = sorted_by_dist.get(1)
        && let Some(r) = grid.get_mut(&(tx, ty))
    {
        r.r_type = DungeonRoomType::Treasure;
    }

    grid
}
