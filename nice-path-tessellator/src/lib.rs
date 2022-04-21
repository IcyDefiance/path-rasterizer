use std::f32::INFINITY;

use array_init::array_init;
use lyon::lyon_tessellation::{BuffersBuilder, FillOptions, FillVertex, VertexBuffers};
use lyon::path::Path as LyonPath;
use lyon::tessellation::FillTessellator as LyonFillTessellator;

#[derive(Clone, Debug)]
pub struct Path {
	commands: Vec<PathCommands>,
}
impl Path {
	pub fn start() -> Self {
		Self { commands: vec![] }
	}

	pub fn move_to(mut self, to: [f32; 2]) -> Self {
		self.commands.push(PathCommands::MoveTo(to));
		self
	}

	pub fn line_to(mut self, to: [f32; 2]) -> Self {
		self.commands.push(PathCommands::LineTo(to));
		self
	}

	pub fn quadratic_curve_to(mut self, ctrl: [f32; 2], to: [f32; 2]) -> Self {
		self.commands.push(PathCommands::QuadraticCurveTo(ctrl, to));
		self
	}
}

#[derive(Clone, Copy, Debug)]
pub enum PathCommands {
	MoveTo([f32; 2]),
	LineTo([f32; 2]),
	/// ctrl, to
	QuadraticCurveTo([f32; 2], [f32; 2]),
}

pub fn fill_tessellate(path: &Path) -> VecPathVertBufs {
	let mut lyon_builder = LyonPath::builder();
	for &cmd in &path.commands {
		match cmd {
			PathCommands::MoveTo(to) => lyon_builder.begin(to.into()),
			PathCommands::LineTo(to) => lyon_builder.line_to(to.into()),
			PathCommands::QuadraticCurveTo(ctrl, to) => lyon_builder.quadratic_bezier_to(ctrl.into(), to.into()),
		};
	}
	lyon_builder.close();
	let mut geometry: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
	LyonFillTessellator::new()
		.tessellate_path(
			&lyon_builder.build(),
			&FillOptions::tolerance(INFINITY),
			&mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| vertex.position().to_array()),
		)
		.unwrap();

	let mut bufs = VecPathVertBufs::new();
	bufs.verts = geometry.vertices;
	bufs.fill_idxs = geometry.indices;
	let mut tess = FillTessellator::start(&mut bufs);
	for &cmd in &path.commands {
		match cmd {
			PathCommands::MoveTo(to) => tess = tess.move_to(to),
			PathCommands::LineTo(to) => tess = tess.line_to(to),
			PathCommands::QuadraticCurveTo(ctrl, to) => tess = tess.quadratic_curve_to(ctrl, to),
		}
	}

	bufs
}

#[derive(Default)]
pub struct VertCounter {
	vert_count: u16,
	fill_idx_count: u16,
	quadratic_idx_count: u16,
}
impl BufsBuilder for VertCounter {
	fn verts_len(&self) -> u16 {
		self.vert_count
	}

	fn push_vert(&mut self, _pos: [f32; 2]) {
		self.vert_count += 1;
	}

	fn push_fill_tri(&mut self, _idxs: [u16; 3]) {
		self.fill_idx_count += 3;
	}

	fn push_quadratic_tri(&mut self, _idxs: [u16; 3]) {
		self.quadratic_idx_count += 3;
	}
}

pub struct FillTessellator<'a> {
	bufs: &'a mut dyn BufsBuilder,
	start: Vert,
	pen: Vert,
}
impl<'a> FillTessellator<'a> {
	pub fn start(bufs: &mut dyn BufsBuilder) -> FillTessellator {
		FillTessellator { bufs, start: Vert::Pos([0.0; 2]), pen: Vert::Pos([0.0; 2]) }
	}

	pub fn move_to(mut self, to: [f32; 2]) -> Self {
		self.start = Vert::Pos(to);
		self.pen = Vert::Pos(to);
		self
	}

