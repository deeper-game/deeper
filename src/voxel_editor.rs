use bevy::prelude::*;
use bevy::ecs::query::QuerySingleError;
use crate::assets::{GameState, ColliderMode, VoxelMeshAssets};
use crate::inventory::{Inventory, ItemType};
use crate::fps_controller::{LogicalPlayer, RenderPlayer};
use crate::level::ActiveLevel;
use crate::level::voxel::{CardinalDir, Voxel, VoxelShape, Texture, Style};
use crate::ui::ActiveHotbarSlot;

pub struct VoxelEditorPlugin;

impl Plugin for VoxelEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(VoxelEditor { enabled: true })
            .add_system(ghost_block.run_if(in_state(GameState::Ready))
                        .after(crate::inventory::update_inventory));
    }
}

#[derive(Clone, Resource)]
pub struct VoxelEditor {
    enabled: bool,
}

#[derive(Component, Default)]
struct GhostBlock {
    rotation: CardinalDir,
    block: VoxelShape,
    texture: Texture,
    style: Style
}

fn ghost_block(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut level: ResMut<ActiveLevel>,
    vma: Res<VoxelMeshAssets>,
    voxel_editor: Res<VoxelEditor>,
    cameras: Query<&Transform, (With<RenderPlayer>, Without<GhostBlock>)>,
    mut ghost_blocks: Query<(Entity, &mut GhostBlock, &mut Transform), Without<RenderPlayer>>,
    active_hotbar_slot: Res<ActiveHotbarSlot>,
    mut inventories: Query<&mut Inventory, With<LogicalPlayer>>,
    key: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
) {
    let inventory = inventories.single();
    match ghost_blocks.get_single_mut() {
        Ok((entity, mut ghost_block, mut transform)) => {
            let hotbar_block = inventory.hotbar[active_hotbar_slot.index]
                .as_ref().map(|x| &x.item_type).cloned();
            let mut despawn = false;
            despawn |= !voxel_editor.enabled;
            despawn |= Some(ItemType::Voxel(ghost_block.block.clone())) != hotbar_block;
            if despawn {
                commands.entity(entity).despawn_recursive();
                return;
            }
            if key.just_pressed(KeyCode::R) {
                ghost_block.rotation = ghost_block.rotation.rotate_ccw_90();
            }
            let Ok(camera) = cameras.get_single() else { return; };
            let in_front_of_camera =
                camera.mul_transform(
                    Transform::from_translation(Vec3::new(0.0, 0.0, -4.0)));
            *transform = Transform::IDENTITY
                .mul_transform(Transform::from_translation(in_front_of_camera.translation.round()))
                .mul_transform(Transform::from_rotation(ghost_block.rotation.as_rotation()))
                .mul_transform(Transform::from_translation(Vec3::new(-0.5, -0.5, 0.5)));
            if mouse.just_pressed(MouseButton::Left) {
                let mut vm = match ghost_block.block {
                    VoxelShape::Solid => vma.solid[0].clone(),
                    VoxelShape::Staircase => vma.staircase[0].clone(),
                    _ => { return; },
                };
                let spawned = vm.spawn(&transform, &mut commands,
                                       &mut meshes, &mut materials);
                let position = (transform.translation - Vec3::new(-0.5, -0.5, 0.5)).as_ivec3();
                level.updates.insert(position, Voxel {
                    orientation: ghost_block.rotation,
                    shape: ghost_block.block.clone(),
                    texture: ghost_block.texture,
                    style: ghost_block.style,
                });
                level.entities.push(spawned);
            }
        },
        Err(QuerySingleError::NoEntities(_)) => {
            if voxel_editor.enabled {
                let active =
                    inventory.hotbar[active_hotbar_slot.index]
                    .as_ref().map(|x| &x.item_type);
                if let Some(ItemType::Voxel(shape)) = active {
                    let mut vm = match *shape {
                        VoxelShape::Solid => vma.solid[0].clone(),
                        VoxelShape::Staircase => vma.staircase[0].clone(),
                        _ => { return; },
                    };
                    vm.collider_mode = ColliderMode::None;
                    vm.ghost = true;
                    let entity = vm.spawn(
                        &Transform::default(), &mut commands,
                        &mut meshes, &mut materials);
                    let mut ghost_block = GhostBlock::default();
                    ghost_block.block = shape.clone();
                    commands.entity(entity).insert(ghost_block);
                }
            }
        },
        Err(QuerySingleError::MultipleEntities(_)) => {
            panic!("Invariant violation: more than one ghost block");
        },
    }
}
