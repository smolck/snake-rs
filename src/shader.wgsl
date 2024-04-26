const colors = array<vec3<f32>, 3>(
    // snake
    vec3<f32>(1., 1., 1.),
    // bg
    vec3<f32>(0., 0., 0.),
    // food
    vec3<f32>(1., 0., 0.)
);

const zero = 0;
const one = 1;
const two = 2;

// @group(0) @binding(0)
// var<uniform> colors: array<vec3f, 3>;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) coloridx: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // why location(0) here?
    @location(0) color: vec3<f32>,
}

/*fn from_linear_rgb(c: [f32; 3]) -> Color {
    let f = |x: f32| -> u32 {
        let y = if x > 0.0031308 {
            let a = 0.055;
            (1.0 + a) * x.powf(-2.4) - a
        } else {
            12.92 * x
        };
        (y * 255.0).round() as u32
    };
    f(c[0]) << 16 | f(c[1]) << 8 | f(c[2])
}*/

/*fn to_srgb(color: vec3<f32>) -> vec3<f32> {
    let r = pow(((color.r + 0.055) / 1.055), 2.4);
    let g = pow(((color.g + 0.055) / 1.055), 2.4);
    let b = pow(((color.b + 0.055) / 1.055), 2.4);
    return vec3<f32>(r, g, b);
}*/

@vertex
fn vs_main(
    model: VertexInput,
    // @builtin(vertex_index) in_vert_idx: u32,
) -> VertexOutput {
    var out: VertexOutput;

    // This is so stupid lmao
    if model.coloridx == u32(0) {
        out.color = colors[zero];
    } else if model.coloridx == u32(1) {
        out.color = colors[one];
    } else if model.coloridx == u32(2) {
        out.color = colors[two];
    }

    out.clip_position = vec4<f32>(model.pos, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
