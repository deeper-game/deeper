#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings
#import deeper::animation

struct GlyphRect {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

let NUM_BUBBLES: i32 = 6;

struct BubblesCircleMaterial {
    @align(16)
    time: f32,
    @align(16)
    bubble_glyphs: array<GlyphRect, NUM_BUBBLES>,
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

fn paste_glyph(
    index: i32,
    size: f32,
    center: vec2<f32>,
    pos: vec2<f32>,
    color: vec4<f32>,
) -> vec4<f32> {
    let g = material.bubble_glyphs[index % NUM_BUBBLES];

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

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    if (uv.x > 0.5) {
        uv.x -= 0.5;
    }
    uv.x *= 2.0;
    uv.x = 1.0 - uv.x;
    var pos = uv - vec2(0.5, 0.5);
    pos *= 0.75; // overall scale factor to make it fit

    var result = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let t = clamp(material.time, 0.001, 0.999);

    let inner_circle_radius = 0.1;
    let inner_circle_weight = 0.01;
    let outer_circle_radius = 0.2;
    let outer_circle_weight = 0.005;
    let bubble_circle_radius = 0.3;
    let bubble_inner_radius = 0.05;
    let bubble_inner_weight = 0.002;
    let bubble_outer_radius = 0.06;
    let bubble_outer_weight = 0.002;
    let bubble_bloom = 1.0;
    let bubble_color = vec4<f32>(bubble_bloom * 0.8,
                                 bubble_bloom * 0.8,
                                 bubble_bloom * 0.9,
                                 1.0);
    let glyph_size = 0.375;
    let glyph_bloom = 1.5;
    let glyph_color = vec4<f32>(glyph_bloom * 0.16863,
                                glyph_bloom * 0.62745,
                                glyph_bloom * 1.0,
                                1.0);
    let ngram_spacing = 0.01;
    let ngram_weight = 0.0015;
    let ngram_bloom = 1.0;
    let ngram_color = vec4<f32>(ngram_bloom * 0.5,
                                ngram_bloom * 0.5,
                                ngram_bloom * 0.6,
                                1.0);

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

    result = single_circle(
        Circle(
            vec2<f32>(0.0, 0.0),
            bubble_outer_radius + bubble_circle_radius,
            t * bubble_outer_weight),
        t * 2.0,
        pos,
        result);

    result.r *= 0.6;
    result.g *= 0.6;
    result.b *= 0.7;

    for(var i: i32 = 0; i < NUM_BUBBLES; i = i + 1) {
        let theta = f32(i) * tau / f32(NUM_BUBBLES);
        let center = bubble_circle_radius * vec2<f32>(cos(theta), sin(theta));

        var bubble_pixel = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        {
            let transition = (0.5 * f32(i) / f32(NUM_BUBBLES)) + 0.45;
            let speed = 10.0;
            bubble_pixel = single_circle(
                Circle(
                    center,
                    bubble_inner_radius,
                    t * bubble_inner_weight),
                sigmoid(t, transition, speed),
                pos,
                bubble_pixel);
            bubble_pixel = single_circle(
                Circle(
                    center,
                    bubble_outer_radius,
                    t * bubble_outer_weight),
                sigmoid(t, transition - 0.02, speed),
                pos,
                bubble_pixel);
            bubble_pixel *= bubble_color;
        }

        var glyph_pixel = paste_glyph(i, glyph_size, center, pos,
                                      vec4<f32>(0.0, 0.0, 0.0, 0.0));
        glyph_pixel = round(glyph_pixel);
        glyph_pixel = glyph_pixel * glyph_color;
        glyph_pixel = clamp(t * 2.0, 0.0, 1.0) * glyph_pixel;

        result = max(result, bubble_pixel);
        result = max(result, glyph_pixel);
    }

    {
        var ngram_pixel = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        for(var i: i32 = 0; i < NUM_BUBBLES; i = i + 1) {
            for(var j: i32 = 0; j < NUM_BUBBLES; j = j + 1) {
                if (abs(j - i) == (NUM_BUBBLES / 2)) {
                    continue;
                }
                let theta_i = f32(i) * tau / f32(NUM_BUBBLES);
                let theta_j = f32(j) * tau / f32(NUM_BUBBLES);
                let center_i = bubble_circle_radius * vec2<f32>(cos(theta_i), sin(theta_i));
                let center_j = bubble_circle_radius * vec2<f32>(cos(theta_j), sin(theta_j));
                ngram_pixel =
                    middle_out_line_segment(center_i, center_j,
                                            t * ngram_weight,
                                            t, pos, ngram_pixel);
            }
        }
        for(var i: i32 = 0; i < NUM_BUBBLES; i = i + 1) {
            let theta = f32(i) * tau / f32(NUM_BUBBLES);
            let center = bubble_circle_radius * vec2<f32>(cos(theta), sin(theta));
            if (length(pos - center) < bubble_inner_radius) {
                ngram_pixel = vec4<f32>(0.0, 0.0, 0.0, 0.0);
            }
        }
        if (abs(length(pos) - outer_circle_radius) < ngram_spacing) {
            ngram_pixel = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
        if (length(pos) < inner_circle_radius + ngram_spacing) {
            ngram_pixel = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
        result = max(result, ngram_pixel * ngram_color);
    }

    return result;
}
