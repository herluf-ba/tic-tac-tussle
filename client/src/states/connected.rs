use bevy::prelude::*;

use crate::{AppState, UIRoot};

#[derive(Component)]
struct WaitingText;

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
            parent
                .spawn_bundle(TextBundle::from_section(
                    "Waiting for an opponent...",
                    TextStyle {
                        font: asset_server.load("Inconsolata.ttf"),
                        font_size: 24.0,
                        color: Color::hex("ebdbb2").unwrap(),
                    },
                ))
                .insert(WaitingText);
        });
}

fn update(mut text_query: Query<&mut Text, With<WaitingText>>, time: Res<Time>) {
    let mut text = text_query.get_single_mut().unwrap();
    let num_dots = (time.time_since_startup().as_secs() % 3) + 1;
    text.sections[0].value = format!(
        "Waiting for an opponent{}{}",
        ".".repeat(num_dots as usize),
        // Pad with spaces to avoid text changing width and dancing all around the screen 🕺
        " ".repeat(3 - num_dots as usize)
    );
}

fn cleanup(mut commands: Commands, ui_root: Query<Entity, With<UIRoot>>) {
    commands
        .entity(ui_root.get_single().unwrap())
        .despawn_recursive();
}

pub struct StateConnected;
impl Plugin for StateConnected {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Connected).with_system(setup));
        app.add_system_set(SystemSet::on_update(AppState::Connected).with_system(update));
        app.add_system_set(SystemSet::on_exit(AppState::Connected).with_system(cleanup));
    }
}
