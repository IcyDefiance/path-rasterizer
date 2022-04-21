struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
};

struct Locals {
    view_size: vec2<f32>;
};
[[group(0), binding(0)]]
var<uniform> r_locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
	var pos: vec2<f32> = (position / r_locals.view_size * 2.0 - 1.0) * vec2<f32>(1.0, -1.0);
    out.position = vec4<f32>(pos, 0.0, 1.0);
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

[[stage(fragment)]]
fn fs_wire() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}
