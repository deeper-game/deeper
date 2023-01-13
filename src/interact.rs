use bevy::prelude::*;
use bevy_rapier3d::prelude::{RapierContext, Collider, QueryFilter};
use bevy_fps_controller::controller::{FpsController, LogicalPlayer, RenderPlayer};
use crate::outline::OutlineMaterial;
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
    let mut inventory = inventories.single_mut();
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
    mut outlines: ResMut<Assets<OutlineMaterial>>,
    mut interactables: Query<&Handle<OutlineMaterial>,
                             (With<GlobalTransform>, With<Collider>, With<Interactable>)>,
    player: Query<&GlobalTransform, With<RenderPlayer>>,
) {
    let camera: &GlobalTransform = player.single();
    if let Some((entity, toi)) = rapier_context.cast_ray(
        camera.translation(), camera.forward(), 2.0, false,
        QueryFilter::exclude_dynamic(),
    ) {
        if selected.entity != Some(entity) {
            if let Some(e) = selected.entity {
                if let Ok(old_material) = interactables.get_mut(e) {
                    outlines.get_mut(old_material).unwrap().width = 0.0;
                }
            }
            selected.entity = None;

            if let Ok(new_material) = interactables.get(entity) {
                outlines.get_mut(new_material).unwrap().width = 3.0;
                selected.entity = Some(entity);
            }
        }
    } else {
        if let Some(e) = selected.entity {
            if let Ok(old_material) = interactables.get(e) {
                outlines.get_mut(old_material).unwrap().width = 0.0;
            }
        }
        selected.entity = None;
    }
}
