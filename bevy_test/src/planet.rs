use bevy::{math::*, prelude::*, render::render_resource::*};
use bevy_rapier2d::prelude::*;
use std::default::Default;
use std::time::Duration;
use std::{f32, f64};

use crate::physics_object::*;

fn mesh_from_polyline(shape: &Vec<Vec2>) -> Mesh {
    use lyon::math::{point, Point};
    use lyon::tessellation::*;

    let mut path_builder = lyon::path::Path::builder();
    path_builder.begin(point(shape[0].x, shape[0].y));
    for vert in shape.iter().skip(1) {
        path_builder.line_to(point(vert.x, vert.y));
    }

    path_builder.close();
    let path = path_builder.build();
    // Will contain the result of the tessellation.
    let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    tessellator
        .tessellate_path(
            &path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(
                &mut geometry,
                lyon::tessellation::geometry_builder::Positions,
            )
            .with_inverted_winding(),
        )
        .unwrap();

    let indices = bevy::render::mesh::Indices::U16(geometry.indices);
    let positions: Vec<[f32; 3]> = geometry.vertices.iter().map(|v| [v.x, v.y, 0.]).collect();
    let normals: Vec<[f32; 3]> = geometry.vertices.iter().map(|_| [0., 0., 1.]).collect();
    let uvs: Vec<[f32; 2]> = geometry.vertices.iter().map(|_| [0., 0.]).collect();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    return mesh;
}

struct PlanetaryConstants {
    // Distance for gravity to falloff 50%.
    // Measured as a ratio of the planet's radius.
    // The realistic value would be 1.41 (sqrt(2))
    // But with tiny planets this tends to feel way too quick.
    gravity_falloff_ratio: f32,
}

pub struct PlanetaryPlugin;
impl Plugin for PlanetaryPlugin {
    fn build(&self, app: &mut App) {
        let constants = PlanetaryConstants {
            gravity_falloff_ratio: 2.,
        };

        app.insert_resource(constants)
            .add_system(apply_planetary_gravity)
            //.add_system(dolly_planet)
            .add_system(orbit_satellites);
    }
}

// An entity that is affected by gravity.
#[derive(Component)]
pub struct Gravity {
    pub is_active: bool,
    pub down: Vec2,
}

impl Default for Gravity {
    fn default() -> Self {
        Self {
            is_active: true,
            down: vec2(0., -1.),
        }
    }
}

// TODO: allow phases other than periapsis at t=0
// TODO: allow CCW orbits
#[derive(Component, Default)]
pub struct Orbit {
    // lowest point in orbit, measured from the center of the reference.
    // This is distinct from periapsis because it includes the radius of the
    // orbited body.
    periapsis: f32,
    // highest point in orbit, measured from the center of reference.
    apoapsis: f32,
    // normalized orientation of the major axis.
    // Facing towards the periapsis.
    // In a perifocal coordinate system, this is the 'p' unit vector.
    major_orient: Vec2,
    // In a perifocal coordinate sytem, this is the 'q' unit vector.
    minor_orient: Vec2,
    // time to make one revolution.
    period: Duration,
}

impl Orbit {
    pub fn new(altitude: f32, period: Duration) -> Self {
        Self {
            periapsis: altitude,
            apoapsis: altitude,
            major_orient: Vec2::new(0., 1.),
            minor_orient: Vec2::new(1., 0.), // TODO: assumes orbit is clockwise
            period,
        }
    }

    pub fn new_elliptical(
        periapsis: f32,
        apoapsis: f32,
        major_orient: Vec2,
        period: Duration,
    ) -> Self {
        let orient = major_orient.try_normalize().unwrap_or(Vec2::new(1., 0.));
        Self {
            periapsis,
            apoapsis,
            major_orient: orient,
            minor_orient: Vec2::new(orient.y, -orient.x), // TODO: assumes orbit is clockwise
            period,
        }
    }

