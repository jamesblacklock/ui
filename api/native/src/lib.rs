use std::rc::Rc;
use std::cell::RefCell;
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::{Window},
};
use wgpu::util::DeviceExt;
use wgpu_text::{
	BrushBuilder,
	section::OwnedSection
};

pub use ui_base::*;

#[derive(Default, Debug)]
pub struct NativeElementData;
impl ElementData for NativeElementData {}
pub type Element = GenericElement<NativeElementData>;

pub type Abi = NoAbi;

const TIMES_NEW_ROMAN: &[u8] = include_bytes!("./Times New Roman.ttf");

pub struct ComponentWindow<C: ComponentBase + 'static> {
	window: Window,
	background: wgpu::Color,
	surface: wgpu::Surface,
	event_loop: EventLoop<()>,
	instance: wgpu::Instance,
	component: Rc<RefCell<C>>,
	root: Element,
	pointer_position: Option<(f32, f32)>,
}

pub struct RenderContext<'a> {
	pub encoder: wgpu::CommandEncoder,
	pub bufs: Vec<wgpu::CommandBuffer>,
	pub frame: wgpu::SurfaceTexture,
	pub surface_config: &'a wgpu::SurfaceConfiguration,
	pub device: &'a wgpu::Device,
	pub queue: &'a wgpu::Queue,
	pub text_sections: Vec<OwnedSection>,
}

pub trait RenderNative {
	fn render(&self, _ectx: &ElementContext, _rctx: &mut RenderContext) {}
}

impl RenderNative for ElementImpl {
	fn render<'a>(&self, ectx: &ElementContext, rctx: &mut RenderContext) {
		match self {
			ElementImpl::Root(..)|ElementImpl::Group => {},
			ElementImpl::Rect(rect) => RenderNative::render(rect, ectx, rctx),
			ElementImpl::Span(_span) => {},
			ElementImpl::Text(text) => RenderNative::render(text, ectx, rctx),
		}
	}
}

impl RenderNative for Text {
	fn render(&self, _ectx: &ElementContext, rctx: &mut RenderContext) {
		rctx.text_sections.last_mut().unwrap().text.push(
			wgpu_text::section::OwnedText::new(&self.content)
				.with_scale(35.0)
				.with_color([0.0, 0.0, 0.0, 1.0]),
		);
	}
}

impl RenderNative for Rect {
	fn render(&self, ectx: &ElementContext, rctx: &mut RenderContext) {
		let view = rctx.frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
		
		let pipeline = rctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Default::default(),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				..Default::default()
			},
			vertex: wgpu::VertexState {
				module: &rctx.device.create_shader_module(&wgpu::include_wgsl!("vertex.wgsl")),
				entry_point: "vs_main",
				buffers: &[Vertex::layout()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &rctx.device.create_shader_module(&wgpu::include_wgsl!("fragment.wgsl")),
				entry_point: "fs_main",
				targets: &[wgpu::ColorTargetState {
					format: rctx.surface_config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				}],
			}),
			depth_stencil: None,
			multisample: Default::default(),
			multiview: None,
		});
		
		let (vertices, indices) = build_rect_vertices(self, ectx);
		let vertex_buf = rctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			contents: bytemuck::cast_slice(&vertices),
			usage: wgpu::BufferUsages::VERTEX,
		});
		let index_buf = rctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			contents: bytemuck::cast_slice(&indices),
			usage: wgpu::BufferUsages::INDEX,
		});
		
		{
			let mut render_pass = rctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Load,
						store: true,
					},
				}],
				depth_stencil_attachment: None,
			});
			render_pass.set_pipeline(&pipeline);
			render_pass.set_vertex_buffer(0, vertex_buf.slice(..));
			render_pass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint16);
			render_pass.draw_indexed(0..6, 0, 0..1);
		}
	}
}

