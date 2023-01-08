use std::{borrow::Cow, collections::HashMap, marker::PhantomData, mem, num::NonZeroU32, rc::Rc};

use glam::{Mat4, Quat};
use wgpu::{
    util::{align_to, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupLayout, Buffer, BufferAddress, Device, Features,
    PrimitiveTopology, Queue, RenderPipeline, Sampler, ShaderModule, Surface, SurfaceConfiguration,
    TextureFormat, TextureView, VertexBufferLayout,
};

use crate::{
    entity::{Entity, EntityDescriptor, EntityList, EntityRendererState},
    mesh::{
        mesh::{Mesh, Texture2DMesh},
        util::{Texture2DVertex, Vertex},
        MeshType, PolygonMode, Topology, Texture2DFormat,
    },
    renderer::Updater,
    RendererBuilder,
};

use super::{uniform::EntityUniformBuffer, scene::Scene, unit::rgba_to_array};

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

// The struct will be depend on texture.
pub struct RenderedTexture2D {
    pub(super) texture2d_view_array: Vec<TextureView>,
    pub(super) sampler_array: Vec<Sampler>,
    texture2d_bind_group: BindGroup,
    texture2d_bind_group_layout: BindGroupLayout,
}

impl RenderedTexture2D {
    fn make_texture(
        texture2d_mesh: &dyn Texture2DMesh,
        device: &Device,
        queue: &Queue,
    ) -> (Sampler, TextureView) {
        let texture = {
            let buf = texture2d_mesh.data();
            let width = texture2d_mesh.width();
            let height = texture2d_mesh.height();

            let size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: match texture2d_mesh.format() {
                    Texture2DFormat::RGBA => wgpu::TextureFormat::Rgba8Unorm,
                },
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            });
            queue.write_texture(
                texture.as_image_copy(),
                &buf,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(width * texture2d_mesh.bytes_per_pixel()),
                    rows_per_image: None,
                },
                size,
            );
            texture
        };

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (sampler, view)
    }

    fn make_bind_group(
        device: &Device,
        texture2d_view_array: &[TextureView],
        sampler_array: &[Sampler],
    ) -> (BindGroupLayout, BindGroup) {
        let texture2d_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: NonZeroU32::new(texture2d_view_array.len() as u32),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: NonZeroU32::new(sampler_array.len() as u32),
                    },
                ],
            });

        let texture2d_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture2d_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(
                        &texture2d_view_array.iter().collect::<Vec<_>>(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::SamplerArray(
                        &sampler_array.iter().collect::<Vec<_>>(),
                    ),
                },
            ],
            label: None,
        });

        (texture2d_bind_group_layout, texture2d_bind_group)
    }

    fn create_or_update(
        texture2d_mesh: &dyn Texture2DMesh,
        device: &Device,
        queue: &Queue,
        rendered_texture2d: &mut Option<RenderedTexture2D>,
    ) {
        let (sampler, view) = Self::make_texture(texture2d_mesh, device, queue);

        let (mut texture2d_view_array, mut sampler_array) = (vec![view], vec![sampler]);

        if let Some(rt) = rendered_texture2d {
            texture2d_view_array.append(&mut rt.texture2d_view_array);
            sampler_array.append(&mut rt.sampler_array);
        }

        let (texture2d_bind_group_layout, texture2d_bind_group) =
            Self::make_bind_group(device, &texture2d_view_array, &sampler_array);

        match rendered_texture2d {
            Some(rt) => {
                rt.texture2d_view_array = texture2d_view_array;
                rt.sampler_array = sampler_array;

                rt.texture2d_bind_group_layout = texture2d_bind_group_layout;
                rt.texture2d_bind_group = texture2d_bind_group;
            }
            None => {
                *rendered_texture2d = Some(RenderedTexture2D {
                    texture2d_view_array,
                    sampler_array,
                    texture2d_bind_group,
                    texture2d_bind_group_layout,
                });
            }
        }
    }
}

// This has a role to update the entity in render process dynamically.
pub(super) struct DynamicRenderer {
    pub(super) rendered_entity: RenderedEntity,
    pub(super) rendered_texture2d: Option<RenderedTexture2D>,
    pub(super) device: Device,
    pub(super) queue: Queue,
}

