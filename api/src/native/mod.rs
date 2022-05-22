use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::{Window},
};
use wgpu::util::DeviceExt;

use super::{
	Component,
	Element,
	// Bounds,
	Root,
	Rect,
	Span,
	Text,
	Group,
};

#[derive(Debug, Clone)]
pub struct FloatBounds {
	pub x: f32,
	pub y: f32,
	pub width: f32,
	pub height: f32,
}

pub struct ComponentWindow<T: Component + 'static> {
	window: Window,
	background: wgpu::Color,
	surface: wgpu::Surface,
	event_loop: EventLoop<()>,
	instance: wgpu::Instance,
	component: T,
	root: Element,
}

pub trait RenderNative {
	fn render(&self, _ctx: &ElementContext, _encoder: &mut wgpu::CommandEncoder) {}
}

impl RenderNative for Root {}
impl RenderNative for Group {}
impl RenderNative for Span {}
impl RenderNative for Text {}

impl RenderNative for Rect {
	fn render(&self, ctx: &ElementContext, encoder: &mut wgpu::CommandEncoder) {
		let view = ctx.frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
		
		let pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Default::default(),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				..Default::default()
			},
			vertex: wgpu::VertexState {
				module: &ctx.device.create_shader_module(&wgpu::include_wgsl!("vertex.wgsl")),
				entry_point: "vs_main",
				buffers: &[Vertex::layout()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &ctx.device.create_shader_module(&wgpu::include_wgsl!("fragment.wgsl")),
				entry_point: "fs_main",
				targets: &[wgpu::ColorTargetState {
					format: ctx.texture_format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				}],
			}),
			depth_stencil: None,
			multisample: Default::default(),
			multiview: None,
		});
		
		let (vertices, indices) = self.build_vertices(ctx);
		let vertex_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			contents: bytemuck::cast_slice(&vertices),
			usage: wgpu::BufferUsages::VERTEX,
		});
		let index_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			contents: bytemuck::cast_slice(&indices),
			usage: wgpu::BufferUsages::INDEX,
		});
		
		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

impl Rect {
	fn build_vertices(&self, ctx: &ElementContext) -> (Vec<Vertex>, Vec<u16>) {
		let vw = ctx.vw / ctx.scale_factor / 2.0;
		let vh = ctx.vh / ctx.scale_factor / 2.0;
		let x1 = ctx.bounds.x / vw - 1.0;
		let x2 = x1 + ctx.bounds.width / vw;
		let y1 = -ctx.bounds.y / vh + 1.0;
		let y2 = y1 - ctx.bounds.height / vh;
		let vertices = vertices(
			self.color.r,
			self.color.g,
			self.color.b,
			self.color.a, &[
			(x1, y1),(x1, y2),
			(x2, y1),(x2, y2),
		]);
		let indices = vec![0, 1, 2, 1, 2, 3];
		(vertices, indices)
	}
}

impl RenderNative for Element {
	fn render(&self, parent_ctx: &ElementContext, encoder: &mut wgpu::CommandEncoder) {
		let ctx = self.create_context(parent_ctx);

		self.element_impl.render(&ctx, encoder);

		for e in self.children.iter() {
			if e.show {
				e.render(&ctx, encoder);
			}
		}
	}
}

impl <T: Component> ComponentWindow<T> {
	pub fn new(window_builder: winit::window::WindowBuilder, component: T) -> Self {
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
			component,
			root: Element::root(),
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
			let frame = self.surface.get_current_texture().unwrap();
			let ctx = ElementContext::root_context(
				&frame,
				config.format,
				&device,
				&queue,
				config.width as f32,
				config.height as f32,
				self.window.scale_factor() as f32,
			);

			match event {
				Event::RedrawRequested(_) => {
					let view = ctx.frame
						.texture
						.create_view(&wgpu::TextureViewDescriptor::default());
					let mut encoder = ctx.device
						.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
					
					{
						let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
							label: None,
							color_attachments: &[wgpu::RenderPassColorAttachment {
								view: &view,
								resolve_target: None,
								ops: wgpu::Operations {
									load: wgpu::LoadOp::Clear(self.background),
									store: true,
								},
							}],
							depth_stencil_attachment: None,
						});
					}

					self.component.update(&mut self.root);
					RenderNative::render(&self.root, &ctx, &mut encoder);

					ctx.queue.submit(Some(encoder.finish()));
					frame.present();
				},
				Event::WindowEvent {
					window_id: _,
					event: WindowEvent::Resized(size),
					..
				} => {
					drop(frame);
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

impl Element {
	fn create_context<'a>(&self, parent: &'a ElementContext) -> ElementContext<'a> {
		let mut bounds = parent.bounds.clone();
		if let Some(b) = self.element_impl.bounds() {
			bounds.x += b.x;
			bounds.y += b.y;
			bounds.width = b.width;
			bounds.height = b.height;
		}
		ElementContext {
			parent: Some(parent),
			frame: parent.frame,
			texture_format: parent.texture_format,
			device: parent.device,
			queue: parent.queue,
			scale_factor: parent.scale_factor,
			vw: parent.vw,
			vh: parent.vh,
			bounds,
		}
	}
}

pub struct ElementContext<'a> {
	pub parent: Option<&'a ElementContext<'a>>,
	pub scale_factor: f32,
	pub vw: f32,
	pub vh: f32,
	pub bounds: FloatBounds,
	pub frame: &'a wgpu::SurfaceTexture,
	pub texture_format: wgpu::TextureFormat,
	pub device: &'a wgpu::Device,
	pub queue: &'a wgpu::Queue,
}

impl <'a> ElementContext<'a> {
	fn root_context(
		frame: &'a wgpu::SurfaceTexture,
		texture_format: wgpu::TextureFormat,
		device: &'a wgpu::Device,
		queue: &'a wgpu::Queue,
		width: f32,
		height: f32,
		scale_factor: f32,
	) -> Self {
		ElementContext {
			parent: None,
			frame,
			texture_format,
			device,
			queue,
			vw: width,
			vh: height,
			scale_factor,
			bounds: FloatBounds { x: 0.0, y: 0.0, width, height },
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