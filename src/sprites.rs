//! Placeholder sprite colors and definitions.
//!
//! This module centralizes all visual definitions so they can easily be
//! swapped out for real sprite assets later.

#![allow(dead_code)]

use bevy::prelude::*;

/// Tile colors (16x16 tiles)
pub mod tiles {
    use super::*;

    pub const AIR: Color = Color::srgb(0.529, 0.808, 0.922); // Sky blue
    pub const SURFACE: Color = Color::srgb(0.133, 0.545, 0.133); // Forest green
    pub const DIRT: Color = Color::srgb(0.545, 0.271, 0.075); // Saddle brown
    pub const TUNNEL: Color = Color::srgb(0.3, 0.3, 0.3); // Dark gray
    pub const CHAMBER: Color = Color::srgb(0.4, 0.35, 0.3); // Tan
    pub const FUNGUS_GARDEN: Color = Color::srgb(0.35, 0.35, 0.3); // Gray with hint of green
    pub const TREE_TRUNK: Color = Color::srgb(0.4, 0.26, 0.13); // Dark brown bark
    pub const TREE_CANOPY: Color = Color::srgb(0.18, 0.42, 0.18); // Dark green leaves
}

/// Ant colors and sizes
pub mod ants {
    use super::*;

    // Colors
    pub const QUEEN: Color = Color::srgb(0.15, 0.1, 0.05); // Very dark brown
    pub const FORAGER: Color = Color::srgb(0.6, 0.3, 0.15); // Reddish brown
    pub const GARDENER: Color = Color::srgb(0.5, 0.35, 0.2); // Light brown
    pub const SOLDIER: Color = Color::srgb(0.25, 0.15, 0.08); // Dark brown

    // Sizes (in pixels)
    pub const QUEEN_SIZE: f32 = 12.0;
    pub const FORAGER_SIZE: f32 = 8.0;
    pub const GARDENER_SIZE: f32 = 6.0;
    pub const SOLDIER_SIZE: f32 = 10.0;
}

/// Egg/larva/pupa colors and sizes
pub mod brood {
    use super::*;

    pub const EGG: Color = Color::srgb(0.95, 0.95, 0.9); // Off-white
    pub const LARVA: Color = Color::srgb(0.92, 0.9, 0.85); // Cream
    pub const PUPA: Color = Color::srgb(0.7, 0.6, 0.5); // Tan

    pub const EGG_SIZE: f32 = 4.0;
    pub const LARVA_SIZE: f32 = 5.0;
    pub const PUPA_SIZE: f32 = 6.0;
}

/// Resource/object colors and sizes
pub mod objects {
    use super::*;

    pub const LEAF_FRAGMENT: Color = Color::srgb(0.3, 0.7, 0.2); // Bright green
    pub const MULCH: Color = Color::srgb(0.25, 0.35, 0.15); // Dark green-brown
    pub const FUNGUS: Color = Color::srgb(0.9, 0.85, 0.7); // Pale yellow-white

    pub const LEAF_SIZE: f32 = 6.0;
    pub const MULCH_SIZE: f32 = 8.0;
    pub const FUNGUS_SIZE: f32 = 6.0;
}

/// Pheromone overlay colors (semi-transparent)
pub mod pheromones {
    use super::*;

    pub const DIG: Color = Color::srgba(1.0, 0.5, 0.0, 0.4); // Orange, 40% opacity
    pub const FORAGE: Color = Color::srgba(0.2, 0.8, 0.2, 0.4); // Green, 40% opacity
    pub const HOME: Color = Color::srgba(0.4, 0.3, 0.8, 0.4); // Purple-blue, 40% opacity
    pub const AVOID: Color = Color::srgba(0.8, 0.2, 0.2, 0.4); // Red, 40% opacity
}

/// UI colors
pub mod ui {
    use super::*;

    pub const TEXT: Color = Color::srgb(0.9, 0.9, 0.9); // Light gray
    pub const BACKGROUND: Color = Color::srgba(0.1, 0.1, 0.1, 0.8); // Dark, semi-transparent
    pub const HIGHLIGHT: Color = Color::srgb(1.0, 0.8, 0.2); // Gold/yellow
}
