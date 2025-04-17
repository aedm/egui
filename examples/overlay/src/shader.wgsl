struct Uniforms {
    time: f32,
    center_x: f32,
    center_y: f32,
    color: f32,
    zoom: f32,
}
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

// meant to be called with 3 vertex indices: 0, 1, 2
// draws one large triangle over the clip space like this:
// (the asterisks represent the clip space bounds)
//-1,1           1,1
// ---------------------------------
// |              *              .
// |              *           .
// |              *        .
// |              *      .
// |              *    .
// |              * .
// |***************
// |            . 1,-1
// |          .
// |       .
// |     .
// |   .
// |.
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var result: VertexOutput;
    let x = f32(vertex_index / 2) * 4.0 - 1.0;
    let y = f32(vertex_index & 1) * 4.0 - 1.0;

    result.position = vec4<f32>(x, y, 0.0, 1.0);

    var t = uniforms.time / 3.0;
    var sn = sin(t);
    var cs = cos(t);
    var xr = x * cs - y * sn;
    var yr = x * sn + y * cs;

    var zoom = pow(2.0, -uniforms.zoom * 20.0);
    result.tex_coords = vec2(xr, yr) * zoom;
    return result;
}


fn mandelbrot(x: f32, y: f32) -> f32 {
    var z = vec2<f32>(x, y);
    var c = vec2<f32>(x, y);
    var i = 0;
    while (i < 1000) {
        z = vec2<f32>(z.x * z.x - z.y * z.y + c.x, 2.0 * z.x * z.y + c.y);
        if (length(z) > 2.0) {
            return f32(i) / 100.0;
        }
        i++;
    }
    return 1.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = vec2<f32>(
        in.tex_coords.x - uniforms.center_x,
        in.tex_coords.y - uniforms.center_y
    );
    let m = mandelbrot(uv.x, uv.y);
    return vec4<f32>(m, m, m, 1.0);
}
