1. **Fix Non-Exhaustive Patterns in `src/web/ui.rs`**
   - The match on `boss.kind` is missing new boss types like `Splitter`, `Trapper`, `Necromancer`, and `ShadowClone`. Add colors for them.

2. **Fix `src/ui.rs`**
   - Address the non-exhaustive patterns and other drawing issues matching what the CLI uses for the boss colors or a suitable equivalent in the console. Actually, let's fix the clippy error in `src/ui.rs` by removing `.into()` in `Color::DarkGrey.into()`.

3. **Fix `src/game.rs`**
   - Fix `clippy::comparison_chain` by using a `match` instead of an `if-else if-else` block for `cmp`.
   - Fix `clippy::collapsible_if` in speedrun replay writing.

4. **Review Pre-commit Steps**
   - Run `pre_commit_instructions` tool to perform proper testing, verifications, reviews and reflections.

5. **Submit Changes**
   - Commit and push changes.
