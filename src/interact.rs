use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;
use bevy_rapier3d::prelude::{RapierContext, Collider, QueryFilter};
use crate::fps_controller::{FpsController, LogicalPlayer, RenderPlayer};
use crate::inventory::{Inventory, InventoryItem, ItemType};

pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Selected { entity: None })
            .add_system(interaction_glow)
            .add_system(interact);
    }
}

#[derive(Component)]
pub struct Item {
    pub item_type: ItemType,
}

#[derive(Component)]
pub struct Interactable;

#[derive(Resource)]
pub struct Selected {
    entity: Option<Entity>,
}

pub fn interact(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    selected: Res<Selected>,
    items: Query<&Item>,
    mut inventories: Query<&mut Inventory, With<LogicalPlayer>>,
    screens: Query<&crate::editor::Screen>,
    mut screen_activated: ResMut<crate::crt::ScreenActivated>,
    mut controllers: Query<&mut FpsController>,
) {
    let Ok(mut inventory) = inventories.get_single_mut() else { return; };
    if mouse.just_pressed(MouseButton::Right) {
        if let Some(entity) = selected.entity {
            if let Ok(item) = items.get(entity) {
                let item_type = item.item_type.clone();
                commands.entity(entity).despawn();
                inventory.insert(&InventoryItem { item_type, equipped: false });
            }
            if screens.get(entity).is_ok() {
                screen_activated.entity = Some(entity);
                for mut controller in &mut controllers {
                    controller.enable_input = false;
                }
            }
        }
    }
}

pub fn interaction_glow(
    rapier_context: Res<RapierContext>,
    mut selected: ResMut<Selected>,
    mut interactables: Query<&mut OutlineVolume,
                             (With<GlobalTransform>, With<Collider>, With<Interactable>)>,
    camera: Query<&GlobalTransform, With<RenderPlayer>>,
    player: Query<Entity, With<LogicalPlayer>>,
) {
    let Ok(camera_transform) = camera.get_single() else { return; };
    let Ok(player_entity) = player.get_single() else { return; };
    if let Some((entity, toi)) = rapier_context.cast_ray(
        camera_transform.translation(), camera_transform.forward(), 2.0, false,
        QueryFilter::default().exclude_rigid_body(player_entity),
    ) {
        if selected.entity != Some(entity) {
            if let Some(e) = selected.entity {
                if let Ok(mut volume) = interactables.get_mut(e) {
                    volume.visible = false;
                }
            }
            selected.entity = None;

            if let Ok(mut volume) = interactables.get_mut(entity) {
                volume.visible = true;
                selected.entity = Some(entity);
            }
        }
    } else {
        if let Some(e) = selected.entity {
            if let Ok(mut volume) = interactables.get_mut(e) {
                volume.visible = false;
            }
        }
        selected.entity = None;
    }
}
