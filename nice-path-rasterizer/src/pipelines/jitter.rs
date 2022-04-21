use std::{borrow::Cow, mem::size_of};
use wgpu::{
	util::{BufferInitDescriptor, DeviceExt},
	BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor,
	BlendOperation, BlendState, Buffer, BufferBindingType, BufferSize, BufferUsages, ColorTargetState, ColorWrites,
	CommandBuffer, Device, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState,
	RenderPassColorAttachment, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource,
	ShaderStages, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use crate::{IntermediateBufs, PathBufs, JITTER_FORMAT};

/// offset x, offset y, mode (0: fill, 1: quadratic)
const JITTER: &[[f32; 3]] = &[
	// fill
	[-1.0f32 / 12.0, -5.0 / 12.0, 0.0],
	[1.0 / 12.0, 1.0 / 12.0, 0.0],
	[3.0 / 12.0, -1.0 / 12.0, 0.0],
	[5.0 / 12.0, 5.0 / 12.0, 0.0],
	[7.0 / 12.0, -3.0 / 12.0, 0.0],
	[9.0 / 12.0, 3.0 / 12.0, 0.0],
	// quadratic
	[-1.0f32 / 12.0, -5.0 / 12.0, 1.0],
	[1.0 / 12.0, 1.0 / 12.0, 1.0],
	[3.0 / 12.0, -1.0 / 12.0, 1.0],
	[5.0 / 12.0, 5.0 / 12.0, 1.0],
	[7.0 / 12.0, -3.0 / 12.0, 1.0],
	[9.0 / 12.0, 3.0 / 12.0, 1.0],
];

pub struct JitterPipeline {
	pub bind_group_layout: BindGroupLayout,
	pub pipeline: RenderPipeline,
	pub instances: Buffer,
	pub uniform: Buffer,
}
impl JitterPipeline {
	pub fn new(device: &Device, view_size: [u32; 2]) -> Self {
		let shader = device.create_shader_module(&ShaderModuleDescriptor {
			label: None,
			source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("jitter.wgsl"))),
		});

		let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: None,
			entries: &[BindGroupLayoutEntry {
				binding: 0,
				visibility: ShaderStages::VERTEX,
				ty: BindingType::Buffer {
					ty: BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: BufferSize::new(8),
				},
				count: None,
			}],
		});
		let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		let vertex_buffers = [
			VertexBufferLayout {
				array_stride: size_of::<[f32; 3]>() as _,
				step_mode: VertexStepMode::Instance,
				attributes: &[VertexAttribute { format: VertexFormat::Float32x3, offset: 0, shader_location: 0 }],
			},
			VertexBufferLayout {
				array_stride: size_of::<[f32; 2]>() as _,
				step_mode: VertexStepMode::Vertex,
				attributes: &[VertexAttribute { format: VertexFormat::Float32x2, offset: 0, shader_location: 1 }],
			},
		];

		let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: VertexState { module: &shader, entry_point: "vs_main", buffers: &vertex_buffers },
			fragment: Some(FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[ColorTargetState {
					format: JITTER_FORMAT,
					blend: Some(BlendState {
						color: BlendComponent {
							src_factor: BlendFactor::One,
							dst_factor: BlendFactor::One,
							operation: BlendOperation::Add,
						},
						alpha: BlendComponent {
							src_factor: BlendFactor::One,
							dst_factor: BlendFactor::One,
							operation: BlendOperation::Add,
						},
					}),
					write_mask: ColorWrites::ALL,
				}],
			}),
			primitive: PrimitiveState { cull_mode: None, ..Default::default() },
			depth_stencil: None,
			multisample: MultisampleState::default(),
			multiview: None,
		});

		let instances = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Instance Buffer"),
			contents: bytemuck::cast_slice(JITTER),
			usage: BufferUsages::VERTEX,
		});

		let view_size = [view_size[0] as f32, view_size[1] as f32];
		let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Uniform Buffer"),
			contents: bytemuck::cast_slice(&view_size),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		Self { bind_group_layout, pipeline, instances, uniform }
	}

	pub fn draw(&self, device: &Device, target: &IntermediateBufs, path_bufs: &PathBufs) -> CommandBuffer {
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &self.bind_group_layout,
			entries: &[wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() }],
			label: None,
		});

		let mut jitter_cmds = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		{
			let mut rpass = jitter_cmds.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[RenderPassColorAttachment {
					view: target.intermediate_tex(),
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
						store: true,
					},
				}],
				depth_stencil_attachment: None,
			});
			rpass.push_debug_group("jitter");
			rpass.set_pipeline(&self.pipeline);
			rpass.set_bind_group(0, &bind_group, &[]);
			rpass.set_index_buffer(path_bufs.fill_idxs.slice(..), wgpu::IndexFormat::Uint16);
			rpass.set_vertex_buffer(0, self.instances.slice(..));
			rpass.set_vertex_buffer(1, path_bufs.verts.slice(..));
			rpass.draw_indexed(0..path_bufs.fill_idx_len as u32, 0, 0..6);
			rpass.set_index_buffer(path_bufs.quadratic_idxs.slice(..), wgpu::IndexFormat::Uint16);
			rpass.draw_indexed(0..path_bufs.quadratic_idx_len as u32, 0, 6..12);
			rpass.pop_debug_group();
		}
		jitter_cmds.finish()
	}
}
