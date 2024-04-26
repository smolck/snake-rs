#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub coloridx: u32, // index into color uniforms
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            // What we're gonna avoid THROUGH THE POWER OF MACROS
            /*attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ],*/
            attributes: &ATTRS,
        }
    }
}

#[inline]
fn normalize(win_width: f32, win_height: f32, n: f32, w: bool) -> f32 {
    -1.0 + 2.0 * (if w { n / win_width } else { n / win_height })
}

pub fn square_for_pos(win_width: f32, win_height: f32, x: f32, y: f32, size: f32) -> [f32; 12] {
    let l = x - size; // (size / 2.);
    let r = x + size; // (size / 2.);
    let t = y + size;
    let b = y - size;

    let mut verts = [l, b, r, b, l, t, r, b, r, t, l, t];

    for i in 0..verts.len() {
        verts[i] = normalize(win_width, win_height, verts[i], i % 2 == 0);
    }

    verts
}
