use std::{borrow::Cow, marker::PhantomData, mem};

use wgpu::{
    util::{align_to, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupLayout, Buffer, BufferAddress, Device, Queue,
    RenderPipeline, Surface, SurfaceConfiguration, TextureFormat, VertexBufferLayout,
};

use crate::{
    entity::{Entity, EntityDescriptor, EntityList},
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
pub struct RenderedEntity {
    pub(super) entities: Vec<Entity>,
    meta_list: Vec<RenderedEntityMeta>,
    entity_uniform_buf: Buffer,
    entity_bind_group: BindGroup,
    entity_bind_group_layout: BindGroupLayout,
    entity_uniform_alignment: u64,
}

// This has a role to update the entity in render process dynamically.
pub(super) struct DynamicRenderer {
    pub(super) rendered_entity: RenderedEntity,
    pub(super) device: Device,
}

impl DynamicRenderer {
    pub fn new(device: Device, renderer_builder: &mut RendererBuilder) -> Self {
        let entity_uniform_size = mem::size_of::<EntityUniformBuffer>() as wgpu::BufferAddress;
        let entity_uniform_alignment = {
            let alignment =
                device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(entity_uniform_size, alignment)
        };
        let entities_length = renderer_builder
            .renderer_specific_attributes
            .maximum_entity_length;
        let entity_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Transform Uniform Buffer"),
            size: entities_length * entity_uniform_alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut entities = vec![];
        let mut meta_list = vec![];
        let mut i = 0;
        while let Some(EntityDescriptor {
            id,
            mesh,
            fill_color,
            coordinates,
        }) = renderer_builder.entities.pop()
        {
            let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(mesh.vertex()),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buf = mesh.index().map(|index| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(index),
                    usage: wgpu::BufferUsages::INDEX,
                })
            });
            entities.push(Entity {
                id,
                fill_color,
                coordinates,
            });
            meta_list.push(RenderedEntityMeta {
                uniform_offset: (i as u64) * entity_uniform_alignment,
                vertex_buf,
                index_buf,
                vertex_length: mesh.vertex().len() as u32,
                index_length: mesh.index().map_or(0, |i| i.len()) as u32,
            });

            i += 1;
        }

        let entity_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let entity_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        DynamicRenderer {
            device,
            rendered_entity: RenderedEntity {
                entities,
                meta_list,
                entity_uniform_buf,
                entity_bind_group,
                entity_bind_group_layout,
                entity_uniform_alignment,
            },
        }
    }
}

impl EntityList for DynamicRenderer {
    fn items(&self) -> &[Entity] {
        &self.rendered_entity.entities
    }

    fn items_mut(&mut self) -> &mut [Entity] {
        &mut self.rendered_entity.entities
    }

    fn push(&mut self, descriptor: EntityDescriptor) {
        let rendered_entity = &mut self.rendered_entity;

        let EntityDescriptor {
            id,
            fill_color,
            coordinates,
            mesh,
        } = descriptor;

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
        rendered_entity.entities.push(Entity {
            id,
            fill_color,
            coordinates,
        });
        rendered_entity.meta_list.push(RenderedEntityMeta {
            uniform_offset: (rendered_entity.meta_list.len() as u64)
                * rendered_entity.entity_uniform_alignment,
            vertex_buf,
            index_buf,
            vertex_length: mesh.vertex().len() as u32,
            index_length: mesh.index().map_or(0, |i| i.len()) as u32,
        });
    }
}

// The struct is immutable basically.
pub struct Renderer<Event> {
    pub(super) dynamic_renderer: DynamicRenderer,
    pub(super) config: SurfaceConfiguration,
    pub(super) surface: Surface,
    pub(super) queue: Queue,
    pub(super) scene: Scene,
    render_pipeline: RenderPipeline,

    phantom_event: PhantomData<Event>,
}

impl<Event> Renderer<Event> {
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

        let dynamic_renderer = DynamicRenderer::new(device, &mut renderer_builder);

        // Load the shaders from disk
        let shader = dynamic_renderer
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/3d.wgsl"))),
            });

        let pipeline_layout =
            dynamic_renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &scene.model_uniform.bind_group_layout,
                        &scene.light_uniform.bind_group_layout,
                        &dynamic_renderer.rendered_entity.entity_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            dynamic_renderer
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
                        targets: &[Some(config.format.into())],
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

        let mut renderer = Self {
            dynamic_renderer,
            config,
            surface,
            queue,
            scene,
            render_pipeline,
            phantom_event: PhantomData,
        };

        if renderer_builder.enable_forward_depth {
            renderer.set_depth_texture();
        }

        renderer
    }

    fn set_depth_texture(&mut self) {
        let depth_texture = self
            .dynamic_renderer
            .device
            .create_texture(&wgpu::TextureDescriptor {
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

    pub fn resize(&mut self, width: u32, height: u32) {
        // Reconfigure the surface with the new size
        self.config.width = width;
        self.config.height = height;
        self.surface
            .configure(&self.dynamic_renderer.device, &self.config);
        self.scene.update_model(&self.queue, &self.config);

        self.set_depth_texture();
    }

    pub fn update(&mut self, updater: &mut dyn Updater<Event = Event>, ev: Event) {
        updater.update(&mut self.dynamic_renderer, &mut self.scene.style, ev);

        // TODO: Invoke it only when light is changed
        self.scene.update_light(&self.queue);
    }

    pub fn draw(&self) {
        let rendered_entity = &self.dynamic_renderer.rendered_entity;

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .dynamic_renderer
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
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.scene.model_uniform.bind_group, &[]);
            rpass.set_bind_group(1, &self.scene.light_uniform.bind_group, &[]);

            for (i, entity) in rendered_entity.entities.iter().enumerate() {
                let meta = rendered_entity
                    .meta_list
                    .get(i)
                    .expect("The length of meta_list must match with entities");
                self.prepare_entity(entity, &meta);
                rpass.set_bind_group(
                    2,
                    &rendered_entity.entity_bind_group,
                    &[meta.uniform_offset as u32],
                );
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
        let renderer_entity = &self.dynamic_renderer.rendered_entity;
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
}
