struct VertexOutput {
	[[location(0)]] tex_coord: vec2<f32>;
	[[builtin(position)]] position: vec4<f32>;
};

struct ViewInfo {
	view_size: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> view_info: ViewInfo;
[[group(0), binding(1)]]
var intermediate: texture_2d<f32>;

[[stage(vertex)]]
fn vs_main(
	[[location(0)]] position: vec2<f32>,
) -> VertexOutput {
	var pos: vec2<f32> = (position * vec2<f32>(textureDimensions(intermediate)) / view_info.view_size * 2.0 - 1.0) * vec2<f32>(1.0, -1.0);

	var out: VertexOutput;
	out.tex_coord = position;
	out.position = vec4<f32>(pos, 0.0, 1.0);
	return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	var intermediate_size: vec2<f32> = vec2<f32>(textureDimensions(intermediate));

	// Get samples for -2/3 and -1/3
	var tex_coord1: vec2<f32> = in.tex_coord * intermediate_size - vec2<f32>(1.0, 0.0);
	var value_l: vec2<f32> = textureLoad(intermediate, vec2<i32>(tex_coord1), 0).xy * 255.0;
	var lower_l: vec2<f32> = value_l % 16.0;
	var upper_l: vec2<f32> = (value_l - lower_l) / 16.0;
	var alpha_l: vec2<f32> = (upper_l % 2.0 + lower_l % 2.0) / 2.0;

	// Get samples for 0, +1/3, and +2/3
	var tex_coord_r: vec2<f32> = in.tex_coord * intermediate_size;
	var value_r: vec3<f32> = textureLoad(intermediate, vec2<i32>(tex_coord_r), 0).xyz * 255.0;
	var lower_r: vec3<f32> = value_r % 16.0;
	var upper_r: vec3<f32> = (value_r - lower_r) / 16.0;
	var alpha_r: vec3<f32> = (upper_r % 2.0 + lower_r % 2.0) / 2.0;

	// Average the energy over the pixels on either side
	return vec4<f32>(1.0, 1.0, 1.0, (alpha_l.x + alpha_r.z + alpha_r.y) / 3.0);

	// return rgba;
}
