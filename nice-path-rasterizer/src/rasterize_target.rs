use std::num::NonZeroU32;

use wgpu::{
	Device, Extent3d, TextureAspect, TextureDescriptor, TextureDimension, TextureUsages, TextureView,
	TextureViewDescriptor, TextureViewDimension,
};

use crate::JITTER_FORMAT;

pub struct IntermediateBufs {
	intermediate_tex: TextureView,
}
impl IntermediateBufs {
	pub fn new(device: &Device, width: u32, height: u32) -> Self {
		let texture = device.create_texture(&TextureDescriptor {
			label: None,
			size: Extent3d { width, height, depth_or_array_layers: 1 },
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D2,
			format: JITTER_FORMAT,
			usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
		});
		let intermediate_tex = texture.create_view(&TextureViewDescriptor {
			label: None,
			format: Some(JITTER_FORMAT),
			dimension: Some(TextureViewDimension::D2),
			aspect: TextureAspect::All,
			base_mip_level: 0,
			mip_level_count: NonZeroU32::new(1),
			base_array_layer: 0,
			array_layer_count: NonZeroU32::new(1),
		});

		IntermediateBufs { intermediate_tex }
	}

	pub fn intermediate_tex(&self) -> &TextureView {
		&self.intermediate_tex
	}
}
