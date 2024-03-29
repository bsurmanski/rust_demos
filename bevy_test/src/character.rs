use bevy::{math::*, prelude::shape::Quad, prelude::*};
use bevy_rapier2d::prelude::*;
use std::default::Default;

use crate::physics_object::PhysicsObjectBundle;
use crate::planet::Gravity;
use crate::ship::Ship;

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_character)
            .add_system(enter_exit_vehicle);
    }
}

#[derive(Component, Default)]
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
    pub pbr_bundle: PbrBundle,
}

impl CharacterBundle {
    pub fn new(
        position: Vec2,
        meshs: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        let hx = 1.;
        let mut physics_object = PhysicsObjectBundle::new(ColliderShape::cuboid(hx, hx));
        physics_object.rigid_body.position = position.into();
        Self {
            physics_object,
            pbr_bundle: PbrBundle {
                mesh: meshs.add(
                    Quad {
                        size: vec2(2. * hx, 2. * hx),
                        flip: false,
                    }
                    .into(),
                ),
                material: materials.add(asset_server.load("happy.png").into()),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/*
fn spawn_character(commands: Commands, materials: Res<Assets<ColorMaterial>>) {
    commands
        .spawn_bundle(CharacterBundle::new(vec2(11., 0.), sprite))
        .insert(crate::camera::CameraAttention {});
} */

fn enter_exit_vehicle(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut characters: Query<(
        Entity,
        &mut Character,
        &mut Transform,
        &mut Visibility,
        &mut RigidBodyPositionComponent,
        &mut RigidBodyVelocityComponent,
        &mut ColliderFlagsComponent,
        Option<&mut Gravity>,
    )>,
    vehicles: Query<(Entity, &Ship, &Transform), Without<Character>>,
) {
    for (
        char_entity,
        mut char,
        mut char_tf,
        mut visibility,
        mut rb_pos,
        mut rb_vel,
        mut collider_flags,
        mut opt_gravity,
    ) in characters.iter_mut()
    {
        // Enter/Exit Vehicle
        if keyboard_input.just_pressed(KeyCode::F) {
            // Exit
            if let Some(active_vehicle) = char.active_vehicle {
                let (_, ship, ship_tf) = vehicles.get(active_vehicle).unwrap();
                char.active_vehicle = None;
                visibility.is_visible = true;
                collider_flags.collision_groups = Default::default();
                if let Some(gravity) = opt_gravity.as_mut() {
                    gravity.is_active = true;
                }
                commands
                    .entity(char_entity)
                    .insert(crate::camera::CameraAttention {});
                commands
                    .entity(active_vehicle)
                    .remove::<crate::camera::CameraAttention>();
                rb_pos.position = ship_tf.translation.xy().into();
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
                    if closest_distsq.sqrt() < 10. {
                        char.active_vehicle = Some(closest.0);
                        visibility.is_visible = false;
                        collider_flags.collision_groups = InteractionGroups::none();
                        if let Some(gravity) = opt_gravity.as_mut() {
                            gravity.is_active = false;
                        }
                        commands
                            .entity(char_entity)
                            .remove::<crate::camera::CameraAttention>();
                        commands
                            .entity(closest.0)
                            .insert(crate::camera::CameraAttention {});
                        rb_vel.angvel = 0.;
                        rb_vel.linvel = vec2(0., 0.).into();
                    }
                }
            }
        }
    }
}

fn move_character(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut characters: Query<(
        &Character,
        &Transform,
        &mut RigidBodyVelocityComponent,
        Option<&mut Gravity>,
    )>,
) {
    for (char, char_tf, mut rb_vel, opt_gravity) in characters.iter_mut() {
        // If in vehicle
        if let Some(_) = char.active_vehicle {
            //TODO: do i need to do anything here?
            //TODO: handle error
            //let (_, _, ship_tf) = vehicles.get(active_vehicle).unwrap();
        } else {
            // Not in vehicle
            let grav_down = opt_gravity
                .map(|g| g.down)
                .unwrap_or(vec2(0., -1.))
                .normalize_or_zero();
            let char_down = char_tf.rotation * vec3(0., -1., 0.);
            let cw_or_ccw = grav_down.extend(0.).cross(char_down).dot(vec3(0., 0., -1.));
            // TODO: don't directly set the angvel. Add a force or something.
            rb_vel.angvel = cw_or_ccw * time.delta_seconds() * 300.;
            if keyboard_input.pressed(KeyCode::A) {
                let char_left = char_tf.rotation * vec3(-1., 0., 0.);
                rb_vel.linvel += Vector::<Real>::from(char_left.xy() * time.delta_seconds() * 20.);
            }
            if keyboard_input.pressed(KeyCode::D) {
                let char_right = char_tf.rotation * vec3(1., 0., 0.);
                rb_vel.linvel += Vector::<Real>::from(char_right.xy() * time.delta_seconds() * 20.);
            }
        }
    }
}
