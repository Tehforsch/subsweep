#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod camera;
mod vis;

use bevy::prelude::*;
use glam::DVec2;
use raxiom::components::Position;
use raxiom::prelude::*;
use raxiom::units::VecLength;
use raxiom::voronoi::DelaunayTriangulation;
use raxiom::voronoi::TetraIndex;
use raxiom::voronoi::VoronoiGrid;
use vis::DrawPolygon;
use vis::DrawTriangle;

use crate::camera::setup_camera_system;
use crate::camera::track_mouse_world_position_system;
use crate::camera::MousePosition;

const HIGHLIGHT_LAYER: f32 = -0.1;
const INTERMEDIATE_LAYER: f32 = -0.5;
const LOW_LAYER: f32 = -2.0;

#[derive(Resource)]
struct Colors {
    red: Handle<ColorMaterial>,
    blue: Handle<ColorMaterial>,
    green: Handle<ColorMaterial>,
}

#[derive(Component, Debug)]
struct VisTriangle {
    index: TetraIndex,
}

#[derive(Component, Debug)]
struct VisPolygon;

#[derive(Component, Debug)]
struct VisCircle;

#[derive(Debug)]
struct RedrawEvent;

fn main() {
    let mut app = App::new();
    app.add_startup_system(add_points_system)
        .add_startup_system(setup_camera_system)
        .add_system_to_stage(CoreStage::PreUpdate, show_voronoi_system)
        .add_system(highlight_triangle_system)
        .add_system(track_mouse_world_position_system)
        .add_system(spawn_points_system)
        .add_plugins(DefaultPlugins)
        .add_event::<RedrawEvent>()
        .run();
}

fn add_points_system(mut commands: Commands) {
    let n_x = 8;
    let n_y = 8;
    for i in 0..n_x {
        for j in 0..n_y {
            commands.spawn((
                LocalParticle,
                Position(VecLength::meters(
                    (i as f64 - n_x as f64 / 2.0 + (j as f64 * 0.6122 * i as f64 * 0.02)) * 0.1,
                    (j as f64 - n_y as f64 / 2.0 - i as f64 * 0.71123) as f64 * 0.1,
                )),
            ));
        }
    }
}

fn show_voronoi_system(
    mut commands: Commands,
    particles: Particles<&Position>,
    triangles: Query<(Entity, &VisTriangle)>,
    polys: Query<(Entity, &VisPolygon)>,
    circles: Query<(Entity, &VisCircle)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut redraw_events: EventReader<RedrawEvent>,
    triangulation: Option<Res<DelaunayTriangulation>>,
) {
    if triangulation.is_some() && redraw_events.iter().count() == 0 {
        return;
    }
    for (e, _) in triangles.iter() {
        commands.entity(e).despawn();
    }
    for (e, _) in polys.iter() {
        commands.entity(e).despawn();
    }
    for (e, _) in circles.iter() {
        commands.entity(e).despawn();
    }
    let colors = Colors {
        blue: materials.add(ColorMaterial::from(Color::BLUE)),
        red: materials.add(ColorMaterial::from(Color::RED)),
        green: materials.add(ColorMaterial::from(Color::GREEN)),
    };
    let triangulation = DelaunayTriangulation::construct(
        &particles
            .into_iter()
            .map(|x| x.value_unchecked())
            .collect::<Vec<_>>(),
    );
    let grid = VoronoiGrid::from(triangulation.clone());
    for cell in grid.cells.iter() {
        for vp in cell.points.iter() {
            commands.spawn((
                VisCircle,
                ColorMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(0.005).into()).into(),
                    material: colors.green.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        vp.x as f32,
                        vp.y as f32,
                        LOW_LAYER,
                    )),
                    ..default()
                },
            ));
        }
    }
    for p in particles.iter() {
        commands.spawn((
            VisCircle,
            ColorMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(0.005).into()).into(),
                material: colors.blue.clone(),
                transform: Transform::from_translation(Vec3::new(
                    p.x().value_unchecked() as f32,
                    p.y().value_unchecked() as f32,
                    LOW_LAYER,
                )),
                ..default()
            },
        ));
    }
    for (index, t) in triangulation.tetras.iter() {
        let triangle = DrawTriangle {
            p1: triangulation.points[t.p1],
            p2: triangulation.points[t.p2],
            p3: triangulation.points[t.p3],
        };
        commands
            .spawn(ColorMesh2dBundle {
                mesh: meshes.add(triangle.get_mesh()).into(),
                material: colors.red.clone(),
                ..default()
            })
            .insert(VisTriangle { index });
    }
    for cell in grid.cells.iter() {
        let poly = DrawPolygon {
            points: cell.points.clone(),
        };
        commands
            .spawn(ColorMesh2dBundle {
                mesh: meshes.add(poly.get_mesh()).into(),
                material: colors.red.clone(),
                ..default()
            })
            .insert(VisPolygon);
    }
    commands.insert_resource(triangulation);
    commands.insert_resource(colors);
}

fn highlight_triangle_system(
    mut particles: Query<(&VisTriangle, &mut Handle<ColorMaterial>, &mut Transform)>,
    triangulation: Res<DelaunayTriangulation>,
    colors: Res<Colors>,
    mouse_pos: Res<MousePosition>,
) {
    let index =
        triangulation.find_containing_tetra(DVec2::new(mouse_pos.0.x as f64, mouse_pos.0.y as f64));
    for (triangle, mut color, mut transform) in particles.iter_mut() {
        if Some(triangle.index) == index {
            *color = colors.red.clone();
            transform.translation.z = HIGHLIGHT_LAYER;
        } else {
            *color = colors.blue.clone();
            transform.translation.z = LOW_LAYER;
        };
    }
    if let Some(index) = index {
        let tetra = &triangulation.tetras[index];
        for face in tetra.iter_faces() {
            for (triangle, mut color, mut transform) in particles.iter_mut() {
                if Some(triangle.index) == face.opposing.as_ref().map(|opposing| opposing.tetra) {
                    *color = colors.green.clone();
                    transform.translation.z = INTERMEDIATE_LAYER;
                };
            }
        }
    }
}

fn spawn_points_system(
    mut commands: Commands,
    mouse_pos: Res<MousePosition>,
    mouse_input: Res<Input<MouseButton>>,
    mut redraw_events: EventWriter<RedrawEvent>,
) {
    for _ in mouse_input.get_just_pressed() {
        commands.spawn((
            LocalParticle,
            Position(VecLength::meters(
                mouse_pos.0.x as f64,
                mouse_pos.0.y as f64,
            )),
        ));
        redraw_events.send(RedrawEvent);
    }
}
