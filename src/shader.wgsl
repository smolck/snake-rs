struct ColorUniforms {
    snake_color: vec3<f32>,
    bg_color: vec3<f32>,
    food_color: vec3<f32>,
}

struct WindowRes {
    width: f32,
    height: f32,
}

@group(0) @binding(0)
var<uniform> color_uniforms: ColorUniforms;

// Split from the main one because of alignment stuff
@group(0) @binding(1)
var<uniform> window_size: vec2<f32>;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) coloridx: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // why location(0) here?
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    if model.coloridx == u32(0) {
        out.color = color_uniforms.snake_color;
    } else if model.coloridx == u32(1) {
        out.color = color_uniforms.bg_color;
    } else if model.coloridx == u32(2) {
        out.color = color_uniforms.food_color;
    }

    let pos_float = model.pos / window_size;
    // Convert to clip coordinates
    let x = (pos_float.x - 0.5) * 2.;
    let y = (pos_float.y - 0.5) * 2.;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
