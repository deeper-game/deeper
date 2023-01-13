@group(1) @binding(1)
var color_texture: texture_2d<f32>;
@group(1) @binding(2)
var color_sampler: sampler;

@group(1) @binding(3)
var overlay_texture: texture_2d<f32>;
@group(1) @binding(4)
var overlay_sampler: sampler;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let color_dims: vec2<i32> = textureDimensions(color_texture);
    let overlay_dims: vec2<i32> = textureDimensions(overlay_texture);
    let reps: vec2<f32> = vec2<f32>(2 * color_dims / overlay_dims);
    let uv_overlay: vec2<f32> = fract(uv * reps);
    return textureSample(color_texture, color_sampler, uv) *
        textureSample(overlay_texture, overlay_sampler, uv_overlay);
}
