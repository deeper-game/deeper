#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings
#import deeper::animation

struct FragmentInput {
    #import bevy_pbr::mesh_vertex_output
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    var pos = uv - vec2(0.5, 0.5);
    // pos.x += 0.002 * sin(globals.time * 70.0);
    // pos.y += 0.002 * cos(globals.time * 70.0);

    var result = 0.4 * vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let color = vec4<f32>(0.8, 0.02, 0.05, 1.0);

    let step_time = 0.7;
    let num_steps = 5;
    let step = i32(floor(globals.time / step_time)) % num_steps;

    if (step == 0) {
        let ring_width = 0.02 + 0.002 * sin(globals.time);
        if ((i32(floor(length(pos) / ring_width)) % 2) == 0) {
            result = color;
        }
    }
    if (step == 1) {
        let checker_width = 0.08 + 0.008 * sin(globals.time);
        if (((i32(floor(pos.x / checker_width)) + i32(floor(pos.y / checker_width))) % 2) == 0) {
            result = color;
        }
    }
    if (step == 2) {
        let hyperbola_width = 0.02 + 0.002 * sin(globals.time);
        if ((i32(floor((pos.x * pos.y) / (hyperbola_width * 0.25))) % 2) == 0) {
            result = color;
        }
    }
    if (step == 3) {
        let line_width = 0.02 + 0.002 * sin(globals.time);
        let sine = 0.05 * sin(globals.time) * sin(pos.x * 20.0);
        if ((i32(floor((pos.y + sine) / line_width)) % 2) == 0) {
            result = color;
        }
    }
    if (step == 4) {
        let line_width = 0.02 + 0.002 * sin(globals.time);
        let sine = 0.05 * sin(globals.time) * sin(pos.y * 20.0);
        if ((i32(floor((pos.x + sine) / line_width)) % 2) == 0) {
            result = color;
        }
    }

    result *= 1.0;

    return result;
}
