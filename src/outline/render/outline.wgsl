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

#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@group(1)
@binding(0)
var<uniform> mesh: Mesh;

struct OutlineMat {
    @align(16)
    width: f32,
    color: vec4<f32>,
};

@group(2)
@binding(0)
var<uniform> outline_mat: OutlineMat;

struct DoubleReciprocalWindowSize {
    @align(16)
    size: vec2<f32>,
};

@group(3)
@binding(0)
var<uniform> window_size: DoubleReciprocalWindowSize;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let mvp = view.view_proj * mesh.model;
    let clip_position = mvp * vec4<f32>(vertex.position, 1.0);
    let clip_normal = mvp * vec4<f32>(vertex.normal, 0.0);
    let extrude_offset = normalize(clip_normal.xy) * outline_mat.width * clip_position.w * window_size.size;
    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip_position.xy + extrude_offset, clip_position.zw);
    return out;
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return outline_mat.color;
}