impl DynamicRenderer {
    pub fn new(device: Device, queue: Queue, renderer_builder: &mut RendererBuilder) -> Self {
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
        let mut texture2d_view_array = vec![];
        let mut sampler_array = vec![];
        let mut i = 0;
        while let Some(EntityDescriptor {
            id,
            mesh,
            fill_color,
            position,
            dimension,
            rotation,
            state,
        }) = renderer_builder.entities.pop()
        {
            let vertex = match mesh.mesh_type() {
                MeshType::Entity => (Some(mesh.vertex()), None),
                MeshType::Texture2D => (None, Some(mesh.texture().expect("Texture is not found"))),
            };
            let vertex_buf = match vertex {
                (Some(vertex), None) => device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertex),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                (None, Some(texture)) => device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Texture Vertex Buffer"),
                    contents: bytemuck::cast_slice(texture),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                _ => unreachable!(),
            };
            let index_buf = mesh.index().map(|index| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(index),
                    usage: wgpu::BufferUsages::INDEX,
                })
            });

            if let Mesh::Texture2D(texture2d_mesh) = mesh.as_ref() {
                let (sampler, view) =
                    RenderedTexture2D::make_texture(texture2d_mesh.as_ref(), &device, &queue);

                texture2d_view_array.push(view);
                sampler_array.push(sampler);
            }

            entities.push(Entity {
                id,
                fill_color,
                position,
                dimension,
                rotation,
                state,
            });
            meta_list.push(RenderedEntityMeta {
                uniform_offset: (i as u64) * entity_uniform_alignment,
                vertex_buf,
                index_buf,
                vertex_length: match vertex {
                    (Some(v), None) => v.len(),
                    (None, Some(v)) => v.len(),
                    _ => unreachable!()
                } as u32,
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

        let rendered_texture2d = if texture2d_view_array.len() > 0 && sampler_array.len() > 0 {
            let (texture2d_bind_group_layout, texture2d_bind_group) =
                RenderedTexture2D::make_bind_group(&device, &texture2d_view_array, &sampler_array);

            Some(RenderedTexture2D {
                texture2d_view_array,
                sampler_array,
                texture2d_bind_group_layout,
                texture2d_bind_group,
            })
        } else {
            None
        };

        DynamicRenderer {
            device,
            queue,
            rendered_entity: RenderedEntity {
                entities,
                meta_list,
                entity_uniform_buf,
                entity_bind_group,
                entity_bind_group_layout,
                entity_uniform_alignment,
            },
            rendered_texture2d,
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
            position,
            dimension,
            rotation,
            mesh,
            state,
        } = descriptor;

        let vertex = match mesh.mesh_type() {
            MeshType::Entity => (Some(mesh.vertex()), None),
            MeshType::Texture2D => (None, Some(mesh.texture().expect("Texture is not found"))),
        };

        let vertex_buf = match vertex {
            (Some(vertex), None) => self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertex),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            (None, Some(texture)) => self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Texture Vertex Buffer"),
                contents: bytemuck::cast_slice(texture),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            _ => unreachable!(),
        };
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
            position,
            dimension,
            rotation,
            state,
        });
        rendered_entity.meta_list.push(RenderedEntityMeta {
            uniform_offset: (rendered_entity.meta_list.len() as u64)
                * rendered_entity.entity_uniform_alignment,
            vertex_buf,
            index_buf,
            vertex_length: match vertex {
                (Some(v), None) => v.len(),
                (None, Some(v)) => v.len(),
                _ => unreachable!()
            } as u32,
            index_length: mesh.index().map_or(0, |i| i.len()) as u32,
        });

        if let Mesh::Texture2D(texture2d_mesh) = mesh.as_ref() {
            RenderedTexture2D::create_or_update(
                texture2d_mesh.as_ref(),
                &self.device,
                &self.queue,
                &mut self.rendered_texture2d,
            );
        }
    }
}

// The struct is immutable basically.
pub struct Renderer<Event> {
    pub(super) dynamic_renderer: DynamicRenderer,
    pub(super) config: SurfaceConfiguration,
    pub(super) surface: Surface,
    pub(super) scene: Scene,
    render_pipelines: HashMap<EntityRendererState, RenderPipeline>,

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

        let adapter_features = if renderer_builder
            .renderer_specific_attributes
            .adapter_features
        {
            adapter.features()
        } else {
            Features::empty()
        };
        // Create the logical device and command queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: adapter_features | renderer_builder.renderer_specific_attributes.features,
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

        let dynamic_renderer = DynamicRenderer::new(device, queue, &mut renderer_builder);

        // Load the shaders from disk
        let mut entity_shader: Option<Rc<ShaderModule>> = None;
        let mut texture_shader: Option<Rc<ShaderModule>> = None;

