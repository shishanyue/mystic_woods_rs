use bevy::prelude::*;

use super::Toward;

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Direction {
    pub val: Vec3,
}

impl Default for Direction {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Direction {
    /// All zeroes.
    pub const ZERO: Self = Self::splat(0.0);

    /// A unit vector pointing along the positive X axis.
    pub const RIGHT: Self = Self::new(1.0, 0.0, 0.0);

    /// A unit vector pointing along the positive Y axis.
    pub const UP: Self = Self::new(0.0, 1.0, 0.0);

    /// A unit vector pointing along the negative X axis.
    pub const LEFT: Self = Self::new(-1.0, 0.0, 0.0);

    /// A unit vector pointing along the negative Y axis.
    pub const DOWN: Self = Self::new(0.0, -1.0, 0.0);

    #[inline]
    #[must_use]
    pub const fn splat(v: f32) -> Self {
        Self {
            val: Vec3::splat(v),
        }
    }

    /// Creates a new direction.
    #[inline(always)]
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            val: Vec3::new(x, y, z),
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn from_toward(toward: &Toward) -> Self {
        match toward {
            Toward::Up => Direction::UP,
            Toward::Down => Direction::DOWN,
            Toward::Left => Direction::LEFT,
            Toward::Right => Direction::RIGHT,
        }
    }
}

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Attack;
