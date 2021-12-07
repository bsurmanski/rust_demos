use bevy::{math::*, pbr::AmbientLight, prelude::*};
use bevy_rapier2d::prelude::*;
use std::default::Default;
use std::time::Duration;

mod camera;
mod character;
mod physics_object;
mod planet;
mod ship;
use crate::character::CharacterBundle;
use crate::planet::PlanetBundle;
use crate::ship::ShipBundle;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_plugin(crate::planet::PlanetaryPlugin)
        .add_plugin(crate::camera::CameraPlugin)
        .add_plugin(crate::ship::ShipPlugin)
        .add_plugin(crate::character::CharacterPlugin)
        .add_startup_system(setup.system())
        .run();
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
        .spawn_bundle(ShipBundle::new_from_xy(10.0, 11.0))
        .insert(crate::camera::CameraAttention {})
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("ship.gltf#Scene0"));
        });

    let scale = 10.;
    commands
        .spawn_bundle(PlanetBundle::new(scale, 10., Default::default()))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("big_planet.gltf#Scene0"));

            /*
            let scale = 5.;
            parent
                .spawn_bundle(PlanetBundle::new(
                    scale,
                    4.,
                    Orbit::new_elliptical(40., 70., Vec2::new(1., 1.), Duration::from_secs(20)),
                ))
                .insert_bundle(PbrBundle {
                    mesh: asset_server.load("planet.gltf#Mesh0/Primitive0"),
                    material: asset_server.load("planet.gltf#Material0"),
                    transform: Transform::from_scale(Vec3::new(scale, scale, scale)),
                    ..Default::default()
                });  */
        });

    let sprite = materials.add(asset_server.load("happy.png").into());
    commands.spawn_bundle(CharacterBundle::new(sprite));
}
