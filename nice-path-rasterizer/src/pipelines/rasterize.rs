use crate::IntermediateBufs;
use std::{borrow::Cow, mem::size_of};
use wgpu::{
	util::{BufferInitDescriptor, DeviceExt},
	BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor,
	BlendOperation, BlendState, Buffer, BufferAddress, BufferBindingType, BufferSize, BufferUsages, ColorTargetState,
	ColorWrites, CommandBuffer, Device, FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState,
	RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureFormat,
	TextureSampleType, TextureView, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat,
	VertexState, VertexStepMode,
};

pub struct RasterizePipeline {
	pub tri: Buffer,
	pub pipeline: RenderPipeline,
	pub bind_group_layout: BindGroupLayout,
	pub uniform: Buffer,
}
impl RasterizePipeline {
	pub fn new(device: &Device, view_format: TextureFormat, view_size: [u32; 2]) -> Self {
		let tri = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Triangle"),
			contents: bytemuck::cast_slice(&[[0.0f32, 0.0], [2.0, 0.0], [0.0, 2.0]]),
			usage: BufferUsages::VERTEX,
		});

		let shader = device.create_shader_module(&ShaderModuleDescriptor {
			label: None,
			source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("rasterize.wgsl"))),
		});

		let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: None,
			entries: &[
				BindGroupLayoutEntry {
					binding: 0,
					visibility: ShaderStages::VERTEX,
					ty: BindingType::Buffer {
						ty: BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: BufferSize::new(8),
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 1,
					visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
					ty: BindingType::Texture {
						multisampled: false,
						sample_type: TextureSampleType::Float { filterable: false },
						view_dimension: TextureViewDimension::D2,
					},
					count: None,
				},
			],
		});
		let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		let vertex_buffers = [VertexBufferLayout {
			array_stride: size_of::<[f32; 2]>() as BufferAddress,
			step_mode: VertexStepMode::Vertex,
			attributes: &[VertexAttribute { format: VertexFormat::Float32x2, offset: 0, shader_location: 0 }],
		}];

		let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: VertexState { module: &shader, entry_point: "vs_main", buffers: &vertex_buffers },
			fragment: Some(FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[ColorTargetState {
					format: view_format,
					blend: Some(BlendState {
						color: BlendComponent {
							src_factor: BlendFactor::OneMinusDst,
							dst_factor: BlendFactor::One,
							operation: BlendOperation::Add,
						},
						alpha: BlendComponent::REPLACE,
					}),
					write_mask: ColorWrites::ALL,
				}],
			}),
			primitive: PrimitiveState { cull_mode: None, ..Default::default() },
			depth_stencil: None,
			multisample: MultisampleState::default(),
			multiview: None,
		});

		let view_size = [view_size[0] as f32, view_size[1] as f32];
		let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Uniform Buffer"),
			contents: bytemuck::cast_slice(&view_size),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		Self { tri, pipeline, bind_group_layout, uniform }
	}

	pub fn draw(&self, device: &Device, target: &IntermediateBufs, view: &TextureView) -> CommandBuffer {
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &self.bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::TextureView(&target.intermediate_tex()),
				},
			],
			label: None,
		});

		let mut rasterize_cmds = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		{
			let mut rpass = rasterize_cmds.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
						store: true,
					},
				}],
				depth_stencil_attachment: None,
			});
			rpass.push_debug_group("rasterize");
			rpass.set_pipeline(&self.pipeline);
			rpass.set_bind_group(0, &bind_group, &[]);
			rpass.set_vertex_buffer(0, self.tri.slice(..));
			rpass.draw(0..3, 0..1);
			rpass.pop_debug_group();
		}
		rasterize_cmds.finish()
	}
}
