use bevy::{math::*, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::physics_object::PhysicsObjectBundle;
use crate::planet::Gravity;

#[derive(Default)]
pub struct Character; // Marker Component

#[derive(Bundle, Default)]
pub struct CharacterBundle {
    pub character: Character,
    pub gravity: Gravity,
    #[bundle]
    pub physics_object: PhysicsObjectBundle,

    #[bundle]
    pub sprite: SpriteBundle,
}

impl CharacterBundle {
    pub fn new(material: Handle<ColorMaterial>) -> Self {
        Self {
            physics_object: PhysicsObjectBundle::new(ColliderShape::cuboid(6.4, 6.4)),
            sprite: SpriteBundle {
                material,
                transform: Transform::from_scale(Vec3::new(0.2, 0.2, 0.2)),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