fn build_rect_vertices(rect: &Rect, ctx: &ElementContext) -> (Vec<Vertex>, Vec<u16>) {
	let vw = ctx.vw / ctx.scale_factor / 2.0;
	let vh = ctx.vh / ctx.scale_factor / 2.0;
	let x1 = ctx.bounds.x / vw - 1.0;
	let x2 = x1 + ctx.bounds.width / vw;
	let y1 = -ctx.bounds.y / vh + 1.0;
	let y2 = y1 - ctx.bounds.height / vh;
	let vertices = vertices(
		rect.color.r,
		rect.color.g,
		rect.color.b,
		rect.color.a, &[
		(x1, y1),(x1, y2),
		(x2, y1),(x2, y2),
	]);
	let indices = vec![0, 1, 2, 1, 2, 3];
	(vertices, indices)
}

impl RenderNative for Element {
	fn render(&self, parent_ctx: &ElementContext, rctx: &mut RenderContext) {
		let ctx = create_context(self, parent_ctx);

		self.element_impl.render(&ctx, rctx);

		let render_text = if let Some(_) = self.element_impl.bounds() {
			rctx.text_sections.push(create_section(ctx.bounds.to_raw(ctx.scale_factor)));
			true
		} else {
			false
		};

		for e in self.children.iter() {
			if e.show {
				e.render(&ctx, rctx);
			}
		}

		if render_text {
			let section = rctx.text_sections.pop().unwrap();

			let mut brush = BrushBuilder::using_font_bytes(TIMES_NEW_ROMAN)
			.unwrap()
			.build(&rctx.device, &rctx.surface_config);

			brush.queue(&section);

			let view = rctx.frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
			let buf = brush.draw(rctx.device, &view, rctx.queue);
			rctx.bufs.push(buf);
		}
	}
}

fn bounds_contain_point(b: &PxBounds, point: &(f32, f32)) -> bool {
	b.x <= point.0 && b.y <= point.1 && b.x+b.width >= point.0 && b.y+b.height >= point.1
}

fn find_element_at_px_point(e: &Element, point: (f32, f32)) -> &Element {
	let bounds = e.element_impl.bounds().unwrap();
	assert!(bounds_contain_point(&bounds, &point));

	let point = (point.0 - bounds.x, point.1 - bounds.y);
	for c in &e.children {
		if let Some(bounds) = c.element_impl.bounds() {
			if bounds_contain_point(&bounds, &point) {
				return find_element_at_px_point(c, point);
			}
		}
	}

	e
}

impl <C: ComponentBase> ComponentWindow<C> {
	pub fn new(window_builder: winit::window::WindowBuilder, component: C) -> Self {
		let event_loop = EventLoop::new();
		let window = window_builder.build(&event_loop).unwrap();

		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe { instance.create_surface(&window) };
		
		Self {
			window,
			background: wgpu::Color::WHITE,
			surface,
			event_loop,
			instance,
			component: Rc::new(RefCell::new(component)),
			root: Element::root(),
			pointer_position: None,
		}
	}

