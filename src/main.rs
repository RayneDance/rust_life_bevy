// Global constants for board configuration
pub const GRID_WIDTH: i32 = 250;
pub const GRID_HEIGHT: i32 = 250;
pub const CELL_SIZE: f32 = 10.0;
pub const TOTAL_CELLS: usize = (GRID_WIDTH * GRID_HEIGHT) as usize;

// Pastel color palette
pub const PASTEL_MINT: Color = Color::rgb(0.6, 0.9, 0.7);
pub const PASTEL_LAVENDER: Color = Color::rgb(0.8, 0.7, 0.9);
pub const PASTEL_PEACH: Color = Color::rgb(1.0, 0.8, 0.7);
pub const PASTEL_SKY: Color = Color::rgb(0.7, 0.8, 0.9);
pub const PASTEL_LEMON: Color = Color::rgb(1.0, 0.9, 0.6);
pub const PASTEL_ROSE: Color = Color::rgb(0.9, 0.7, 0.8);
pub const PASTEL_CORAL: Color = Color::rgb(1.0, 0.7, 0.6);
pub const PASTEL_AQUA: Color = Color::rgb(0.6, 0.9, 0.8);

lazy_static::lazy_static! {
    pub static ref CELL_COLORS: Vec<Color> = vec![
        PASTEL_MINT,
        PASTEL_LAVENDER,
        PASTEL_PEACH,
        PASTEL_SKY,
        PASTEL_LEMON,
        PASTEL_ROSE,
        PASTEL_CORAL,
        PASTEL_AQUA,
    ];
}

use bevy::{
    prelude::*,
    render::camera::Viewport,
    color::palettes::css::BLACK,
};

use bevy::ecs::relationship::RelationshipSourceCollection;
use bevy::prelude::ops::powf;
use crate::cell::{CellMesh, Cells};
use rand::Rng;

mod cell;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            ..default()
        }))
        .insert_resource(Cells(Vec::new()))
        .insert_resource(CellMesh(
            Handle::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update_viewport)
        .add_systems(Update, game_of_life)
        .add_systems(FixedUpdate, (controls, reset_request))
        .run();
}

fn game_of_life(
    cell_ids: Res<Cells>,
    mut cell_comps: Query<(&mut cell::CellAlive, &mut MeshMaterial2d<ColorMaterial>, &mut Transform)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut next_states: Vec<bool> = Vec::with_capacity(TOTAL_CELLS);
    let cells = &cell_ids.0;

    for i in 0..cells.len() {
        let entity_id = cells[i];
        let current_row = i / GRID_WIDTH as usize;
        let current_col = i % GRID_WIDTH as usize;
        let mut live_neighbors = 0;

        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let mut neighbor_row_signed = current_row as i32 + dr;
                let mut neighbor_col_signed = current_col as i32 + dc;

                if neighbor_row_signed < 0 {
                    neighbor_row_signed = GRID_HEIGHT - 1;
                } else if neighbor_row_signed >= GRID_HEIGHT {
                    neighbor_row_signed = 0;
                }

                if neighbor_col_signed < 0 {
                    neighbor_col_signed = GRID_WIDTH - 1;
                } else if neighbor_col_signed >= GRID_WIDTH {
                    neighbor_col_signed = 0;
                }

                let neighbor_row = neighbor_row_signed as usize;
                let neighbor_col = neighbor_col_signed as usize;
                let neighbor_flat_index = neighbor_row * GRID_WIDTH as usize + neighbor_col;

                if neighbor_flat_index < cells.len() {
                    let neighbor_entity_id = cells[neighbor_flat_index];
                    if let Ok((neighbor_alive_comp, _, _)) = cell_comps.get(neighbor_entity_id) {
                        if neighbor_alive_comp.0 {
                            live_neighbors += 1;
                        }
                    }
                }
            }
        }

        let current_cell_is_alive = match cell_comps.get(entity_id) {
            Ok((alive_comp, _, _)) => alive_comp.0,
            Err(_) => {
                error!("Current Entity {:?} could not be read. Treating as dead.", entity_id);
                false
            }
        };

        let goes_to_next_state_alive = if current_cell_is_alive {
            live_neighbors == 2 || live_neighbors == 3
        } else {
            live_neighbors == 3
        };
        next_states.push(goes_to_next_state_alive);
    }

    for i in 0..cells.len() {
        let entity_id = cells[i];
        let new_alive_status = next_states[i];

        if let Ok((mut alive_comp, material_comp, _transform)) = cell_comps.get_mut(entity_id) {
            if alive_comp.0 != new_alive_status {
                alive_comp.0 = new_alive_status;
                let material_handle = &material_comp.0; // Access the handle from MeshMaterial2d
                if let Some(material_asset) = materials.get_mut(material_handle) {
                    if new_alive_status && !alive_comp.0 {
                        // Cell becoming alive - assign random pastel color
                        let mut rng = rand::rng();
                        let color_idx = rng.gen_range(0..CELL_COLORS.len());
                        material_asset.color = CELL_COLORS[color_idx];
                    } else if !new_alive_status {
                        // Cell dying - turn black
                        material_asset.color = Color::from(BLACK);
                    }
                    // If cell stays alive, keep its current color
                }
            }
        }
    }
}

