#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings

struct GlyphRect {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

struct BubblesCircleMaterial {
    @align(16)
    time: f32,
    @align(16)
    bubble_glyph_0: GlyphRect,
    @align(16)
    bubble_glyph_1: GlyphRect,
    @align(16)
    bubble_glyph_2: GlyphRect,
    @align(16)
    bubble_glyph_3: GlyphRect,
    @align(16)
    bubble_glyph_4: GlyphRect,
    @align(16)
    bubble_glyph_5: GlyphRect,
}

@group(1) @binding(0)
var<uniform> material: BubblesCircleMaterial;

@group(1) @binding(1)
var font_texture: texture_2d<f32>;
@group(1) @binding(2)
var font_sampler: sampler;

struct FragmentInput {
    #import bevy_pbr::mesh_vertex_output
}

let tau: f32 = 6.28318530718;
let tau_1_3: f32 = 2.09439510239;
let tau_2_3: f32 = 4.18879020479;

struct Circle {
    center: vec2<f32>,
    radius: f32,
    line_width: f32,
}

fn circular_arc(
    circle: Circle,
    start_angle: f32,
    end_angle: f32,
    pos: vec2<f32>,
    color: vec4<f32>,
) -> vec4<f32> {
    let start_angle_mod = fract(start_angle / tau) * tau;
    let end_angle_mod = fract(end_angle / tau) * tau;
    let shifted = pos - circle.center;
    let distance = sqrt(dot(shifted, shifted));
    var result = color;
    let angle = atan2(shifted.y, shifted.x) + tau / 2.0;
    if ((distance <= (circle.radius + (circle.line_width / 2.0)))
        && (distance >= (circle.radius - (circle.line_width / 2.0)))) {
        if (start_angle_mod < end_angle_mod) {
            if ((start_angle_mod <= angle) && (angle <= end_angle_mod)) {
                result = vec4<f32>(1.0, 1.0, 1.0, 1.0);
            }
        } else if (end_angle_mod < start_angle_mod) {
            if ((angle <= end_angle_mod) || (angle >= start_angle_mod)) {
                result = vec4<f32>(1.0, 1.0, 1.0, 1.0);
            }
        } else {
            result = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }
    }
    return result;
}

fn single_circle(
    circle: Circle,
    time: f32, // from 0.0 to 1.0
    pos: vec2<f32>,
    color: vec4<f32>,
) -> vec4<f32> {
    let angle = clamp(time, 0.0, 1.0) * tau;
    var result = color;
    result = circular_arc(circle, 0.0, angle, pos, result);
    return result;
}

fn triple_circle(
    circle: Circle,
    time: f32, // from 0.0 to 1.0
    pos: vec2<f32>,
    color: vec4<f32>,
) -> vec4<f32> {
    let angle = clamp(time, 0.0, 1.0) * tau;
    var result = color;
    result = circular_arc(circle, 0.0, angle, pos, result);
    result = circular_arc(circle, tau_1_3, angle + tau_1_3, pos, result);
    result = circular_arc(circle, tau_2_3, angle + tau_2_3, pos, result);
    return result;
}

fn paste_glyph(
    index: i32,
    size: f32,
    center: vec2<f32>,
    pos: vec2<f32>,
    color: vec4<f32>,
) -> vec4<f32> {
    var g = material.bubble_glyph_0;
    if (index == 0) {
        g = material.bubble_glyph_0;
    } else if (index == 1) {
        g = material.bubble_glyph_1;
    } else if (index == 2) {
        g = material.bubble_glyph_2;
    } else if (index == 3) {
        g = material.bubble_glyph_3;
    } else if (index == 4) {
        g = material.bubble_glyph_4;
    } else if (index == 5) {
        g = material.bubble_glyph_5;
    }

    let width = g.max_x - g.min_x;
    let height = g.max_y - g.min_y;

    let scaled_width = width * size;
    let scaled_height = height * size;

    let upper_left = center - (vec2<f32>(scaled_width, scaled_height) / 2.0);
    var shifted = pos - upper_left;
    shifted.x = scaled_width - shifted.x;

    let x = (shifted.x / size) + g.min_x;
    let y = (shifted.y / size) + g.min_y;
    var result = textureSample(font_texture, font_sampler, vec2<f32>(x, y));

    if ((pos - center).x < -scaled_width / 2.0) {
        result = color;
    }
    if ((pos - center).x > scaled_width / 2.0) {
        result = color;
    }
    if ((pos - center).y < -scaled_height / 2.0) {
        result = color;
    }
    if ((pos - center).y > scaled_height / 2.0) {
        result = color;
    }

    return result;
}

fn line_segment(
    start: vec2<f32>,
    end: vec2<f32>,
    line_width: f32,
    pos: vec2<f32>,
    color: vec4<f32>,
) -> vec4<f32> {
    let es: vec2<f32> = end - start;
    let ps: vec2<f32> = pos - start;
    let h = clamp(dot(ps, es) / dot(es, es), 0.0, 1.0);
    let d = length(ps - es * h);
    var result = color;
    if (d < line_width) {
        result = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
    return result;
}

fn start_to_end_line_segment(
    start: vec2<f32>,
    end: vec2<f32>,
    line_width: f32,
    time: f32, // from 0.0 to 1.0
    pos: vec2<f32>,
    color: vec4<f32>,
) -> vec4<f32> {
    return line_segment(
        start, start + ((end - start) * time), line_width, pos, color);
}

fn middle_out_line_segment(
    start: vec2<f32>,
    end: vec2<f32>,
    line_width: f32,
    time: f32, // from 0.0 to 1.0
    pos: vec2<f32>,
    color: vec4<f32>,
) -> vec4<f32> {
    var result = color;
    let middle = (start + end) / 2.0;
    result =
        start_to_end_line_segment(middle, start, line_width, time, pos, color);
    result =
        start_to_end_line_segment(middle, end, line_width, time, pos, color);
    return result;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let time = material.time;

    var uv = in.uv;
    if (uv.x > 0.5) {
        uv.x -= 0.5;
    }
    uv.x *= 2.0;
    uv.x = 1.0 - uv.x;
    var pos = uv - vec2(0.5, 0.5);

    var result = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let t = clamp(time / 2.5, 0.0, 1.0);

    let inner_circle_radius = 0.1;
    let inner_circle_weight = 0.01;
    let outer_circle_radius = 0.2;
    let outer_circle_weight = 0.005;
    let bubble_circle_radius = 0.3;
    let bubble_inner_radius = 0.05;
    let bubble_inner_weight = 0.002;
    let bubble_outer_radius = 0.06;
    let bubble_outer_weight = 0.002;
    let glyph_size = 0.375;
    let hexagram_spacing = 0.01;
    let hexagram_weight = 0.001;


    result = triple_circle(
        Circle(
            vec2<f32>(0.0, 0.0),
            inner_circle_radius,
            inner_circle_weight),
        t * 3.0,
        pos,
        result);

    result = triple_circle(
        Circle(
            vec2<f32>(0.0, 0.0),
            outer_circle_radius,
            outer_circle_weight),
        t,
        pos,
        result);

    for(var i: i32 = 0; i < 6; i = i + 1) {
        let theta = f32(i) * tau / 6.0;
        let center = bubble_circle_radius * vec2<f32>(cos(theta), sin(theta));

        if (time > f32(i) / 6.0) {
            result = single_circle(
                Circle(
                    center,
                    bubble_inner_radius,
                    t * bubble_inner_weight),
                time - f32(i) / 6.0,
                pos,
                result);
            result = single_circle(
                Circle(
                    center,
                    bubble_outer_radius,
                    t * bubble_outer_weight),
                time - f32(i) / 6.0,
                pos,
                result);
        }
        var glyph_pixel = paste_glyph(i, glyph_size, center, pos,
                                      vec4<f32>(0.0, 0.0, 0.0, 0.0));
        glyph_pixel = round(glyph_pixel);
        glyph_pixel = t * 0.75 * glyph_pixel;
        result += glyph_pixel;
    }

    {
        var hexagram_pixel = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        for(var i: i32 = 0; i < 6; i = i + 1) {
            for(var j: i32 = 0; j < 6; j = j + 1) {
                if ((j - i) == 3) {
                    continue;
                }
                if ((i - j) == 3) {
                    continue;
                }
                let theta_i = f32(i) * tau / 6.0;
                let theta_j = f32(j) * tau / 6.0;
                let center_i = bubble_circle_radius * vec2<f32>(cos(theta_i), sin(theta_i));
                let center_j = bubble_circle_radius * vec2<f32>(cos(theta_j), sin(theta_j));
                hexagram_pixel =
                    middle_out_line_segment(center_i, center_j,
                                            t * hexagram_weight,
                                            t, pos, hexagram_pixel);
            }
        }
        for(var i: i32 = 0; i < 6; i = i + 1) {
            let theta = f32(i) * tau / 6.0;
            let center = bubble_circle_radius * vec2<f32>(cos(theta), sin(theta));
            if (length(pos - center) < bubble_inner_radius) {
                hexagram_pixel = vec4<f32>(0.0, 0.0, 0.0, 0.0);
            }
        }
        if (abs(length(pos) - outer_circle_radius) < hexagram_spacing) {
            hexagram_pixel = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
        if (length(pos) < inner_circle_radius + hexagram_spacing) {
            hexagram_pixel = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
        result = max(result, hexagram_pixel);
    }


    //if (abs(pos.x) > 0.495) {
    //    result = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    //}
    //if (abs(pos.y) > 0.495) {
    //    result = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    //}

    return result;
}
