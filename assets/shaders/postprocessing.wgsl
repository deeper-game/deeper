struct PostprocessingMaterial {
    @align(16)
    placeholder: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: PostprocessingMaterial;

@group(1) @binding(1)
var input_texture: texture_2d<f32>;
@group(1) @binding(2)
var input_sampler: sampler;

@group(1) @binding(3)
var palette_texture: texture_2d<f32>;
@group(1) @binding(4)
var palette_sampler: sampler;

fn rgba2hsba(color: vec4<f32>) -> vec4<f32> {
    let k = vec4<f32>(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    let p = mix(vec4<f32>(color.bg, k.wz),
                vec4<f32>(color.gb, k.xy),
                step(color.b, color.g));
    let q = mix(vec4<f32>(p.xyw, color.r),
                vec4<f32>(color.r, p.yzx),
                step(p.x, color.r));
    let d = q.x - min(q.w, q.y);
    let e = 1.0e-10;
    return vec4<f32>(
        abs(q.z + (q.w - q.y) / (6.0 * d + e)),
        d / (q.x + e),
        q.x,
        color.a);
}

fn hsba2rgba(color: vec4<f32>) -> vec4<f32> {
    let rgb0 = (color.x * 6.0 + vec3<f32>(0.0, 4.0, 2.0)) / 6.0;
    let rgb1 = abs(((rgb0 - trunc(rgb0)) * 6.0) - 3.0) - 1.0;
    let rgb2 = vec3<f32>(
        clamp(rgb1.r, 0.0, 1.0),
        clamp(rgb1.g, 0.0, 1.0),
        clamp(rgb1.b, 0.0, 1.0));
    let rgb3 = rgb2 * rgb2 * (3.0 - 2.0 * rgb2);
    let rgb4 = color.z * mix(vec3<f32>(1.0, 1.0, 1.0), rgb3, color.y);
    return vec4<f32>(rgb4.rgb, color.a);
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let uv = vec2<f32>(uv.x, 1.0 - uv.y);
    // let input_dims: vec2<i32> = textureDimensions(input_texture);
    // let palette_dims: vec2<i32> = textureDimensions(palette_texture);
    // let reps: vec2<f32> = vec2<f32>(2 * input_dims / palette_dims);
    // let uv_overlay: vec2<f32> = fract(uv * reps);
    // let color = textureSample(input_texture, input_sampler, uv) *
    //     textureSample(palette_texture, palette_sampler, uv_overlay);
    let color = textureSample(input_texture, input_sampler, uv);
    // let exponent = 1.2;
    // let rounded_color = vec4<f32>(
    //     pow(round(pow(color.r, exponent) * num_colors) / num_colors, 1.0 / exponent),
    //     pow(round(pow(color.g, exponent) * num_colors) / num_colors, 1.0 / exponent),
    //     pow(round(pow(color.b, exponent) * num_colors) / num_colors, 1.0 / exponent),
    //     color.a);
    let hsba = rgba2hsba(color);
    let rounded_hsba = vec4<f32>(
        floor(hsba.r * 32.0) / 32.0,
        floor(hsba.g * 32.0) / 32.0,
        floor(hsba.b * 64.0) / 64.0,
        hsba.a);
    //let hsba = vec4<f32>(
    //    clamp(hsba.r, 0.0, 1.0),
    //    clamp(hsba.g, 0.0, 1.0),
    //    clamp(hsba.b, 0.0, 1.0),
    //    hsba.a);
    return color; // hsba2rgba(rounded_hsba);
}
