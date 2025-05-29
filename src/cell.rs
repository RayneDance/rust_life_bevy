use bevy::prelude::*;
#[derive(Component)]
pub struct CellAlive(pub bool);

#[derive(Component)]
pub struct CellColor(pub Color);

#[derive(Component)]
pub struct BoardData {
    pub width: i32,
    pub height: i32,
}

#[derive(Resource)]
pub struct Cells(pub Vec<Entity>);

#[derive(Resource)]
pub struct CellMesh(pub Handle<Mesh>);

#[derive(Component)]
pub struct IsBoard;

#[derive(Component)]
pub struct IsBackground;
