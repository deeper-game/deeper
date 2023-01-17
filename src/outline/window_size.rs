use bevy::{
    core::cast_slice,
    ecs::system::{Resource, lifetimeless::SRes, SystemParamItem},
    math::Vec2,
    prelude::{Commands, Entity, EventReader, Res, ResMut},
    render::{
        Extract,
        render_phase::{EntityRenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::{BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
    window::WindowResized,
};

use crate::outline::OutlinePipeline;

#[derive(Resource)]
pub(crate) struct ExtractedWindowSize {
    width: f32,
    height: f32,
}

#[derive(ShaderType, Resource)]
pub(crate) struct DoubleReciprocalWindowSizeUniform {
    size: Vec2,
    padding: Vec2,
}

#[derive(Resource)]
pub(crate) struct DoubleReciprocalWindowSizeMeta {
    pub buffer: Buffer,
    pub bind_group: Option<BindGroup>,
}

pub(crate) fn extract_window_size(
    mut commands: Commands,
    mut resized_events: Extract<EventReader<WindowResized>>,
) {
    if let Some(size_change) = resized_events.iter().last() {
        if size_change.id.is_primary() {
            let width = size_change.width;
            let height = size_change.height;
            commands.insert_resource(ExtractedWindowSize { width, height });
        }
    }
}

pub(crate) fn prepare_window_size(
    window_size: Res<ExtractedWindowSize>,
    window_size_meta: ResMut<DoubleReciprocalWindowSizeMeta>,
    render_queue: Res<RenderQueue>,
) {
    if window_size.is_changed() {
        let window_size_uniform = DoubleReciprocalWindowSizeUniform {
            size: Vec2::new(2.0 / window_size.width, 2.0 / window_size.height),
            padding: Vec2::new(0.0, 0.0),
        };
        render_queue.write_buffer(
            &window_size_meta.buffer,
            0,
            cast_slice(&[window_size_uniform.size]),
        )
    }
}

pub(crate) fn queue_window_size_bind_group(
    render_device: Res<RenderDevice>,
    mut double_reciprocal_window_size_meta: ResMut<DoubleReciprocalWindowSizeMeta>,
    pipeline: Res<OutlinePipeline>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("window size bind group"),
        layout: &pipeline.window_size_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: double_reciprocal_window_size_meta
                .buffer
                .as_entire_binding(),
        }],
    });
    double_reciprocal_window_size_meta.bind_group = Some(bind_group);
}

pub(crate) struct SetWindowSizeBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetWindowSizeBindGroup<I> {
    type Param = SRes<DoubleReciprocalWindowSizeMeta>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        window_size: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let window_size_bind_group = window_size.into_inner().bind_group.as_ref().unwrap();
        pass.set_bind_group(I, window_size_bind_group, &[]);

        RenderCommandResult::Success
    }
}
