struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
	[[location(0)]] uv: vec2<f32>;
	[[location(1)]] mode: u32;
	[[location(2)]] instance_index: u32;
};

struct Locals {
    view_size: vec2<f32>;
};
[[group(0), binding(0)]]
var<uniform> r_locals: Locals;

[[stage(vertex)]]
fn vs_main(
	[[builtin(instance_index)]] instance_index: u32,
	[[builtin(vertex_index)]] vertex_index: u32,
    [[location(0)]] instance: vec3<f32>,
    [[location(1)]] position: vec2<f32>,
) -> VertexOutput {
	var jitter: vec2<f32> = instance.xy;
	var mode: u32 = u32(instance.z);

	var pos: vec2<f32> = ((position + jitter) / r_locals.view_size * 2.0 - 1.0) * vec2<f32>(1.0, -1.0);

	var u: f32 = (f32(vertex_index) + 1.0) % 3.0 / 2.0;
	var uv: vec2<f32> = vec2<f32>(u, floor(u));

    var out: VertexOutput;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = uv;
	out.mode = mode;
	out.instance_index = instance_index;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	// quadratic curve
	if (in.mode == 1u && in.uv.x * in.uv.x - in.uv.y > 0.0) {
		discard;
	}

	if (in.instance_index % 6u == 0u) {
		return vec4<f32>(1.0 / 255.0, 0.0, 0.0, 1.0);
	} else if (in.instance_index % 6u == 1u) {
		return vec4<f32>(16.0 / 255.0, 0.0, 0.0, 1.0);
	} else if (in.instance_index % 6u == 2u) {
		return vec4<f32>(0.0, 1.0 / 255.0, 0.0, 1.0);
	} else if (in.instance_index % 6u == 3u) {
		return vec4<f32>(0.0, 16.0 / 255.0, 0.0, 1.0);
	} else if (in.instance_index % 6u == 4u) {
		return vec4<f32>(0.0, 0.0, 1.0 / 255.0, 1.0);
	} else if (in.instance_index % 6u == 5u) {
		return vec4<f32>(0.0, 0.0, 16.0 / 255.0, 1.0);
	}
	return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