    // Average orbital motion in radians per second.
    pub fn mean_motion(&self) -> f32 {
        let period_secs = self.period.as_secs() as f32;
        if period_secs <= f32::EPSILON {
            return 0.;
        }
        return f32::consts::TAU / self.period.as_secs() as f32;
    }

    // mean_anomaly in radians.
    // t: time elapsed since periapsis.
    pub fn mean_anomaly(&self, t: Duration) -> f32 {
        return self.mean_motion() * t.as_secs_f32();
    }

    // From: https://en.wikipedia.org/wiki/Orbital_eccentricity
    pub fn eccentricity(&self) -> f32 {
        let denom = self.apoapsis + self.periapsis;
        if denom < f32::EPSILON {
            return 0.;
        }
        return (self.apoapsis - self.periapsis) / denom;
    }

    // Length of the semi-major axis.
    pub fn major_radius(&self) -> f32 {
        return (self.apoapsis + self.periapsis) / 2.;
    }

    // Length fo the semi-minor axis.
    // From: https://en.wikipedia.org/wiki/Semi-major_and_semi-minor_axes
    pub fn minor_radius(&self) -> f32 {
        let e = self.eccentricity();
        return self.major_radius() * (1. - e * e).sqrt();
    }

    // position assuming the primary focus is at (0, 0).
    // and the orbit is phased at the periapsis at t=0.
    pub fn get_position(&self, t: Duration) -> Vec2 {
        let anomaly = self.mean_anomaly(t); // TODO: use true anomaly

        // the origin is at the primary focus; Not at the center of the ellipse.
        let focus_offset = (self.periapsis - self.apoapsis) / 2. * self.major_orient;
        return focus_offset
            + (anomaly.cos() * self.major_radius() * self.major_orient)
            + (anomaly.sin() * self.minor_radius() * self.minor_orient);
    }
}

#[derive(Component, Default)]
pub struct Planet {
    pub radius: f32,
    pub gravity: f32, // acceleration
}

#[derive(Bundle, Default)]
pub struct PlanetBundle {
    pub planet: Planet,
    pub orbit: Orbit,

    #[bundle]
    pub physics_bundle: PhysicsObjectBundle,

    #[bundle]
    pub pbr_bundle: PbrBundle,
}

impl PlanetBundle {
    pub fn new(scale: f32, gravity: f32, orbit: Orbit) -> Self {
        let position = orbit.get_position(Duration::ZERO);
        Self {
            planet: Planet {
                radius: scale,
                gravity,
            },
            orbit,
            physics_bundle: PhysicsObjectBundle {
                rigid_body: RigidBodyBundle {
                    dominance: RigidBodyDominance(10).into(), // planets shouldn't be pushable
                    ..Default::default()
                },
                collider: ColliderBundle {
                    shape: ColliderShape::ball(scale).into(),
                    position: position.into(),
                    ..Default::default()
                },
                position_sync: ColliderPositionSync::Discrete,
                ..Default::default()
            },
            pbr_bundle: Default::default(),
        }
    }

