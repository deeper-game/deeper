use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowFocused};
use bevy_egui::{egui, EguiContexts};
use crate::fps_controller::FpsController;
use crate::assets::{GameState, ImageAssets};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(
                UiState::PlayingGame(PlayingGameState {})
                // UiState::CreateOrJoin(CreateOrJoinState {
                //     room_id: String::new(),
                //     room_size: 2,
                // })
            )
            .add_system(manage_cursor)
            .add_system(show_create_or_join)
            .add_system(show_crosshair.in_schedule(OnEnter(GameState::Ready)))
            .add_system(show_inventory.in_schedule(OnEnter(GameState::Ready)));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CreateOrJoinState {
    room_id: String,
    room_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LobbyState {
    room_id: String,
    room_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlayingGameState {
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UsingEditorState {
    screen: Entity,
}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Resource)]
pub enum UiState {
    CreateOrJoin(CreateOrJoinState),
    Lobby(LobbyState),
    PlayingGame(PlayingGameState),
    UsingEditor(UsingEditorState),
}

impl UiState {
    pub fn from_create_or_join(&self) -> Option<CreateOrJoinState> {
        match self {
            UiState::CreateOrJoin(ref state) => Some(state.clone()),
            _ => None,
        }
    }

    pub fn from_lobby(&self) -> Option<LobbyState> {
        match self {
            UiState::Lobby(ref state) => Some(state.clone()),
            _ => None,
        }
    }

    pub fn from_playing_game(&self) -> Option<PlayingGameState> {
        match self {
            UiState::PlayingGame(ref state) => Some(state.clone()),
            _ => None,
        }
    }

    pub fn from_using_editor(&self) -> Option<UsingEditorState> {
        match self {
            UiState::UsingEditor(ref state) => Some(state.clone()),
            _ => None,
        }
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
                image: UiImage::new(images.crosshair.clone()),
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
                        width: Val::Px(513.0),
                        height: Val::Px(129.0),
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
                                image: UiImage::new(images.empty.clone()),
                                ..default()
                            })
                                .insert(InventorySlot { position: (i, j) });
                        });
                    }
                }
            });
        });
}

pub fn show_create_or_join(
    mut commands: Commands,
    mut egui_contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let mut next_ui_state = None;
    let width = windows.single().width();
    let height = windows.single().height();

    match *ui_state {
        UiState::CreateOrJoin(ref mut state) => {
            egui::Window::new("Deeper")
                .resizable(false)
                .collapsible(false)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(egui::Pos2 { x: width / 2.0, y: height / 2.0 })
                .show(egui_contexts.ctx_mut(), |ui| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                        ui.add(egui::Separator::default().spacing(12.0));
                        ui.horizontal(|ui| {
                            ui.label("Size:");
                            ui.add(egui::DragValue::new(&mut state.room_size)
                                   .speed(0.1));
                            ui.add(egui::Separator::default().spacing(12.0));
                            let create = egui::Button::new("Create Lobby");
                            if ui.add(create).clicked() {
                                let uuid = uuid::Uuid::new_v4();
                                state.room_id = format!("{}", uuid.simple());
                                state.room_id.truncate(12);
                                commands.insert_resource(
                                    crate::netcode::connect(&state.room_id,
                                                            state.room_size));
                                next_ui_state = Some(UiState::Lobby(LobbyState {
                                    room_id: state.room_id.clone(),
                                    room_size: state.room_size,
                                }));
                            }
                        });
                        ui.add(egui::Separator::default().spacing(12.0));
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut state.room_id);
                            let join = egui::Button::new("Join Lobby");
                            if ui.add(join).clicked() {
                                commands.insert_resource(
                                    crate::netcode::connect(&state.room_id,
                                                            state.room_size));
                                next_ui_state = Some(UiState::Lobby(LobbyState {
                                    room_id: state.room_id.clone(),
                                    room_size: state.room_size,
                                }));
                            }
                        });
                    });
                });
        },
        UiState::Lobby(ref mut state) => {
            let full = true; // TODO(taktoa): wait for players
            egui::Window::new("Lobby")
                .resizable(false)
                .collapsible(false)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(egui::Pos2 { x: width / 2.0, y: height / 2.0 })
                .show(egui_contexts.ctx_mut(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Lobby name: ");
                        let mut buf = state.room_id.clone();
                        ui.add(egui::TextEdit::singleline(&mut buf));
                    });
                    if full {
                        next_ui_state = Some(UiState::PlayingGame(PlayingGameState {
                        }));
                    }
                    // let start = egui::Button::new("Start Game");
                    // if ui.add_enabled(full, start).clicked() {
                    //     next_ui_state = Some(UiState::PlayingGame(PlayingGameState {
                    //     }));
                    // }
                });
        },
        _ => {},
    }

    if let Some(next) = next_ui_state {
        *ui_state = next;
    }
}

pub fn manage_cursor(
    ui_state: Res<UiState>,
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut controllers: Query<&mut FpsController>,
    mut focus_events: EventReader<WindowFocused>,
) {
    if ui_state.from_playing_game().is_some() {
        let (window_entity, mut window) = windows.single_mut();
        let mut grabbed = None;
        if btn.just_pressed(MouseButton::Left) {
            grabbed = Some(true);

        }
        if key.just_pressed(KeyCode::Escape) {
            grabbed = Some(false);
        }
        for focus_event in focus_events.iter() {
            if focus_event.window == window_entity {
                if !focus_event.focused {
                    grabbed = Some(false);
                }
            }
        }
        match grabbed {
            Some(true) => {
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
                for mut controller in &mut controllers {
                    controller.enable_input = true;
                }
            },
            Some(false) => {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
                for mut controller in &mut controllers {
                    controller.enable_input = false;
                }
            },
            None => {},
        }
    }
}
