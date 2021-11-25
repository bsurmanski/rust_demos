use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Bundle, Default)]
pub struct PhysicsObjectBundle {
    #[bundle]
    pub rigid_body: RigidBodyBundle,
    #[bundle]
    pub collider: ColliderBundle,

    pub position_sync: ColliderPositionSync,
}

impl PhysicsObjectBundle {
    pub fn new(shape: SharedShape) -> Self {
        Self {
            collider: ColliderBundle {
                shape,
                material: ColliderMaterial {
                    restitution: 0.7,
                    ..Default::default()
                },
                ..Default::default()
            },
            position_sync: ColliderPositionSync::Discrete,
            ..Default::default()
        }
    }
}
