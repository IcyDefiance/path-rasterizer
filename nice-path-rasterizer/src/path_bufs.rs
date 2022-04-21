use nice_path_tessellator::{fill_tessellate, Path};
use wgpu::{
	util::{BufferInitDescriptor, DeviceExt},
	Buffer, BufferUsages, Device,
};

pub struct PathBufs {
	pub verts: Buffer,
	pub fill_idxs: Buffer,
	pub fill_idx_len: usize,
	pub quadratic_idxs: Buffer,
	pub quadratic_idx_len: usize,
}
impl PathBufs {
	pub fn new(device: &Device, path: &Path) -> Self {
		// TODO: send data directly to the gpu
		let cpu_bufs = fill_tessellate(&path);

		let verts = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Vertex Buffer"),
			contents: bytemuck::cast_slice(&cpu_bufs.verts),
			usage: BufferUsages::VERTEX,
		});

		let fill_idxs = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Index Buffer"),
			contents: bytemuck::cast_slice(&cpu_bufs.fill_idxs),
			usage: BufferUsages::INDEX,
		});

		let quadratic_idxs = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Index Buffer"),
			contents: bytemuck::cast_slice(&cpu_bufs.quadratic_idxs),
			usage: BufferUsages::INDEX,
		});

		Self {
			verts,
			fill_idxs,
			fill_idx_len: cpu_bufs.fill_idxs.len(),
			quadratic_idxs,
			quadratic_idx_len: cpu_bufs.quadratic_idxs.len(),
		}
	}
}
