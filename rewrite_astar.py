import re

with open('src/game.rs', 'r') as f:
    content = f.read()

new_astar = """    pub fn calculate_autopilot_path(&self,) -> Option<(Direction, std::collections::VecDeque<Point>)> {
        let start = self.snake.head();

        let mut targets = vec![self.food];
        if let Some((bf_p, _,),) = self.bonus_food {
            targets.push(bf_p,);
        }
        if let Some(pu,) = &self.power_up
            && pu.activation_time.is_none()
        {
            targets.push(pu.location,);
        }

        let mut open_set = std::collections::BinaryHeap::new();
        let mut g_score = std::collections::HashMap::new();
        let mut came_from = std::collections::HashMap::new();

        g_score.insert(start, 0,);

        let heuristic = |p: Point| -> u16 {
            let can_pass_through_walls = self.power_up.as_ref().is_some_and(|pu| {
                pu.p_type == PowerUpType::PassThroughWalls
                    && pu.activation_time.is_some_and(|time| {
                        time.elapsed().unwrap_or_default() < Duration::from_secs(5,)
                    },)
            },);
            targets
                .iter()
                .map(|t| {
                    let mut dx = p.x.abs_diff(t.x,);
                    let mut dy = p.y.abs_diff(t.y,);
                    if self.wrap_mode || can_pass_through_walls {
                        dx = std::cmp::min(dx, self.width.saturating_sub(2,).saturating_sub(dx,),);
                        dy = std::cmp::min(dy, self.height.saturating_sub(2,).saturating_sub(dy,),);
                    }
                    dx + dy
                },)
                .min()
                .unwrap_or(0,)
        };

        let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right,];
        open_set.push(AStarState {
            f_score: heuristic(start,),
            position: start,
        },);

        while let Some(AStarState {
            position: current,
            ..
        },) = open_set.pop()
        {
            if targets.contains(&current,) {
                // Reconstruct path
                let mut path = std::collections::VecDeque::new();
                let mut curr = current;
                while let Some(&(prev, dir)) = came_from.get(&curr) {
                    path.push_front(curr);
                    curr = prev;
                    if curr == start {
                        return Some((dir, path));
                    }
                }
                // Fallback if current == start (shouldn't happen if targets != start)
            }

            let current_g = *g_score.get(&current,).unwrap_or(&u16::MAX,);

            for &d in &dirs {
                let next_p = Self::calculate_next_head_dir(current, d,);
                if let Some(final_p,) = self.get_final_p(next_p,)
                    && self.is_safe_final_p(final_p,)
                {
                    let tentative_g = current_g.saturating_add(1,);
                    if tentative_g < *g_score.get(&final_p,).unwrap_or(&u16::MAX,) {
                        came_from.insert(final_p, (current, d));
                        g_score.insert(final_p, tentative_g,);
                        open_set.push(AStarState {
                            f_score: tentative_g.saturating_add(heuristic(final_p,),),
                            position: final_p,
                        },);
                    }
                }
            }
        }

        // Fallback: Just return any safe direction if no path to target is found
        for &d in &dirs {
            let next_p = Self::calculate_next_head_dir(start, d,);
            if let Some(final_p,) = self.get_final_p(next_p,)
                && self.is_safe_final_p(final_p,)
            {
                let mut path = std::collections::VecDeque::new();
                path.push_back(final_p);
                return Some((d, path));
            }
        }
        None
    }"""

pattern = r'    pub fn calculate_autopilot_move\(&self,\) -> Option<Direction,> \{.*?(?=\n    const fn calculate_wrapped_head)'
content = re.sub(pattern, new_astar, content, flags=re.DOTALL)

with open('src/game.rs', 'w') as f:
    f.write(content)
