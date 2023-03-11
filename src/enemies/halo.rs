use bevy::prelude::*;
use bevy_scene_hook::{SceneHook, HookedSceneBundle};
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{
    AsBindGroup, ShaderRef, ShaderType
};
use crate::assets::GameState;
use crate::fps_controller::RenderPlayer;
use std::collections::HashMap;

pub struct HaloPlugin;

impl Plugin for HaloPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(MaterialPlugin::<HaloMaterial>::default())
            .insert_resource(HaloTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
            .add_system(create_halos.run_if(in_state(GameState::Ready)))
            .add_system(halo_behavior.run_if(in_state(GameState::Ready)))
            .add_system(halo_cycle.run_if(in_state(GameState::Ready)));
    }
}

#[derive(Resource)]
struct HaloTimer(Timer);

/// Add this component to an entity to spawn a halo. The SceneBundle will be
/// attached to the same entity that the SpawnHalo was attached to, but
#[derive(Clone, Debug, Component, Default)]
pub struct SpawnHalo {
    pub transform: Transform,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum HaloAnimation {
    HaloUnriffled,
    HaloRiffled1,
    HaloRiffled2,
    HaloRiffled3,
    HaloRiffled4,
}

#[derive(Clone, Debug, Component)]
pub struct Halo {
    pub halo_animation: HaloAnimation,
}

fn create_halos(
    mut commands: Commands,
    mut halo_materials: ResMut<Assets<HaloMaterial>>,
    asset_server: Res<AssetServer>,
    halos: Query<(Entity, &SpawnHalo)>,
) {
    let halo_scene = asset_server.load("gltf/halo.glb#Scene0");
    let halo_material = halo_materials.add(HaloMaterial {});
    for (entity, spawn_halo) in halos.iter() {
        let spawn_halo = spawn_halo.clone();
        let halo_material = halo_material.clone();
        commands.entity(entity)
            .remove::<SpawnHalo>()
            .insert((
                Halo {
                    halo_animation: HaloAnimation::HaloUnriffled,
                },
                HookedSceneBundle {
                    scene: SceneBundle {
                        scene: halo_scene.clone(),
                        transform: spawn_halo.transform,
                        ..Default::default()
                    },
                    hook: SceneHook::new(move |entity, cmds| {
                        let (Some(name), Some(children)) = (
                            entity.get::<Name>().map(|t| t.as_str()),
                            entity.get::<Children>(),
                        ) else {
                            return;
                        };

                        let halo_names = [
                            "HaloUnriffled",
                            "HaloRiffled1",
                            "HaloRiffled2",
                            "HaloRiffled3",
                            "HaloRiffled4",
                        ];
                        for n in halo_names {
                            if name == n {
                                cmds.commands()
                                    .entity(*children.first().unwrap())
                                    .remove::<Handle<StandardMaterial>>()
                                    .insert(halo_material.clone());
                            }
                        }
                    }),
                },
            ));
    }
}

fn halo_behavior(
    mut commands: Commands,
    mut scenes: ResMut<Assets<Scene>>,
    names: Query<(Entity, &Name)>,
    players: Query<&GlobalTransform, With<RenderPlayer>>,
    halos: Query<(&Handle<Scene>, &GlobalTransform, &Halo)>,
) {
    for (halo_scene_handle, halo_transform, halo) in halos.iter() {
        // TODO: create a system for calculating the nearest player, that runs
        // at a slower cadence due to the n^2 nature of the calculation
        let nearest: Vec3 = {
            let mut nearest_player = None;
            let mut distance = f32::MAX;
            for player in players.iter() {
                let h = halo_transform.translation();
                let p = player.translation();
                let d = (h - p).length();
                if d < distance {
                    distance = d;
                    nearest_player = Some(p.clone());
                }
            }
            let Some(n) = nearest_player else { return; };
            n
        };

        let Some(scene) = scenes.get_mut(halo_scene_handle) else {
            continue;
        };

        let mut name_to_entity = HashMap::<String, Entity>::new();
        for (entity, name) in names.iter() {
            name_to_entity.insert(name.as_str().to_string(), entity.clone());
        }

        let mut halo_names = [
            "HaloUnriffled",
            "HaloRiffled1",
            "HaloRiffled2",
            "HaloRiffled3",
            "HaloRiffled4",
        ];

        for name in halo_names {
            if let Some(entity) = name_to_entity.get(name) {
                commands.entity(entity.clone()).insert(
                    if name == format!("{:?}", halo.halo_animation) {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    });
            }
        }
    }
}

fn halo_cycle(
    time: Res<Time>,
    mut timer: ResMut<HaloTimer>,
    mut halos: Query<&mut Halo>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::H) { // timer.0.finished() {
        for mut halo in halos.iter_mut() {
            if halo.halo_animation == HaloAnimation::HaloUnriffled {
                timer.0 = Timer::from_seconds(0.1, TimerMode::Repeating);
            }
            halo.halo_animation = match halo.halo_animation {
                HaloAnimation::HaloUnriffled => HaloAnimation::HaloRiffled1,
                HaloAnimation::HaloRiffled1 => HaloAnimation::HaloRiffled2,
                HaloAnimation::HaloRiffled2 => HaloAnimation::HaloRiffled3,
                HaloAnimation::HaloRiffled3 => HaloAnimation::HaloRiffled4,
                HaloAnimation::HaloRiffled4 => HaloAnimation::HaloUnriffled,
            };
            if halo.halo_animation == HaloAnimation::HaloUnriffled {
                timer.0 = Timer::from_seconds(2.0, TimerMode::Repeating);
            }
        }
        timer.0.reset();
    }
    timer.0.tick(time.delta());
}


#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "8a5f3fd8-76ce-4e5e-9209-0c94e1e00331"]
pub struct HaloMaterial {
}

use bevy::render::{mesh::*, render_resource::*};
use bevy::pbr::*;

impl Material for HaloMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/halo.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
