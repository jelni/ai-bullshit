#!/bin/bash
sed -i 's/pub mod companion;/pub mod companion;\npub mod hero_class;/' src/game/mod.rs
sed -i 's/pub use companion::{Companion, CompanionType};/pub use companion::{Companion, CompanionType};\npub use hero_class::HeroClass;/' src/game/mod.rs
