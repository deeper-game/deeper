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

use std::hash::Hash;

use bevy::{
    math::Vec3A,
    prelude::{Deref, DerefMut},
    render::mesh::{Mesh, VertexAttributeValues},
    utils::FloatOrd,
    utils::HashMap,
};

/// An ordered float3 array which implements Eq and Hash
#[derive(Debug, Clone, Copy, Deref, DerefMut, Default)]
#[repr(transparent)]
struct Float3Ord([f32; 3]);

impl PartialEq for Float3Ord {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self[0]).eq(&FloatOrd(other[0]))
            && FloatOrd(self[1]).eq(&FloatOrd(other[1]))
            && FloatOrd(self[2]).eq(&FloatOrd(other[2]))
    }
}
impl Eq for Float3Ord {}

impl Hash for Float3Ord {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        FloatOrd(self[0]).hash(state);
        FloatOrd(self[1]).hash(state);
        FloatOrd(self[2]).hash(state);
    }
}

/// smooth the normals of vertex at same position
pub(crate) fn smooth_normal(mesh: &Mesh) -> VertexAttributeValues {
    let mut normals_map = HashMap::new();
    let v_positions = get_float3x3(mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap());
    let v_normals = get_float3x3(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap());
    let mut smoothed_normals = vec![[0.; 3]; v_positions.len()];
    v_positions
        .iter()
        .zip(v_normals.iter())
        .enumerate()
        .for_each(|(index, (pos, normal))| {
            let key = Float3Ord(*pos);
            let entry = normals_map.entry(key).or_insert((vec![], Vec3A::ZERO));
            entry.0.push(index);
            entry.1 += Vec3A::from(*normal);
        });
    normals_map.drain().for_each(|(_, (indices, normal))| {
        let ave_normal = normal.normalize().into();
        indices.into_iter().for_each(|index| {
            smoothed_normals[index] = ave_normal;
        })
    });
    VertexAttributeValues::Float32x3(smoothed_normals)
}

#[inline(always)]
fn get_float3x3(values: &VertexAttributeValues) -> &Vec<[f32; 3]> {
    match values {
        VertexAttributeValues::Float32x3(v) => v,
        _ => panic!("Vertex Position must be a Float32x3"),
    }
}
