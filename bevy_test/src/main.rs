use bevy::{math::*, pbr::AmbientLight, prelude::*};
use bevy_rapier2d::prelude::*;
use bevy_svg::prelude::*;
use std::default::Default;

mod camera;
mod character;
mod physics_object;
mod planet;
mod ship;
mod ui;
use crate::character::CharacterBundle;
use crate::planet::{Orbit, PlanetBundle};
use crate::ship::ShipBundle;

fn main() {
    App::new()
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
        .add_startup_system_to_stage(StartupStage::PostStartup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ambient_light: ResMut<AmbientLight>,
    mut config: ResMut<RapierConfiguration>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    config.gravity = vector![0.0, 0.0];
    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 1.0;
    commands.insert_resource(ClearColor(Color::rgb(0.4, 0.6, 0.9)));

    commands
        .spawn_bundle(ShipBundle::new_from_xy(10.0, 11.0))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("ship.gltf#Scene0"));
        });

    let scale = 10.;

    commands.spawn_bundle(PlanetBundle::generate(
        scale,
        12.,
        &mut mesh_assets,
        &mut material_assets,
    ));

    commands
        .spawn_bundle(CharacterBundle::new(
            vec2(11., 0.),
            &mut mesh_assets,
            &mut material_assets,
            &asset_server,
        ))
        .insert(crate::camera::CameraAttention {});
}
