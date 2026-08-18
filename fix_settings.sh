sed -i 's/2 => {/1 => {/g' src/main.rs
sed -i 's/3 => game.wrap_mode = !game.wrap_mode,/2 => game.wrap_mode = !game.wrap_mode,/g' src/main.rs
sed -i 's/4 => {/3 => {/g' src/main.rs
