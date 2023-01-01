use bevy::prelude::*;
use crate::assets::{GameState, ImageAssets};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_enter(GameState::Ready)
                            .with_system(show_crosshair))
            .add_system_set(SystemSet::on_enter(GameState::Ready)
                            .with_system(show_inventory));
    }
}

pub type InventoryPosition = (usize, usize);

#[derive(Component)]
pub struct InventorySlot {
    pub position: InventoryPosition,
}

pub fn show_crosshair(
    mut commands: Commands,
    images: Res<ImageAssets>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage(images.crosshair.clone()),
                style: Style {
                    size: Size::new(Val::Px(32.0), Val::Px(32.0)),
                    ..default()
                },
                ..default()
            });
        });
}

pub fn show_inventory(
    images: Res<ImageAssets>,
    mut commands: Commands,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Px(512.0),
                        height: Val::Px(128.0),
                    },
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                ..default()
            }).with_children(|parent| {
                for i in 0 .. 16 {
                    for j in 0 .. 4 {
                        parent.spawn(NodeBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            background_color: Color::rgb(0.4, 0.4, 0.4).into(),
                            ..default()
                        }).with_children(|parent| {
                            parent.spawn(ImageBundle {
                                style: Style {
                                    size: Size {
                                        width: Val::Px(28.0),
                                        height: Val::Px(28.0),
                                    },
                                    ..default()
                                },
                                image: UiImage(images.empty.clone()),
                                ..default()
                            })
                                .insert(InventorySlot { position: (i, j) });
                        });
                    }
                }
            });
        });
}
