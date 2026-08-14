import re

with open("src/game/game_struct.rs", "r") as f:
    content = f.read()

def replace_zombie_margin_if_needed(content):
    # Fix the spawn_zombie logic in process_food_collision (lines 8320+)
    # It has:
    #             if spawn_zombie {
    #                 let margin = self.safe_zone_margin;
    #                 let mut zombie_margin = margin;
    #                 if zombie_margin > 0 && (self.width < 10 || self.height < 10) {
    #                     zombie_margin = 0; // prevent None in small grids
    #                 }

    # We will replace that with:
    #             if spawn_zombie {
    #                 let mut zombie_margin = self.safe_zone_margin;
    #                 if zombie_margin > 0 && (self.width < 10 || self.height < 10) {
    #                     zombie_margin = 0; // prevent None in small grids
    #                 }

    content = content.replace(
        """            if spawn_zombie {
                let margin = self.safe_zone_margin;
                let mut zombie_margin = margin;
                if zombie_margin > 0 && (self.width < 10 || self.height < 10) {
                    zombie_margin = 0; // prevent None in small grids
                }""",
        """            if spawn_zombie {
                let mut zombie_margin = self.safe_zone_margin;
                if zombie_margin > 0 && (self.width < 10 || self.height < 10) {
                    zombie_margin = 0; // prevent None in small grids
                }"""
    )

    # And the else branch:
    #         } else {
    #             self.gain_xp(1);
    #
    #             if spawn_zombie {
    #                 let margin = self.safe_zone_margin;
    #                 if let Some(pos) = Self::get_random_empty_point(

    # Needs to become:
    #         } else {
    #             self.gain_xp(1);
    #
    #             if spawn_zombie {
    #                 let mut zombie_margin = self.safe_zone_margin;
    #                 if zombie_margin > 0 && (self.width < 10 || self.height < 10) {
    #                     zombie_margin = 0; // prevent None in small grids
    #                 }
    #                 if let Some(pos) = Self::get_random_empty_point(

    content = content.replace(
        """        } else {
            self.gain_xp(1);

            if spawn_zombie {
                let margin = self.safe_zone_margin;
                if let Some(pos) = Self::get_random_empty_point(
                    self.width,
                    self.height,
                    &self.snake,
                    |p: &Point| {
                        self.obstacles.contains(p)
                            || self.snake.body_map.contains_key(p)
                            || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
                            || self.bots.iter().any(|b| b.body_map.contains_key(p))
                    },
                    &mut self.rng,
                    margin,
                ) {""",
        """        } else {
            self.gain_xp(1);

            if spawn_zombie {
                let mut zombie_margin = self.safe_zone_margin;
                if zombie_margin > 0 && (self.width < 10 || self.height < 10) {
                    zombie_margin = 0; // prevent None in small grids
                }
                if let Some(pos) = Self::get_random_empty_point(
                    self.width,
                    self.height,
                    &self.snake,
                    |p: &Point| {
                        self.obstacles.contains(p)
                            || self.snake.body_map.contains_key(p)
                            || self.player2.as_ref().is_some_and(|p2| p2.body_map.contains_key(p))
                            || self.bots.iter().any(|b| b.body_map.contains_key(p))
                    },
                    &mut self.rng,
                    zombie_margin,
                ) {"""
    )

    # Fix the other instance in process bot movement
    #                     if self.mode == GameMode::Zombie {
    #                         let margin = self.safe_zone_margin;
    #                         if let Some(pos) = Self::get_random_empty_point(

    content = content.replace(
        """                    if self.mode == GameMode::Zombie {
                        let margin = self.safe_zone_margin;
                        if let Some(pos) = Self::get_random_empty_point(
                            self.width,
                            self.height,
                            &self.snake,
                            |p: &Point| {
                                self.obstacles.contains(p)
                                    || self.snake.body_map.contains_key(p)
                                    || self
                                        .player2
                                        .as_ref()
                                        .is_some_and(|p2| p2.body_map.contains_key(p))
                                    || self.bots.iter().any(|b| b.body_map.contains_key(p))
                            },
                            &mut self.rng,
                            margin,
                        ) {""",
        """                    if self.mode == GameMode::Zombie {
                        let mut zombie_margin = self.safe_zone_margin;
                        if zombie_margin > 0 && (self.width < 10 || self.height < 10) {
                            zombie_margin = 0;
                        }
                        if let Some(pos) = Self::get_random_empty_point(
                            self.width,
                            self.height,
                            &self.snake,
                            |p: &Point| {
                                self.obstacles.contains(p)
                                    || self.snake.body_map.contains_key(p)
                                    || self
                                        .player2
                                        .as_ref()
                                        .is_some_and(|p2| p2.body_map.contains_key(p))
                                    || self.bots.iter().any(|b| b.body_map.contains_key(p))
                            },
                            &mut self.rng,
                            zombie_margin,
                        ) {"""
    )

    return content

new_content = replace_zombie_margin_if_needed(content)

with open("src/game/game_struct.rs", "w") as f:
    f.write(new_content)
