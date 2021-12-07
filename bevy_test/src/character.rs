use bevy::{math::*, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::physics_object::PhysicsObjectBundle;
use crate::planet::Gravity;
use crate::ship::Ship;

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(move_character.system());
    }
}

#[derive(Default)]
pub struct Character {
    pub active_vehicle: Option<Entity>,
}

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
            physics_object: PhysicsObjectBundle::new(ColliderShape::cuboid(5., 5.)),
            sprite: SpriteBundle {
                material,
                transform: Transform::from_scale(Vec3::new(0.2, 0.2, 0.2)),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

fn move_character(
    keyboard_input: Res<Input<KeyCode>>,
    mut characters: Query<(
        &mut Character,
        &mut Transform,
        &mut RigidBodyVelocity,
        &mut Visible,
        &mut ColliderFlags,
    )>,
    vehicles: Query<(Entity, &Ship, &Transform), Without<Character>>,
) {
    for (mut char, mut char_tf, mut rb_vel, mut visible, mut collider_flags) in
        characters.iter_mut()
    {
        // Enter/Exit Vehicle
        if keyboard_input.just_pressed(KeyCode::F) {
            // Exit
            if let Some(active_vehicle) = char.active_vehicle {
                char.active_vehicle = None;
                visible.is_visible = true;
                collider_flags.collision_groups = Default::default();
            } else {
                // Enter
                let mut closest = None;
                let mut closest_distsq = 0.;
                for (e, ship, ship_tf) in vehicles.iter() {
                    let dsq = char_tf
                        .translation
                        .xy()
                        .distance_squared(ship_tf.translation.xy());
                    if closest.is_none() || dsq < closest_distsq {
                        closest = Some((e, ship, ship_tf));
                        closest_distsq = dsq;
                    }
                }

                // If we found a vehicle to get into, hide player and collision.
                if let Some(closest) = closest {
                    char.active_vehicle = Some(closest.0);
                    visible.is_visible = false;
                    collider_flags.collision_groups = InteractionGroups::none();
                }
            }
        }

        // If in vehicle
        if let Some(active_vehicle) = char.active_vehicle {
            //TODO: handle error
            let (_, _, ship_tf) = vehicles.get(active_vehicle).unwrap();
        }
    }
}
