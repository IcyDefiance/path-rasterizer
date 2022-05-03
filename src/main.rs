mod framework;

use bytemuck::{Pod, Zeroable};
use nice_path_rasterizer::{IntermediateBufs, Path, PathBufs, PathPipelines};
use std::{future::Future, pin::Pin, task};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
	pos: [f32; 2],
}

// This can be done simpler with `FutureExt`, but we don't want to add
// a dependency just for this small case.
struct ErrorFuture<F> {
	inner: F,
}
impl<F: Future<Output = Option<wgpu::Error>>> Future for ErrorFuture<F> {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<()> {
		let inner = unsafe { self.map_unchecked_mut(|me| &mut me.inner) };
		inner.poll(cx).map(|error| {
			if let Some(e) = error {
				panic!("Rendering {}", e);
			}
		})
	}
}

struct Example {
	path_bufs: PathBufs,
	path_pipelines: PathPipelines,
	intermediate_bufs: IntermediateBufs,
}
impl framework::Example for Example {
	fn init(
		config: &wgpu::SurfaceConfiguration,
		_adapter: &wgpu::Adapter,
		device: &wgpu::Device,
		_queue: &wgpu::Queue,
	) -> Self {
		let path_pipelines = PathPipelines::new(device, config.format, [config.width, config.height]);

		let path = Path::start()
			.move_to([100.0, 100.0])
			.quadratic_curve_to([150.0, 50.0], [200.0, 100.0])
			.line_to([200.0, 200.0])
			.line_to([155.0, 200.0])
			.line_to([175.0, 125.0])
			.line_to([125.0, 125.0])
			.line_to([145.0, 200.0])
			.line_to([100.0, 200.0]);
		let path_bufs = PathBufs::new(device, &path);

		// let wind_cmds = path_pipelines.wind.draw(device, &path_bufs);
		// queue.submit(Some(wind_cmds));

		let intermediate_bufs = IntermediateBufs::new(device, config.width, config.height);

		Example { path_bufs, path_pipelines, intermediate_bufs }
	}

	fn update(&mut self, _event: winit::event::WindowEvent) {
		// empty
	}

	fn resize(&mut self, _config: &wgpu::SurfaceConfiguration, _device: &wgpu::Device, _queue: &wgpu::Queue) {
		// let view_size = &[config.width as f32, config.height as f32];
		// queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(view_size));
	}

	fn render(
		&mut self,
		view: &wgpu::TextureView,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		spawner: &framework::Spawner,
	) {
		device.push_error_scope(wgpu::ErrorFilter::Validation);

		let jitter_cmds = self.path_pipelines.jitter.draw(device, &self.intermediate_bufs, &self.path_bufs);
		let rasterize_cmds = self.path_pipelines.rasterize.draw(device, &self.intermediate_bufs, view);
		queue.submit(vec![jitter_cmds, rasterize_cmds]);

		spawner.spawn_local(ErrorFuture { inner: device.pop_error_scope() });
	}
}

fn main() {
	framework::run::<Example>("shape");
}
