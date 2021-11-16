use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_startup_system(setup_physics.system())
        .add_startup_system(setup_assets.system())
        .run();
}

fn setup_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(ClearColor(Color::rgb(0.4, 0.4, 1.0)));
    let mut camera = OrthographicCameraBundle::new_2d();
    commands.spawn_bundle(camera);

    let texture_handle = asset_server.load("happy.png");
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(texture_handle.into()),
        ..Default::default()
    });
}

fn setup_physics(mut commands: Commands, mut config: ResMut<RapierConfiguration>) {
    config.scale = 10.0;
    //config.gravity = vector![0.0, 0.0];
    commands.spawn_bundle(ColliderBundle {
        shape: ColliderShape::cuboid(100.0, 0.1),
        ..Default::default()
    });

    commands
        .spawn_bundle(RigidBodyBundle {
            position: Vec2::new(0.0, 5.0).into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(2.0),
            material: ColliderMaterial {
                restitution: 0.7,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::default());
}
