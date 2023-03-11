// Copyright 2023 the bevy_outline Contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use bevy::{
    core::cast_slice,
    ecs::change_detection::DetectChanges,
    ecs::query::With,
    ecs::system::{Query, Resource, lifetimeless::SRes, SystemParamItem},
    math::Vec2,
    prelude::{Commands, Entity, EventReader, Res, ResMut},
    render::{
        Extract,
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass
        },
        render_resource::{BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
    window::{PrimaryWindow, Window, WindowResized},
};

use crate::outline::OutlinePipeline;

#[derive(Resource)]
pub(crate) struct ExtractedWindowSize {
    width: f32,
    height: f32,
}

#[derive(ShaderType, Resource)]
pub(crate) struct DoubleReciprocalWindowSizeUniform {
    #[align(16)]
    size: Vec2,
}

#[derive(Resource)]
pub(crate) struct DoubleReciprocalWindowSizeMeta {
    pub buffer: Buffer,
    pub bind_group: Option<BindGroup>,
}

pub(crate) fn extract_window_size(
    mut commands: Commands,
    mut resized_events: Extract<EventReader<WindowResized>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Some(size_change) = resized_events.iter().last() {
        if let Ok(_) = windows.get(size_change.window.clone()) {
            let width = size_change.width;
            let height = size_change.height;
            commands.insert_resource(ExtractedWindowSize { width, height });
        }
    }
}

pub(crate) fn prepare_window_size(
    optional_window_size: Option<Res<ExtractedWindowSize>>,
    window_size_meta: ResMut<DoubleReciprocalWindowSizeMeta>,
    render_queue: Res<RenderQueue>,
) {
    if let Some(window_size) = optional_window_size {
        if window_size.is_changed() {
            let window_size_uniform = DoubleReciprocalWindowSizeUniform {
                size: Vec2::new(2.0 / window_size.width, 2.0 / window_size.height),
            };
            render_queue.write_buffer(
                &window_size_meta.buffer,
                0,
                cast_slice(&[window_size_uniform.size]),
            )
        }
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
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetWindowSizeBindGroup<I> {
    type Param = SRes<DoubleReciprocalWindowSizeMeta>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();

    fn render<'w>(
        _item: &P,
        _view: (),
        _entity: (),
        window_size: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let window_size_bind_group = window_size.into_inner().bind_group.as_ref().unwrap();
        pass.set_bind_group(I, window_size_bind_group, &[]);

        RenderCommandResult::Success
    }
}
