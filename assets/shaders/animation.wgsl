const tau: f32 = 6.28318530718;
const tau_1_3: f32 = 2.09439510239;
const tau_2_3: f32 = 4.18879020479;

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

fn sigmoid(x: f32, transition: f32, speed: f32) -> f32 {
    return 1.0 + tanh(speed * (x - transition));
}
