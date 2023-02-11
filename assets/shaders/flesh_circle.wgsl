#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings

// Based on https://twitter.com/zozuar/status/1621229990267310081

struct FleshCircleMaterial {
    @align(16)
    resolution: f32,
    radius: f32,
    border: f32,
    flesh_time: f32,
    alpha: f32,
}

@group(1) @binding(0)
var<uniform> material: FleshCircleMaterial;

struct FragmentInput {
    #import bevy_pbr::mesh_vertex_output
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let resolution = material.resolution;
    let radius = material.radius;
    let border = material.border;
    let flesh_time = material.flesh_time;
    let alpha = material.alpha;

    var uv = in.uv;
    if (uv.x > 0.5) {
        uv.x -= 0.5;
    }
    uv.x *= 2.0;
    var pos = uv - vec2(0.5, 0.5);

    let pixelated_pos = floor(pos * resolution) / resolution;
    let flesh_time_scale = 4.0;
    let flesh_scale = 0.5;

    var flesh: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    {
        let t: f32 = flesh_time * flesh_time_scale;
        var p: vec2<f32> = pixelated_pos / flesh_scale;
        var n: vec2<f32> = vec2(0.0, 0.0);
        var d: f32 = dot(p, p);
        var S: f32 = 9.0;
        var i: f32 = 0.0;
        var a: f32 = 0.0;
        var j: f32 = 0.0;
        let theta = 5.0;
        let m = mat2x2<f32>(cos(theta), sin(theta), -sin(theta), cos(theta));
        loop {
            if (j >= 30.0) {
                break;
            }
            {
                p *= m;
                n *= m;
                let q: vec2<f32> =
                    p * S + t + sin(t - d * 6.0) * 0.8 + j + n;
                a += dot(cos(q) / S, vec2<f32>(0.2, 0.2));
                n -= sin(q);
                S *= 1.2;
            }
            continuing {
                j = j + 1.0;
            }
        }
        flesh = vec4<f32>(0.0, 0.0, 0.0, 1.0)
            + (a + 0.2) * vec4<f32>(4.0, 2.0, 1.0, 0.0)
            + (2.0 * a)
            + (-1.0 * d);
    }
    let transparent_flesh = vec4<f32>(flesh.x, flesh.y, flesh.z, alpha);

    var masked = transparent_flesh;
    let r_squared = dot(pos, pos);
    if (r_squared > (radius * radius)) {
        masked = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
    let r_plus_b = radius + border;
    if (r_squared > (r_plus_b * r_plus_b)) {
        masked = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    return masked;
}
