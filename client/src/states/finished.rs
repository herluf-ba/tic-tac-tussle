use bevy::prelude::*;

use crate::{AppState, UIRoot};

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        // A container that centers its children on the screen
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                // Bevy UI is placing children such that first child goes at the bottom
                // This is the opposite of how a browser does it, but we can get back to
                // familiarity by just using ColumnReverse 👍
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(UIRoot)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Something went wrong! please restart TicTacTussle 🙏",
                TextStyle {
                    font: asset_server.load("Inconsolata.ttf"),
                    font_size: 24.0,
                    color: Color::hex("d65d0e").unwrap(),
                },
            ));
        });
}

fn cleanup(mut commands: Commands, ui_root: Query<Entity, With<UIRoot>>) {
    commands
        .entity(ui_root.get_single().unwrap())
        .despawn_recursive();
}

pub struct StateFinished;
impl Plugin for StateFinished {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Error).with_system(setup));
        app.add_system_set(SystemSet::on_exit(AppState::Error).with_system(cleanup));
    }
}
