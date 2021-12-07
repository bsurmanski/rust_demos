use bevy::{math::*, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::planet::Gravity;
use crate::physics_object::PhysicsObjectBundle;

pub struct ShipPlugin;
impl Plugin for ShipPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(move_ship.system());
    }
}

#[derive(Default)]
pub struct Ship; // Marker Component

#[derive(Bundle, Default)]
pub struct ShipBundle {
    pub transform: Transform,
    pub global_tranform: GlobalTransform,

    ship: Ship,
    gravity: Gravity,
    #[bundle]
    physics_object: PhysicsObjectBundle,
}

impl ShipBundle {
    pub fn new() -> Self {
        Self {
            physics_object: PhysicsObjectBundle::new(ColliderShape::cuboid(1., 1.)),
            ..Default::default()
        }
    }

    pub fn new_from_xy(x: f32, y: f32) -> Self {
        let mut n = Self::new();
        n.physics_object.collider.position = [x, y].into();
        return n;
    }
}

fn move_ship(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Ship, &Transform, &mut RigidBodyVelocity)>,
) {
    for (_, ship_tf, mut rb_vel) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::W) {
            let fwd = ship_tf.rotation.mul_vec3(Vec3::new(0., 1., 0.));
            rb_vel.linvel += Vector::<Real>::from(fwd.xy() * 0.3);
        }
        if keyboard_input.pressed(KeyCode::S) {
            let bwd = ship_tf.rotation.mul_vec3(Vec3::new(0., -1., 0.));
            rb_vel.linvel += Vector::<Real>::from(bwd.xy() * 0.3);
        }
        if keyboard_input.pressed(KeyCode::A) {
            rb_vel.angvel += 10. * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::D) {
            rb_vel.angvel -= 10. * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::F) {

        }
    }
}