use bevy::prelude::*;
use bevy::ecs::query::QuerySingleError;
use crate::assets::GameState;
use crate::fps_controller::RenderPlayer;

pub struct VoxelEditorPlugin;

impl Plugin for VoxelEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(VoxelEditor { enabled: true })
            .add_system(ghost_block.run_if(in_state(GameState::Ready)));
    }
}

#[derive(Clone, Resource)]
pub struct VoxelEditor {
    enabled: bool,
}

#[derive(Component)]
struct GhostBlock;

fn ghost_block(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    voxel_editor: Res<VoxelEditor>,
    cameras: Query<&Transform, (With<RenderPlayer>, Without<GhostBlock>)>,
    mut ghost_blocks: Query<(Entity, &mut Transform), (With<GhostBlock>, Without<RenderPlayer>)>,
) {
    match ghost_blocks.get_single_mut() {
        Ok((ghost_block, mut transform)) => {
            if !voxel_editor.enabled {
                commands.entity(ghost_block).despawn();
                return;
            }
            let Ok(camera) = cameras.get_single() else { return; };
            let in_front_of_camera =
                camera.mul_transform(
                    Transform::from_translation(Vec3::new(0.0, 0.0, -5.0)));
            *transform = Transform::from_translation(in_front_of_camera.translation.round());
        },
        Err(QuerySingleError::NoEntities(_)) => {
            if voxel_editor.enabled {
                let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
                commands.spawn((
                    PbrBundle {
                        mesh: cube,
                        material: materials.add(StandardMaterial {
                            base_color: Color::rgba(1.0, 1.0, 1.0, 0.2),
                            alpha_mode: AlphaMode::Blend,
                            ..default()
                        }),
                        ..default()
                    },
                    GhostBlock,
                ));
            }
        },
        Err(QuerySingleError::MultipleEntities(_)) => {
            panic!("Invariant violation: more than one ghost block");
        },
    }
}
