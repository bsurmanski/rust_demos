use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use std::ops::Add;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(update_camera);
    }
}

#[derive(Component, Default)]
pub struct CameraAttention {}

#[derive(Component, Default)]
pub struct GameCamera {}

#[derive(Default, Bundle)]
pub struct GameCameraBundle {
    #[bundle]
    camera: PerspectiveCameraBundle,

    game_camera: GameCamera, // marker component
}

impl GameCameraBundle {
    pub fn new() -> Self {
        Self {
            camera: PerspectiveCameraBundle {
                transform: Transform::from_xyz(0.0, 0.0, 100.0)
                    .looking_at(vec3(0., 0., 0.), Vec3::Y),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(GameCameraBundle::new());
}

fn update_camera(
    mut q: QuerySet<(
        QueryState<(&Camera, &mut Transform), With<GameCamera>>,
        QueryState<(&CameraAttention, &Transform)>,
    )>,
) {
    let attention_translation = q
        .q1()
        .get_single()
        .expect("There should be one CameraAttention.")
        .1
        .translation
        .clone();
    for (_, mut cam_tf) in q.q0().iter_mut() {
        cam_tf.translation = attention_translation.add(vec3(0., 0., 100.));
    }
}
