#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings
#import deeper::animation

struct AssociaCircleMaterial {
    @align(16)
    time: f32,
}

@group(1) @binding(0)
var<uniform> material: AssociaCircleMaterial;

struct FragmentInput {
    #import bevy_pbr::mesh_vertex_output
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    // var uv = in.uv;
    // if (uv.x > 0.5) {
    //     uv.x -= 0.5;
    // }
    // uv.x *= 2.0;
    // uv.x = 1.0 - uv.x;
    // var pos = uv - vec2(0.5, 0.5);
    // pos *= 0.75; // overall scale factor to make it fit
    //
    // var result = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    // let t = clamp(material.time, 0.001, 0.999);
    //
    // result = triple_circle(
    //     Circle(
    //         vec2<f32>(0.0, 0.0),
    //         0.25,
    //         0.03),
    //     t,
    //     pos,
    //     result);

    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
