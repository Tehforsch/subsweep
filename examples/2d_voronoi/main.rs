#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod vis;

use bevy::prelude::*;
use raxiom::components::Position;
use raxiom::prelude::*;
use raxiom::units::VecLength;
use raxiom::voronoi::DelaunayTriangulation;
use vis::DrawTriangle;

const SCALE: f64 = 900.0;

fn main() {
    let mut app = App::new();
    app.add_startup_system(add_points_system)
        .add_startup_system(setup_camera_system)
        .add_startup_system_to_stage(StartupStage::PostStartup, show_voronoi_system)
        .add_plugins(DefaultPlugins)
        .run();
}

fn add_points_system(mut commands: Commands) {
    let n_x = 3;
    let n_y = 3;
    for i in 0..n_x {
        for j in 0..n_y {
            commands.spawn((
                LocalParticle,
                Position(VecLength::meters(
                    (i as f64 - n_x as f64 / 2.0) * 0.1,
                    (j as f64 - n_y as f64 / 2.0) as f64 * 0.1,
                )),
            ));
        }
    }
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn show_voronoi_system(
    mut commands: Commands,
    particles: Particles<&Position>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let triangulation = DelaunayTriangulation::construct(
        &particles
            .into_iter()
            .map(|x| x.value_unchecked())
            .collect::<Vec<_>>(),
    );
    for p in particles.iter() {
        let c = DrawCircle::from_position_and_color(**p, RColor::BLUE);
        commands.spawn(ColorMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(5.0).into()).into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::from_translation(
                SCALE as f32
                    * Vec3::new(
                        p.x().value_unchecked() as f32,
                        p.y().value_unchecked() as f32,
                        1.0,
                    ),
            ),
            ..default()
        });
    }
    for t in triangulation.tetras {
        let triangle = DrawTriangle {
            p1: triangulation.points[t.p1] * SCALE,
            p2: triangulation.points[t.p2] * SCALE,
            p3: triangulation.points[t.p3] * SCALE,
        };
        commands.spawn(ColorMesh2dBundle {
            mesh: meshes.add(triangle.get_mesh()).into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            ..default()
        });
    }
}
