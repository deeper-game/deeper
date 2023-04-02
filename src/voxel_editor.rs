use bevy::prelude::*;
use bevy::ecs::query::QuerySingleError;
use crate::assets::{GameState, ColliderMode, VoxelMeshAssets};
use crate::inventory::{Inventory, ItemType};
use crate::fps_controller::{LogicalPlayer, RenderPlayer};
use crate::level::voxel::{CardinalDir, VoxelShape};
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

#[derive(Component)]
struct GhostBlock {
    rotation: CardinalDir,
    block: VoxelShape,
}

fn ghost_block(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    vma: Res<VoxelMeshAssets>,
    voxel_editor: Res<VoxelEditor>,
    cameras: Query<&Transform, (With<RenderPlayer>, Without<GhostBlock>)>,
    mut ghost_blocks: Query<(Entity, &mut GhostBlock, &mut Transform), Without<RenderPlayer>>,
    active_hotbar_slot: Res<ActiveHotbarSlot>,
    mut inventories: Query<&mut Inventory, With<LogicalPlayer>>,
    key: Res<Input<KeyCode>>,
) {
    let inventory = inventories.single();
    match ghost_blocks.get_single_mut() {
        Ok((entity, mut ghost_block, mut transform)) => {
            let hotbar_block = inventory.hotbar[active_hotbar_slot.index]
                .as_ref().map(|x| &x.item_type).cloned();
            if Some(ItemType::Voxel(ghost_block.block.clone())) != hotbar_block {
                commands.entity(entity).despawn_recursive();
                return;
            }
            if !voxel_editor.enabled {
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
                    commands.entity(entity).insert(GhostBlock {
                        rotation: CardinalDir::default(),
                        block: shape.clone(),
                    });
                }
            }
        },
        Err(QuerySingleError::MultipleEntities(_)) => {
            panic!("Invariant violation: more than one ghost block");
        },
    }
}
