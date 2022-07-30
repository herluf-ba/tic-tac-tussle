use bevy::prelude::*;

use crate::AppState;

type TileIndex = usize;

#[derive(Component)]
struct HoverDot(pub TileIndex);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn board background
    commands.spawn_bundle(SpriteBundle {
        transform: Transform::from_xyz(0.0, -30.0, 0.0),
        sprite: Sprite {
            custom_size: Some(Vec2::new(480.0, 480.0)),
            ..default()
        },
        texture: asset_server.load("background.png").into(),
        ..default()
    });

    for x in 0..3 {
        for y in 0..3 {
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform::from_xyz(
                        160.0 * (x as f32 - 1.0),
                        -30.0 + 160.0 * (y as f32 - 1.0),
                        0.0,
                    ),
                    sprite: Sprite {
                        color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                        custom_size: Some(Vec2::new(160.0, 160.0)),
                        ..default()
                    },
                    texture: asset_server.load("dot.png").into(),
                    ..default()
                })
                .insert(HoverDot(x + y * 3));
        }
    }

    commands.spawn_bundle(SpriteBundle {
        transform: Transform::from_xyz(-160.0, -30.0, 0.0),
        sprite: Sprite {
            custom_size: Some(Vec2::new(160.0, 160.0)),
            ..default()
        },
        texture: asset_server.load("tac.png").into(),
        ..default()
    });

    commands.spawn_bundle(SpriteBundle {
        transform: Transform::from_xyz(160.0, -30.0, 0.0),
        sprite: Sprite {
            custom_size: Some(Vec2::new(160.0, 160.0)),
            ..default()
        },
        texture: asset_server.load("tic.png").into(),
        ..default()
    });
}

fn update(windows: Res<Windows>, mut hover_dots: Query<(&HoverDot, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    if let Some(mouse_position) = window.cursor_position() {
        // Determine the index of the tile that the mouse is currently over
        let x_tile: usize = (mouse_position.x / 160.0).floor() as usize;
        let y_tile: usize = (mouse_position.y / 160.0).floor() as usize;
        let tile = x_tile + y_tile * 3;

        // If mouse is outside of board so we do nothing further
        if 8 < tile {
            return;
        }

        // Toggle hover dots on and off
        for (dot, mut dot_sprite) in hover_dots.iter_mut() {
            if dot.0 == tile {
                dot_sprite.color.set_a(1.0);
            } else {
                dot_sprite.color.set_a(0.0);
            }
        }

        // TODO: Spawn events on click
    }
}

fn cleanup() {
    // TODO: remove game sprites and ui
}

pub struct StateInGame;
impl Plugin for StateInGame {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup));
        app.add_system_set(SystemSet::on_update(AppState::InGame).with_system(update));
        app.add_system_set(SystemSet::on_exit(AppState::InGame).with_system(cleanup));
    }
}
