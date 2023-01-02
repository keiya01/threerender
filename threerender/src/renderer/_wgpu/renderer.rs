use std::{borrow::Cow, mem};

use wgpu::{
    util::{align_to, DeviceExt},
    vertex_attr_array, Adapter, BindGroup, Buffer, BufferAddress, Device, PipelineLayout, Queue,
    RenderPipeline, ShaderModule, Surface, SurfaceConfiguration, TextureFormat, VertexBufferLayout,
};

use crate::{
    entity::{Entity, EntityDescriptor},
    mesh::util::Vertex,
    renderer::Updater,
    RendererBuilder,
};

use super::{entity::EntityUniformBuffer, scene::Scene, unit::rgba_to_array};

struct RenderedEntityMeta {
    uniform_offset: BufferAddress,
    vertex_buf: Buffer,
    index_buf: Option<Buffer>,
    vertex_length: u32,
    index_length: u32,
}

// The struct will be depend on entity.
// These properties will be recreated when the entity is updated other than uniform.
pub struct RenderedEntity {
    pub(super) entities: Vec<Entity>,
    meta_list: Vec<RenderedEntityMeta>,
    shader: ShaderModule,
    pipeline_layout: PipelineLayout,
    render_pipeline: RenderPipeline,
    entity_uniform_buf: Buffer,
    entity_bind_group: BindGroup,
}

// The struct is immutable basically.
pub struct Renderer {
    pub(super) rendered_entity: Option<RenderedEntity>,
    pub(super) device: Device,
    pub(super) config: SurfaceConfiguration,
    pub(super) adapter: Adapter,
    pub(super) surface: Surface,
    pub(super) queue: Queue,
    pub(super) scene: Scene,
}

impl Renderer {
    const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn new<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        window: &W,
        mut renderer_builder: RendererBuilder,
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits:
                    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            },
            None,
        ))
        .expect("Failed to create device");

        let config = surface
            .get_default_config(&adapter, renderer_builder.width, renderer_builder.height)
            .expect("Surface isn't supported by the adapter.");

        surface.configure(&device, &config);

        let scene = Scene::new(&device, &config, renderer_builder.scene.take().unwrap());
        let mut renderer = Self {
            rendered_entity: None,
            device,
            config,
            surface,
            queue,
            adapter,
            scene,
        };

        if renderer_builder.enable_forward_depth {
            renderer.set_depth_texture();
        }

        renderer.update_renderer_entity(renderer_builder);

        renderer
    }

    fn set_depth_texture(&mut self) {
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        });

        self.scene.forward_depth =
            Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));
    }

    pub fn update_renderer_entity(&mut self, renderer_builder: RendererBuilder) {
        // Load the shaders from disk
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/3d.wgsl"))),
            });

        let entity_uniform_size = mem::size_of::<EntityUniformBuffer>() as wgpu::BufferAddress;
        let entity_uniform_alignment = {
            let alignment =
                self.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(entity_uniform_size, alignment)
        };
        let entities_length = renderer_builder.entities.len() as u64;
        let entity_uniform_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Transform Uniform Buffer"),
            size: entities_length * entity_uniform_alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut entities = vec![];
        let mut meta_list = vec![];
        for (
            i,
            EntityDescriptor {
                mesh,
                fill_color,
                coordinates,
            },
        ) in renderer_builder.entities.into_iter().enumerate()
        {
            let vertex_buf = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(mesh.vertex()),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            let index_buf = mesh.index().map(|index| {
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(index),
                        usage: wgpu::BufferUsages::INDEX,
                    })
            });
            entities.push(Entity {
                fill_color,
                coordinates,
            });
            meta_list.push(RenderedEntityMeta {
                uniform_offset: (i as u64) * entity_uniform_alignment,
                vertex_buf,
                index_buf,
                vertex_length: mesh.vertex().len() as u32,
                index_length: mesh.index().map_or(0, |i| i.len()) as u32,
            })
        }

        let entity_bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0, // transform
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: wgpu::BufferSize::new(entity_uniform_size),
                        },
                        count: None,
                    }],
                });

        let entity_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &entity_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &entity_uniform_buf,
                    offset: 0,
                    size: wgpu::BufferSize::new(entity_uniform_size),
                }),
            }],
            label: None,
        });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &self.scene.model_uniform.bind_group_layout,
                    &self.scene.light_uniform.bind_group_layout,
                    &entity_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[VertexBufferLayout {
                        array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![0 => Float32x4, 1 => Float32x3],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(self.config.format.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Self::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        self.rendered_entity = Some(RenderedEntity {
            entities,
            meta_list,
            shader,
            pipeline_layout,
            render_pipeline,
            entity_uniform_buf,
            entity_bind_group,
        });
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        // Reconfigure the surface with the new size
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.scene.update_model(&self.queue, &self.config);

        self.set_depth_texture();
    }

    pub fn update(&mut self, updater: &mut dyn Updater) {
        match &mut self.rendered_entity {
            Some(entity) => updater.update(&mut entity.entities, &mut self.scene.style),
            None => {}
        }

        // TODO: Invoke it only when light is changed
        self.scene.update_light(&self.queue);
    }

    pub fn draw(&self) {
        let state = match &self.rendered_entity {
            Some(state) => state,
            None => return,
        };

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                })],
                depth_stencil_attachment: self.scene.forward_depth.as_ref().map(|view| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }
                }),
            });
            rpass.set_pipeline(&state.render_pipeline);
            rpass.set_bind_group(0, &self.scene.model_uniform.bind_group, &[]);
            rpass.set_bind_group(1, &self.scene.light_uniform.bind_group, &[]);

            for (i, entity) in state.entities.iter().enumerate() {
                let meta = state
                    .meta_list
                    .get(i)
                    .expect("The length of meta_list must match with entities");
                self.prepare_entity(entity, &meta);
                rpass.set_bind_group(2, &state.entity_bind_group, &[meta.uniform_offset as u32]);
                rpass.set_vertex_buffer(0, meta.vertex_buf.slice(..));
                match &meta.index_buf {
                    Some(index_buf) => {
                        rpass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint16);
                        rpass.draw_indexed(0..meta.index_length, 0, 0..1);
                    }
                    None => rpass.draw(0..meta.vertex_length, 0..1),
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn prepare_entity(&self, entity: &Entity, meta: &RenderedEntityMeta) {
        match &self.rendered_entity {
            Some(renderer_entity) => {
                let buf = EntityUniformBuffer {
                    transform: entity.coordinates.inner().to_cols_array_2d(),
                    color: rgba_to_array(&entity.fill_color),
                };
                self.queue.write_buffer(
                    &renderer_entity.entity_uniform_buf,
                    meta.uniform_offset,
                    bytemuck::bytes_of(&buf),
                );
            }
            None => {}
        }
    }
}
