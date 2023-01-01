use std::collections::HashMap;
use bevy::prelude::*;
use crate::LogicalPlayer;
use crate::assets::{ImageAssets, GameState};
use crate::ui::{InventorySlot, InventoryPosition};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_update(GameState::Ready)
                            .with_system(update_inventory));
    }
}

#[derive(Clone, Debug)]
pub struct InventoryItem {
    pub item_type: ItemType,
    pub equipped: bool,
}

#[derive(Component, Debug)]
pub struct Inventory {
    pub width: usize,
    pub height: usize,
    pub map: HashMap<InventoryPosition, InventoryItem>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            width: 16,
            height: 4,
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, item: &InventoryItem) {
        for x in 0 .. self.width {
            for y in 0 .. self.height {
                if !self.map.contains_key(&(x, y)) {
                    self.map.insert((x, y), item.clone());
                    return;
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ItemType {
    Potion,
    Staff,
    Book,
}

impl ItemType {
    pub fn icon(&self, image_assets: &ImageAssets) -> Handle<Image> {
        match *self {
            ItemType::Potion => image_assets.coin.clone(),
            ItemType::Staff => image_assets.coin.clone(),
            ItemType::Book => image_assets.coin.clone(),
        }
    }
}

pub fn update_inventory(
    images: Res<ImageAssets>,
    mut inventory_slots: Query<(&InventorySlot, &mut UiImage)>,
    mut inventories: Query<&mut Inventory, With<LogicalPlayer>>,
) {
    let mut inventory = inventories.single_mut();
    for (slot, mut image) in inventory_slots.iter_mut() {
        if inventory.map.contains_key(&slot.position) {
            *image = UiImage(inventory.map[&slot.position]
                             .item_type.icon(&images));
        } else {
            *image = UiImage(images.empty.clone());
        }
    }
}
