use std::cell::RefMut;
use std::collections::HashMap;
use bevy::prelude::*;
use crate::{App, GameState, SystemSet};
use crate::game::player::setup_player;
use crate::game::world::world::*;

mod player;
mod world;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Game)
            .with_system(setup_game).with_system(setup_world));
        app.add_system_set(SystemSet::on_update(GameState::Game)
            .with_system(player::update_controller).with_system(update_world).with_system(load_chunks));
        app.init_resource::<world::world::World>();
    }
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    setup_player(commands, meshes, materials);
}