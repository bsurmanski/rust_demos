use bevy::{math::*, pbr::AmbientLight, prelude::*};
use bevy_rapier2d::prelude::*;
use bevy_svg::prelude::*;
use std::default::Default;
use std::time::Duration;

mod camera;
mod character;
mod physics_object;
mod planet;
mod ship;
mod ui;
use crate::character::{CharacterAssets, CharacterBundle};
use crate::planet::{Orbit, PlanetBundle};
use crate::ship::ShipBundle;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_plugin(SvgPlugin)
        .add_plugin(crate::planet::PlanetaryPlugin)
        .add_plugin(crate::camera::CameraPlugin)
        .add_plugin(crate::ship::ShipPlugin)
        .add_plugin(crate::character::CharacterPlugin)
        .add_plugin(crate::ui::UiPlugin)
        .add_startup_system_to_stage(StartupStage::PostStartup, setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ambient_light: ResMut<AmbientLight>,
    mut config: ResMut<RapierConfiguration>,
    char_assets: Res<CharacterAssets>,
    mesh_assets: ResMut<Assets<Mesh>>,
    material_assets: ResMut<Assets<StandardMaterial>>,
) {
    config.gravity = vector![0.0, 0.0];
    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 1.0;
    commands.insert_resource(ClearColor(Color::rgb(0.4, 0.6, 0.9)));

    commands
        .spawn_bundle(ShipBundle::new_from_xy(10.0, 11.0))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("ship.gltf#Scene0"));
            /*
            parent.spawn_bundle(SvgBundle {
                svg: asset_server.load("circle.svg"),
                origin: Origin::Center,
                ..Default::default()
            });*/
        });

    let mut scale = 10.;

    commands.spawn_bundle(PlanetBundle::generate(scale, 12., mesh_assets, material_assets));
    /*
    commands
        .spawn_bundle(PlanetBundle::new(scale, 15., Default::default()))
        .with_children(|parent| {
            parent.spawn_bundle(SvgBundle {
                svg: asset_server.load("circle.svg"),
                origin: Origin::Center,
                transform: Transform::from_scale(vec3(scale, scale, scale) * 2.),
                ..Default::default()
            });
        });*/

        /*
    scale = 60.;
    commands
        .spawn_bundle(PlanetBundle::new(
            scale,
            10.,
            Orbit::new(100., Duration::from_secs(60)),
        ))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("big_planet.gltf#Scene0"));
            parent.spawn_bundle(SvgBundle {
                svg: asset_server.load("circle.svg"),
                origin: Origin::Center,
                transform: Transform::from_scale(vec3(scale, scale, scale) * 2.),
                ..Default::default()
            });
        });*/

    //parent.spawn_scene(asset_server.load("big_planet.gltf#Scene0"));

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
            transform: Transform::from_scale(vec3(scale, scale, scale)),
            ..Default::default()
        });  */
    commands
        .spawn_bundle(CharacterBundle::new(vec2(11., 0.), char_assets))
        .insert(crate::camera::CameraAttention {});
}