        let lazy_load_shader = |shader: &mut Option<Rc<ShaderModule>>, source: &str| match shader {
            Some(ref s) => s.clone(),
            None => {
                let s = Rc::new(dynamic_renderer.device.create_shader_module(
                    wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
                    },
                ));
                *shader = Some(s.clone());
                s
            }
        };

        let mut bind_group_layouts = vec![
            &scene.model_uniform.bind_group_layout,
            &scene.light_uniform.bind_group_layout,
            &dynamic_renderer.rendered_entity.entity_bind_group_layout,
        ];

        if let Some(rt) = &dynamic_renderer.rendered_texture2d {
            bind_group_layouts.push(&rt.texture2d_bind_group_layout);
        }

        let pipeline_layout =
            dynamic_renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &&bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let mut render_pipelines = HashMap::new();
        for state in renderer_builder.states {
            let (shader, vertex_buf_size, vertex_buf_attr) = match &state.mesh_type {
                MeshType::Entity => (
                    lazy_load_shader(&mut entity_shader, include_str!("shaders/entity.wgsl")),
                    mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    vertex_attr_array![0 => Float32x4, 1 => Float32x3].to_vec(),
                ),
                MeshType::Texture2D => (
                    lazy_load_shader(&mut texture_shader, include_str!("shaders/texture2d.wgsl")),
                    mem::size_of::<Texture2DVertex>() as wgpu::BufferAddress,
                    vertex_attr_array![0 => Float32x4, 1 => Float32x3, 2 => Float32x2].to_vec(),
                ),
            };

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
                                array_stride: vertex_buf_size,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &vertex_buf_attr,
                            }],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: "fs_main",
                            targets: &[Some(config.format.into())],
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: match state.topology {
                                Topology::PointList => PrimitiveTopology::PointList,
                                Topology::LineList => PrimitiveTopology::LineList,
                                Topology::TriangleList => PrimitiveTopology::TriangleList,
                            },
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: Some(wgpu::Face::Back),
                            polygon_mode: match &state.polygon_mode {
                                PolygonMode::Fill => wgpu::PolygonMode::Fill,
                                PolygonMode::Line => wgpu::PolygonMode::Line,
                                PolygonMode::Point => wgpu::PolygonMode::Point,
                            },
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
            render_pipelines.insert(
                EntityRendererState::from_renderer_state(state),
                render_pipeline,
            );
        }

        let mut renderer = Self {
            dynamic_renderer,
            config,
            surface,
            scene,
            render_pipelines,
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
        self.scene
            .update_model(&self.dynamic_renderer.queue, &self.config);

        self.set_depth_texture();
    }

    pub fn update(&mut self, updater: &mut dyn Updater<Event = Event>, ev: Event) {
        updater.update(&mut self.dynamic_renderer, &mut self.scene.style, ev);

        // TODO: Invoke it only when light is changed
        self.scene.update_light(&self.dynamic_renderer.queue);
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
            rpass.set_bind_group(0, &self.scene.model_uniform.bind_group, &[]);
            rpass.set_bind_group(1, &self.scene.light_uniform.bind_group, &[]);

            if let Some(rt) = &self.dynamic_renderer.rendered_texture2d {
                rpass.set_bind_group(3, &rt.texture2d_bind_group, &[]);
            }

            for (i, entity) in rendered_entity.entities.iter().enumerate() {
                rpass.set_pipeline(
                    self.render_pipelines
                        .get(&entity.state)
                        .expect("Specified renderer state is not found"),
                );
                let meta = rendered_entity
                    .meta_list
                    .get(i)
                    .expect("The length of meta_list must match with entities");
                self.prepare_entity(entity, meta);
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

        self.dynamic_renderer.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn prepare_entity(&self, entity: &Entity, meta: &RenderedEntityMeta) {
        let renderer_entity = &self.dynamic_renderer.rendered_entity;
        let buf = EntityUniformBuffer {
            transform: Mat4::from_scale_rotation_translation(
                entity.dimension,
                Quat::from_rotation_x(entity.rotation.x)
                    .mul_quat(Quat::from_rotation_y(entity.rotation.y))
                    .mul_quat(Quat::from_rotation_z(entity.rotation.z)),
                entity.position,
            )
            .to_cols_array_2d(),
            color: rgba_to_array(&entity.fill_color),
        };
        self.dynamic_renderer.queue.write_buffer(
            &renderer_entity.entity_uniform_buf,
            meta.uniform_offset,
            bytemuck::bytes_of(&buf),
        );
    }
}