    pub fn generate(
        radius: f32,
        gravity: f32,
        meshs: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Self {
        use noise::{Cycle, NoiseFn, Perlin};

        //let circum = radius * f32::consts::TAU; // 2 * PI * R
        let nsegments = 100; //(circum / 50.).max(12.).floor();
        let roughness = radius * 0.3; // radius will vary by +- 10%

        let perlin = Perlin::default();
        let cyclic_perlin = Cycle::new(perlin)
            .set_x_period(f64::consts::TAU)
            .set_y_period(100.);
        let mut path: Vec<Vec2> = vec![];
        let delta = f32::consts::TAU / nsegments as f32;
        for i in 0..(nsegments as usize) {
            let theta = delta * i as f32;
            let r = radius + roughness * cyclic_perlin.get([theta as f64, 8.1]) as f32;
            path.push(vec2(theta.cos() * r, theta.sin() * r));
        }

        let mesh = mesh_from_polyline(&path);
        let shape = {
            let points = path.iter().map(|v| Point::new(v.x, v.y)).collect();
            SharedShape::convex_polyline(points).unwrap()
        };

        let orbit = Orbit::default();
        let position = orbit.get_position(Duration::ZERO);
        Self {
            planet: Planet { radius, gravity },
            orbit,
            physics_bundle: PhysicsObjectBundle {
                rigid_body: RigidBodyBundle {
                    dominance: RigidBodyDominance(10).into(), // planets shouldn't be pushable
                    position: position.into(),
                    ..Default::default()
                },
                collider: ColliderBundle {
                    shape: shape.into(),
                    ..Default::default()
                },
                position_sync: ColliderPositionSync::Discrete,
                ..Default::default()
            },
            pbr_bundle: PbrBundle {
                mesh: meshs.add(mesh),
                material: materials.add(Color::rgb(1., 1., 1.).into()),
                ..Default::default()
            },
        }
    }
}

// Orbit satellite planets. Anything with an 'Orbit'.
fn orbit_satellites(
    time: Res<Time>,
    mut q: Query<(&Orbit, &Transform, &mut RigidBodyVelocityComponent)>,
) {
    for (orbit, planet_tf, mut rb_vel) in q.iter_mut() {
        let target_pos = orbit.get_position(time.time_since_startup());
        let delta = target_pos - planet_tf.translation.xy();
        rb_vel.linvel = Vector::<Real>::from(delta);
    }
}

// Some influence is taken from lighting calculations.
// See: https://imdoingitwrong.wordpress.com/2011/01/31/light-attenuation/
// and: https://gamedev.stackexchange.com/questions/131372/light-attenuation-formula-derivation
// and: https://docs.blender.org/manual/en/2.79/render/blender_render/lighting/lights/attenuation.html
fn apply_planetary_gravity(
    time: Res<Time>,
    constants: Res<PlanetaryConstants>,
    mut q0: Query<(&mut RigidBodyVelocityComponent, &Transform, &mut Gravity)>,
    q1: Query<(&Planet, &Transform)>,
) {
    for (mut rb_vel, rb_tf, mut gravity) in q0.iter_mut() {
        if !gravity.is_active {
            continue;
        }
        let mut gravity_vec = Vec2::ZERO;
        for (planet, planet_tf) in q1.iter() {
            let offset = planet_tf.translation.xy() - rb_tf.translation.xy();
            let dist = offset.length();
            // biased dist so planet's surface has target gravity.
            let surf_dist = 0f32.max(dist - planet.radius);
            let falloff_dist = planet.radius * constants.gravity_falloff_ratio;
            let falloff_distsq = falloff_dist * falloff_dist;
            // let radius_dist = dist / planet.radius;
            // let attenuation = 1. / (radius_dist * radius_dist).max(1.);
            //TODO: design attenuation so that the orbits don't violate Bertrand's Theorem:
            // https://en.wikipedia.org/wiki/Bertrand%27s_theorem
            // I think I'd have to use 1/r^2 with only scaling and offset.
            // probably something in the form of: (k / (x+c)^2).
            let attenuation = falloff_distsq / (falloff_distsq + surf_dist * surf_dist);
            gravity_vec += offset.normalize_or_zero() * planet.gravity * attenuation;
        }
        gravity.down = gravity_vec.normalize_or_zero();
        gravity_vec *= time.delta_seconds();
        rb_vel.linvel += Vector::<Real>::from(gravity_vec);
    }
}

fn dolly_planet(
    mut q: QuerySet<(
        QueryState<(&mut Transform, &Planet)>,
        QueryState<&Transform, With<crate::camera::CameraAttention>>,
    )>,
) {
    let attention_tf = q
        .q1()
        .get_single()
        .expect("Assume theres at least one CameraAttention.")
        .clone();
    for (mut planet_tf, planet) in q.q0().iter_mut() {
        let dist = planet_tf
            .translation
            .xy()
            .distance(attention_tf.translation.xy());
        let offset = -(dist - (planet.radius)) * 2.;
        planet_tf.translation = Vec3::new(planet_tf.translation.x, planet_tf.translation.y, offset);
    }
}
