use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;
use crate::interact::Interactable;
use crate::outline::OutlineMaterial;
use crate::key_translator::TranslatedKey;

pub struct CrtPlugin;

impl Plugin for CrtPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ScreenActivated { entity: None })
            .add_plugin(MaterialPlugin::<CrtMaterial>::default())
            .add_startup_system(create_screen)
            .add_system(run_editor);
    }
}

#[derive(Resource)]
pub struct ScreenActivated {
    pub entity: Option<Entity>,
}

fn create_screen(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut crt_materials: ResMut<Assets<CrtMaterial>>,
    mut outlines: ResMut<Assets<OutlineMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    use bevy::render::render_resource::*;

    let mut image = Image::new_fill(
        Extent3d { width: 1360, height: 768, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[255u8, 0u8, 255u8, 255u8],
        TextureFormat::Rgba8UnormSrgb);

    let potato: Handle<Image> = asset_server.load("crt-potato-thin.png");

    let crt_material_handle = crt_materials.add(CrtMaterial {
        color_texture: Some(images.add(image)),
        overlay_texture: Some(potato),
    });

    commands.spawn((
        crate::editor::Screen::new(crate::editor::Editor::new()),
        MaterialMeshBundle {
            mesh: meshes.add(
                Mesh::from(shape::Plane { size: 0.25, ..default() })),
            material: crt_material_handle,
            transform: Transform::from_xyz(1.0, 0.75, 1.0),
            //.looking_at(Vec3::new(1.5, 1.5, 1.5), Vec3::new(0.0, 1.0, 0.0)),
            ..default()
        },
        Interactable,
        Collider::cuboid(0.125, 0.01, 0.125),
        outlines.add(OutlineMaterial {
            width: 0.,
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        }),
    ));
}

fn run_editor(
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<CrtMaterial>>,
    mut screens: Query<(&mut crate::editor::Screen, &Handle<CrtMaterial>)>,
    screen_activated: Res<ScreenActivated>,
    mut keyboard_events: EventReader<TranslatedKey>,
) {
    if let Some(entity) = screen_activated.entity {
        let (mut screen, material_handle) = screens.get_mut(entity).unwrap();

        let mut needs_rerender = false;

        for key in keyboard_events.iter() {
            if key.pressed {
                screen.editor.process_keypress(key.key);
                needs_rerender = true;
            }
        }

        if !needs_rerender {
            return;
        }

        screen.editor.refresh_screen();

        let rasterized = screen.editor.rasterize().unwrap();

        let image_handle =
            materials.get_mut(material_handle).unwrap()
            .color_texture.clone().unwrap();
        let image: &mut Image = images.get_mut(&image_handle).unwrap();

        {
            let mut index = 0;
            for y in 0 .. rasterized.height {
                for x in 0 .. rasterized.width {
                    let [a, r, g, b] =
                        rasterized.get((rasterized.width - 1) - x, y).to_le_bytes();
                    image.data[index + 0] = r;
                    image.data[index + 1] = g;
                    image.data[index + 2] = b;
                    image.data[index + 3] = a;
                    index += 4;
                }
            }
        }

    }
}

use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "ac413474-8412-4933-b03b-a9876282ca33"]
pub struct CrtMaterial {
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    overlay_texture: Option<Handle<Image>>,
}

impl Material for CrtMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/crt_material.wgsl".into()
    }
}
