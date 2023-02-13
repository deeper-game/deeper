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
    prelude::{Assets, Component, Handle, Mesh, Query, ResMut, With},
    render::mesh::VertexAttributeValues,
};

use crate::outline::{
    smooth_normal::smooth_normal, OutlineMaterial, ATTRIBUTE_OUTLINE_NORMAL
};

#[derive(Component, Clone)]
pub struct OutlineNormals(pub VertexAttributeValues);

pub fn prepare_outline_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    outline_without_normals: Query<&Handle<Mesh>, With<Handle<OutlineMaterial>>>,
) {
    for mesh_handle in outline_without_normals.iter() {
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            // Don't have outline normal, just compute it.
            if !mesh.contains_attribute(ATTRIBUTE_OUTLINE_NORMAL) {
                mesh.insert_attribute(ATTRIBUTE_OUTLINE_NORMAL, smooth_normal(mesh));
            }
        }
    }
}
