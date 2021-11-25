use bevy::{math::*, pbr::AmbientLight, prelude::*};
use bevy_rapier2d::prelude::*;
use std::default::Default;
use std::time::Duration;

mod camera;
mod physics_object;
mod planet;
mod character;
use crate::physics_object::*;
use crate::planet::*;
use crate::character::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_plugin(crate::planet::PlanetaryPlugin)
        .add_plugin(crate::camera::CameraPlugin)
        .add_startup_system(setup.system())
        .add_system(move_ship.system())
        .run();
}

#[derive(Default)]
struct Ship; // Marker Component

#[derive(Bundle, Default)]
struct ShipBundle {
    pub transform: Transform,
    pub global_tranform: GlobalTransform,

    ship: Ship,
    gravity: Gravity,
    #[bundle]
    physics_object: PhysicsObjectBundle,
}

impl ShipBundle {
    fn new() -> Self {
        Self {
            physics_object: PhysicsObjectBundle::new(ColliderShape::ball(10.0)),
            ..Default::default()
        }
    }

    fn new_from_xy(x: f32, y: f32) -> Self {
        let mut n = Self::new();
        n.physics_object.collider.position = [x, y].into();
        return n;
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
    mut config: ResMut<RapierConfiguration>,
) {
    //config.scale = 10.0;
    config.gravity = vector![0.0, 0.0];
    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 1.0;
    commands.insert_resource(ClearColor(Color::rgb(0.4, 0.6, 0.9)));
    commands
        .spawn_bundle(ShipBundle::new_from_xy(100.0, 110.0))
        .insert(crate::camera::CameraAttention {})
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("ship.gltf#Scene0"));
        });

    let scale = 100.;
    commands
        .spawn_bundle(PlanetBundle::new(scale, 10., Default::default()))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("big_planet.gltf#Scene0"));

            let scale = 50.;
            parent
                .spawn_bundle(PlanetBundle::new(
                    scale,
                    5.,
                    Orbit::new_elliptical(300., 500., Vec2::new(1., 1.), Duration::from_secs(20)),
                ))
                .insert_bundle(PbrBundle {
                    mesh: asset_server.load("planet.gltf#Mesh0/Primitive0"),
                    material: asset_server.load("planet.gltf#Material0"),
                    transform: Transform::from_scale(Vec3::new(scale, scale, scale)),
                    ..Default::default()
                });
        });

    let sprite = materials.add(asset_server.load("happy.png").into());
    commands.spawn_bundle(CharacterBundle::new(sprite));
}

fn move_ship(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Ship, &Transform, &mut RigidBodyForces)>,
) {
    for (_, ship_tf, mut forces) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::W) {
            let up = ship_tf.rotation.mul_vec3(Vec3::new(0.0, 20000.0, 0.0));
            forces.force = up.xy().into();
        }
        if keyboard_input.pressed(KeyCode::A) {
            forces.torque = 80000.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            forces.torque = -80000.0;
        }
    }
}
