# User Interface

The terminal UI is managed largely in `src/ui.rs` and styled using `src/color.rs`.

- Uses `crossterm` to handle terminal resizing, raw mode input (keyboard events), and cursor positioning.
- Draws specific menus like the Main Menu, Settings, Game Over screen, and an Auction House.
- **Color Palettes:** Various unlockable themes modify the visual representation of the snake and background elements.