fn reset_request(
    mut cell_comps: Query<(&mut cell::CellAlive, &mut MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        let mut rng = rand::rng();
        for (mut alive_comp, material) in cell_comps.iter_mut() {
            alive_comp.0 = rng.random_bool(0.7);
            let material_handle = &material.0;
            if let Some(material_asset) = materials.get_mut(material_handle) {
                if alive_comp.0 {
                    let color_idx = rng.gen_range(0..CELL_COLORS.len());
                    material_asset.color = CELL_COLORS[color_idx];
                } else {
                    material_asset.color = Color::from(BLACK);
                }
            }
        }
    }
}


fn controls(
    mut camera_query: Query<(&mut Camera, &mut Transform, &mut Projection)>,
    window: Query<&Window>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
){

    let Ok(_) = window.single() else {
        return;
    };
    let Ok((mut _camera, mut transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    let fspeed = 600.0 * time.delta_secs();

    if input.pressed(KeyCode::KeyW) {
        transform.translation.y += fspeed;
    }
    if input.pressed(KeyCode::KeyS) {
        transform.translation.y -= fspeed;
    }
    if input.pressed(KeyCode::KeyA) {
        transform.translation.x -= fspeed;
    }
    if input.pressed(KeyCode::KeyD) {
        transform.translation.x += fspeed;
    }
    if let Projection::Orthographic(projection2d) = &mut *projection {
        if input.pressed(KeyCode::KeyQ) {
            projection2d.scale *= powf(4.0f32, time.delta_secs());
        }

        if input.pressed(KeyCode::KeyE) {
            projection2d.scale *= powf(0.25f32, time.delta_secs());
        }
    }
}

fn update_viewport(
    mut camera_query: Query<&mut Camera>,
    window: Single<&Window>,
) {
    let Ok(mut camera) = camera_query.single_mut() else {
        return;
    };
    let physical_size = window.resolution.physical_size();
    camera.viewport = Some(Viewport {
        physical_size,
        ..default()
    });
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cells: ResMut<Cells>,
    mut cell_mesh: ResMut<CellMesh>,
    window: Single<&Window>
) {
    
    let mut rng = rand::rng();
    commands.spawn((
        Camera2d,
        Transform {
            translation: Vec3::new(
                500.,
                500.,
                1000.,
            ),
            ..default()
        },
        Camera {
            viewport: Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: window.resolution.physical_size(),
                ..default()
            }),
            ..default()
        },
    ));

    cell_mesh.0 = meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE));
    for i in 0..TOTAL_CELLS {
        let alive = rng.random_bool(0.7);
        let color_idx = rng.gen_range(0..CELL_COLORS.len());
        let cell_color = if alive { CELL_COLORS[color_idx] } else { Color::from(BLACK) };
        cells.0.add(commands.spawn((
                Mesh2d(cell_mesh.0.clone()),
                cell::CellAlive(alive),
                MeshMaterial2d(materials.add(cell_color)),
                Transform::from_translation(Vec3::new(
                    (i % GRID_WIDTH as usize) as f32 * CELL_SIZE,
                    (i / GRID_WIDTH as usize) as f32 * CELL_SIZE,
                    0.0,
                )),
        )).id());
    }
}