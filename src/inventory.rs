use std::collections::HashMap;
use bevy::prelude::*;
use crate::assets::{ImageAssets, GameState};
use crate::fps_controller::LogicalPlayer;
use crate::level::voxel::VoxelShape;
use crate::ui::{ActiveHotbarSlot, HotbarSlot, InventorySlot, InventoryPosition};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(update_inventory.run_if(in_state(GameState::Ready)));
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
    pub hotbar: Vec<Option<InventoryItem>>,
    pub active: Option<InventoryItem>,
}

impl Inventory {
    pub fn new() -> Self {
        let mut hotbar = Vec::new();
        hotbar.resize(8, None);
        hotbar[0] = Some(InventoryItem {
            item_type: ItemType::Voxel(VoxelShape::Solid),
            equipped: false,
        });
        hotbar[1] = Some(InventoryItem {
            item_type: ItemType::Voxel(VoxelShape::Staircase),
            equipped: false,
        });
        hotbar[2] = Some(InventoryItem {
            item_type: ItemType::Voxel(VoxelShape::Roof),
            equipped: false,
        });
        Inventory {
            width: 16,
            height: 4,
            map: HashMap::new(),
            hotbar,
            active: None,
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
    Voxel(VoxelShape),
    Potion,
    Staff,
    Book,
}

impl ItemType {
    pub fn icon(&self, image_assets: &ImageAssets) -> Handle<Image> {
        match *self {
            ItemType::Voxel(VoxelShape::Solid) =>
                image_assets.solid.clone(),
            ItemType::Voxel(VoxelShape::Staircase) =>
                image_assets.staircase.clone(),
            ItemType::Voxel(_) => image_assets.coin.clone(),
            ItemType::Potion => image_assets.coin.clone(),
            ItemType::Staff => image_assets.coin.clone(),
            ItemType::Book => image_assets.coin.clone(),
        }
    }
}

pub fn update_inventory(
    images: Res<ImageAssets>,
    active_slot: Res<ActiveHotbarSlot>,
    mut inventory_slots: Query<(&InventorySlot, &mut UiImage), Without<HotbarSlot>>,
    mut hotbar_slots: Query<(&HotbarSlot, &mut UiImage), Without<InventorySlot>>,
    mut inventories: Query<&mut Inventory, With<LogicalPlayer>>,
) {
    let mut inventory = inventories.single_mut();
    inventory.active = inventory.hotbar[active_slot.index].clone();
    for (slot, mut image) in inventory_slots.iter_mut() {
        if inventory.map.contains_key(&slot.position) {
            *image = UiImage::new(inventory.map[&slot.position]
                                  .item_type.icon(&images));
        } else {
            *image = UiImage::new(images.empty.clone());
        }
    }
    for (slot, mut image) in hotbar_slots.iter_mut() {
        if let Some(item) = &inventory.hotbar[slot.position] {
            *image = UiImage::new(item.item_type.icon(&images));
        } else {
            *image = UiImage::new(images.empty.clone());
        }
    }
}