	pub fn line_to(mut self, to: [f32; 2]) -> Self {
		self.line_to_impl(Vert::Pos(to));
		self
	}

	fn quadratic_curve_to(mut self, ctrl: [f32; 2], to: [f32; 2]) -> Self {
		let mut to = Vert::Pos(to);
		[self.pen, _, to] = self.push_quadratic_tri([self.pen, Vert::Pos(ctrl), to]);
		self.line_to_impl(to);
		self
	}

	fn line_to_impl(&mut self, to: Vert) {
		// if self.pen != self.start {
		// 	[self.start, self.pen, to] = self.push_fill_tri([self.start, self.pen, to]);
		// }
		self.pen = to;
	}

	// fn push_fill_tri(&mut self, verts: [Vert; 3]) -> [Vert; 3] {
	// let idxs = self.make_tri(verts);
	// self.bufs.push_fill_tri(idxs);
	// array_init(|i| Vert::Idx(idxs[i]))
	// 	verts
	// }

	fn push_quadratic_tri(&mut self, verts: [Vert; 3]) -> [Vert; 3] {
		let idxs = self.make_tri(verts);
		self.bufs.push_quadratic_tri(idxs);
		array_init(|i| Vert::Idx(idxs[i]))
	}

	fn make_tri(&mut self, verts: [Vert; 3]) -> [u16; 3] {
		let mut idxs = [0; 3];
		for (i, &vert) in verts.iter().enumerate() {
			match vert {
				Vert::Pos(pos) => {
					idxs[i] = self.bufs.verts_len();
					self.bufs.push_vert(pos);
				},
				Vert::Idx(idx) => idxs[i] = idx,
			}
		}
		idxs
	}
}
pub trait BufsBuilder {
	fn verts_len(&self) -> u16;
	fn push_vert(&mut self, pos: [f32; 2]);
	fn push_fill_tri(&mut self, idxs: [u16; 3]);
	fn push_quadratic_tri(&mut self, idxs: [u16; 3]);
}

#[derive(Clone, Debug)]
pub struct VecPathVertBufs {
	pub verts: Vec<[f32; 2]>,
	pub fill_idxs: Vec<u16>,
	pub quadratic_idxs: Vec<u16>,
	pub left_top: [Option<f32>; 2],
	pub right_bottom: [Option<f32>; 2],
}
impl VecPathVertBufs {
	pub fn new() -> Self {
		VecPathVertBufs {
			verts: vec![],
			fill_idxs: vec![],
			quadratic_idxs: vec![],
			left_top: [None; 2],
			right_bottom: [None; 2],
		}
	}
}
impl BufsBuilder for VecPathVertBufs {
	fn verts_len(&self) -> u16 {
		self.verts.len() as _
	}

	fn push_vert(&mut self, pos: [f32; 2]) {
		self.left_top[0] = self.left_top[0].map(|x| x.min(pos[0])).or(Some(pos[0]));
		self.left_top[1] = self.left_top[1].map(|x| x.min(pos[1])).or(Some(pos[1]));
		self.right_bottom[0] = self.right_bottom[0].map(|x| x.max(pos[0])).or(Some(pos[0]));
		self.right_bottom[1] = self.right_bottom[1].map(|x| x.max(pos[1])).or(Some(pos[1]));

		self.verts.push(pos);
	}

	fn push_fill_tri(&mut self, idxs: [u16; 3]) {
		self.fill_idxs.push(idxs[0]);
		self.fill_idxs.push(idxs[1]);
		self.fill_idxs.push(idxs[2]);
	}

	fn push_quadratic_tri(&mut self, idxs: [u16; 3]) {
		self.quadratic_idxs.push(idxs[0]);
		self.quadratic_idxs.push(idxs[1]);
		self.quadratic_idxs.push(idxs[2]);
	}
}

#[derive(Clone, Copy, PartialEq)]
pub enum Vert {
	Pos([f32; 2]),
	Idx(u16),
}
