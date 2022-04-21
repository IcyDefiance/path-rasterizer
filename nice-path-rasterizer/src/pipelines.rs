mod jitter;
mod rasterize;

pub use jitter::*;
pub use rasterize::*;

use wgpu::{Device, TextureFormat};

pub struct PathPipelines {
	pub jitter: JitterPipeline,
	pub rasterize: RasterizePipeline,
}
impl PathPipelines {
	pub fn new(device: &Device, view_format: TextureFormat, view_size: [u32; 2]) -> Self {
		Self {
			jitter: JitterPipeline::new(device, view_size),
			rasterize: RasterizePipeline::new(device, view_format, view_size),
		}
	}
}
