use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use std::ops::Add;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .add_system(update_camera.system());
    }
}

#[derive(Default)]
pub struct CameraAttention {}

#[derive(Bundle)]
pub struct GameCamera {
    #[bundle]
    camera: PerspectiveCameraBundle,
}

impl GameCamera {
    pub fn new() -> Self {
        Self {
            camera: PerspectiveCameraBundle {
                transform: Transform::from_xyz(0.0, 0.0, 100.0)
                    .looking_at(vec3(0., 0., 0.), Vec3::Y),
                ..Default::default()
            },
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(GameCamera::new());
}

fn update_camera(
    mut q: QuerySet<(
        Query<(&Camera, &mut Transform)>,
        Query<(&CameraAttention, &Transform)>,
    )>,
) {
    let attention_translation = q
        .q1()
        .single()
        .expect("There should be one CameraAttention.")
        .1
        .translation
        .clone();
    for (_, mut cam_tf) in q.q0_mut().iter_mut() {
        cam_tf.translation = attention_translation.add(vec3(0., 0., 100.));
    }
}