	pub async fn run(mut self) {
		let adapter = self.instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				compatible_surface: Some(&self.surface),
				..Default::default()
			})
			.await
			.expect("Failed to find an appropriate adapter");
		
		let size = self.window.inner_size();
		let mut config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: self.surface.get_preferred_format(&adapter).unwrap(),
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
		};

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					features: wgpu::Features::empty(),
					limits: wgpu::Limits::downlevel_defaults(),
				},
				None,
			)
			.await
			.expect("Failed to create device");

		self.surface.configure(&device, &config);

		self.event_loop.run(move |event, _, control_flow| {
			let ctx = ElementContext::root_context(
				config.width as f32,
				config.height as f32,
				self.window.scale_factor() as f32,
			);

			self.root.element_impl = ElementImpl::Root(ctx.vw / ctx.scale_factor, ctx.vh / ctx.scale_factor);

			match event {
				Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
					*self.pointer_position.as_mut().unwrap() =
						(position.x as f32 / ctx.scale_factor, position.y as f32 / ctx.scale_factor);
				},
				Event::WindowEvent { event: WindowEvent::CursorEntered {..}, .. } => {
					self.pointer_position = Some((f32::INFINITY, f32::INFINITY));
				},
				Event::WindowEvent { event: WindowEvent::CursorLeft {..}, .. } => {
					self.pointer_position = None;
				},
				Event::WindowEvent { event: WindowEvent::MouseInput { state, button, .. }, .. } => {
					if state == winit::event::ElementState::Released && button == winit::event::MouseButton::Left {
						let e = find_element_at_px_point(&self.root, self.pointer_position.unwrap());
						if let Some(callback) = &e.events.pointer_click {
							callback.call();
							self.window.request_redraw();
						}
					}
				},
				Event::RedrawRequested(_) => {
					ComponentBase::update(self.component.clone(), &mut self.root);

					let root_text_section = create_section(
						RawBounds {
							x: 0.0,
							y: 0.0,
							width: config.width as f32,
							height: config.height as f32
						},
					);
			
					let mut rctx = RenderContext {
						device: &device,
						queue: &queue,
						surface_config: &config,
						frame: self.surface.get_current_texture().unwrap(),
						encoder: device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }),
						bufs: Vec::new(),
						text_sections: vec![root_text_section],
					};

					{
						let _ = rctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
							label: None,
							color_attachments: &[wgpu::RenderPassColorAttachment {
								view: &rctx.frame.texture.create_view(&wgpu::TextureViewDescriptor::default()),
								resolve_target: None,
								ops: wgpu::Operations {
									load: wgpu::LoadOp::Clear(self.background),
									store: true,
								},
							}],
							depth_stencil_attachment: None,
						});
					}
					
					RenderNative::render(&self.root, &ctx, &mut rctx);

					queue.submit(Some(rctx.encoder.finish()));
					queue.submit(rctx.bufs);

					rctx.frame.present();
				},
				Event::WindowEvent {
					window_id: _,
					event: WindowEvent::Resized(size),
					..
				} => {
					config.width = size.width;
					config.height = size.height;
					self.surface.configure(&device, &config);
					self.window.request_redraw();
				},
	
				Event::WindowEvent {
					window_id: _,
					event: WindowEvent::CloseRequested,
					..
				} => {
					*control_flow = ControlFlow::Exit;
				}
				_ => {}
			}
		});
	}
}

fn create_context<'a>(e: &'a Element, parent: &'a ElementContext) -> ElementContext<'a> {
	let mut bounds = parent.bounds.clone();
	if let Some(b) = e.element_impl.bounds() {
		bounds.x += b.x;
		bounds.y += b.y;
		bounds.width = b.width;
		bounds.height = b.height;
	}
	ElementContext {
		parent: Some(parent),
		scale_factor: parent.scale_factor,
		vw: parent.vw,
		vh: parent.vh,
		bounds,
	}
}

pub struct ElementContext<'a> {
	pub parent: Option<&'a ElementContext<'a>>,
	pub scale_factor: f32,
	pub vw: f32,
	pub vh: f32,
	pub bounds: PxBounds,
}

fn create_section(bounds: RawBounds) -> wgpu_text::section::OwnedSection {
	use wgpu_text::{
		section::{
			BuiltInLineBreaker,
			Layout,
			Section,
			VerticalAlign,
		}
	};
	Section::default()
		.with_bounds((bounds.width, bounds.height))
		.with_layout(
			Layout::default()
				.v_align(VerticalAlign::Top)
				.line_breaker(BuiltInLineBreaker::UnicodeLineBreaker),
		)
		.with_screen_position((bounds.x, bounds.y))
		.to_owned()
}

impl <'a> ElementContext<'a> {
	fn root_context(
		width: f32,
		height: f32,
		scale_factor: f32,
	) -> Self {
		ElementContext {
			parent: None,
			vw: width,
			vh: height,
			scale_factor,
			bounds: PxBounds { x: 0.0, y: 0.0, width, height },
		}
	}
}

use bytemuck::{Zeroable, Pod};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex {
	x: f32, y: f32, z: f32,
	r: f32, g: f32, b: f32, a: f32,
}

impl Vertex {
	fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: wgpu::VertexFormat::Float32x3,
				},
				wgpu::VertexAttribute {
					offset: (std::mem::size_of::<f32>() * 3) as wgpu::BufferAddress,
					shader_location: 1,
					format: wgpu::VertexFormat::Float32x4,
				},
			]
		}
	}
}

fn vertices(r: f32, g: f32, b: f32, a: f32, points: &[(f32,f32)]) -> Vec<Vertex> {
	points.iter().map(|&(x,y)| Vertex {
		x, y, z: 0.0,
		r, g, b, a,
	})
	.collect()
}