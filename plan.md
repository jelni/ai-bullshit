1. **Pre commit instructions**:
    - Ran `./clippy.sh` and observed a `similar_names` warning between `new_p_elo` and `new_b_elo`.
    - Fixed the warning by renaming `new_b_elo` to `new_bot_elo`.
    - All tests pass via `cargo test`.
    - `cargo test --test '*'` passed.
2. Submit
