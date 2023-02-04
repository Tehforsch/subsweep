#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod vis;

use bevy::prelude::*;
use raxiom::components::Position;
use raxiom::prelude::*;
use raxiom::units::VecLength;
use raxiom::voronoi::DelaunayTriangulation;
use vis::DrawTriangle;

fn main() {
    let mut app = App::new();
    app.add_startup_system(add_points_system)
        .add_startup_system(setup_camera_system)
        .add_startup_system_to_stage(StartupStage::PostStartup, show_voronoi_system)
        .add_plugins(DefaultPlugins)
        .run();
}

fn add_points_system(mut commands: Commands) {
    for i in 0..10 {
        for j in 0..10 {
            commands.spawn((
                LocalParticle,
                Position(VecLength::meters(i as f64 * 0.1, j as f64 * 0.1)),
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
    for t in triangulation.tetras {
        let triangle = DrawTriangle {
            p1: triangulation.points[t.p1],
            p2: triangulation.points[t.p2],
            p3: triangulation.points[t.p3],
        };
        commands.spawn(ColorMesh2dBundle {
            mesh: meshes.add(triangle.get_mesh()).into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::from_scale(Vec3::splat(100.0)),
            ..default()
        });
    }
}
