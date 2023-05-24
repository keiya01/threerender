use std::{borrow::Cow, collections::HashMap, marker::PhantomData, mem, num::NonZeroU32, rc::Rc};

use threerender_math::Transform;
use threerender_traits::entity::{EntityDescriptor, EntityRendererState};
use wgpu::{
    util::{align_to, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupLayout, Buffer, BufferAddress, Device, Features,
    PrimitiveTopology, Queue, RenderPipeline, Sampler, ShaderModule, Surface, SurfaceConfiguration,
    TextureView, VertexBufferLayout,
};

use crate::{
    entity::{Entity, EntityList},
    mesh::{
        Mesh, MeshType, PolygonMode, TextureFormat, TextureMesh, TextureVertex, Topology, Vertex,
    },
    renderer::Updater,
    utils::vec::count_some,
    RendererBuilder,
};

use super::{
    processor::{ProcessOption, Processor},
    scene::{is_storage_supported, Reflection, Scene},
    shadow::ShadowBaker,
    uniform::EntityUniformBuffer,
    unit::{rgba_to_array, rgba_to_array_64},
};

#[derive(Debug)]
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
    meta_list: Vec<Option<RenderedEntityMeta>>,
    entity_uniform_buf: Buffer,
    entity_bind_group: BindGroup,
    entity_bind_group_layout: BindGroupLayout,
}

impl RenderedEntity {
    fn make_entity(mesh: &Mesh, device: &Device) -> (Buffer, Option<Buffer>, u32) {
        let vertex = match mesh.mesh_type() {
            MeshType::Entity => (Some(mesh.vertex()), None),
            MeshType::Texture => (None, Some(mesh.texture().expect("Texture is not found"))),
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

        let vertex_length = match vertex {
            (Some(v), None) => v.len(),
            (None, Some(v)) => v.len(),
            _ => unreachable!(),
        } as u32;

        (vertex_buf, index_buf, vertex_length)
    }

    pub(super) fn make_uniform(
        device: &Device,
        length: usize,
    ) -> (wgpu::BufferAddress, Buffer, wgpu::BufferAddress) {
        let entity_uniform_size = mem::size_of::<EntityUniformBuffer>() as wgpu::BufferAddress;
        let entity_uniform_alignment = {
            let alignment =
                device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(entity_uniform_size, alignment)
        };
        let entities_length = length as wgpu::BufferAddress;
        let entity_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Transform Uniform Buffer"),
            size: entities_length * entity_uniform_alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        (
            entity_uniform_size,
            entity_uniform_buf,
            entity_uniform_alignment,
        )
    }

    pub(super) fn make_bind_group(
        device: &Device,
        entity_uniform_size: wgpu::BufferAddress,
        entity_uniform_buf: &Buffer,
    ) -> (BindGroupLayout, BindGroup) {
        let entity_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0, // transform
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
                    buffer: entity_uniform_buf,
                    offset: 0,
                    size: wgpu::BufferSize::new(entity_uniform_size),
                }),
            }],
            label: None,
        });

        (entity_bind_group_layout, entity_bind_group)
    }
}

// The struct will be depend on texture.
pub struct RenderedTexture {
    pub(super) texture_view_array: Vec<TextureView>,
    pub(super) sampler_array: Vec<Sampler>,
    texture_bind_group: BindGroup,
    texture_bind_group_layout: BindGroupLayout,

    cur_tex_idx: u32,
}

impl RenderedTexture {
    fn make_texture(
        texture_mesh: &dyn TextureMesh,
        device: &Device,
        queue: &Queue,
    ) -> (Sampler, TextureView) {
        let texture = {
            let buf = texture_mesh.data();
            let width = texture_mesh.width();
            let height = texture_mesh.height();

            let size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Texture mesh texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: match texture_mesh.format() {
                    TextureFormat::Rgba8 => wgpu::TextureFormat::Rgba8Unorm,
                    TextureFormat::Rgba16 => wgpu::TextureFormat::Rgba16Unorm,
                },
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            queue.write_texture(
                texture.as_image_copy(),
                buf,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width * texture_mesh.bytes_per_pixel()),
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
        texture_view_array: &[TextureView],
        sampler_array: &[Sampler],
    ) -> (BindGroupLayout, BindGroup) {
        let texture_bind_group_layout =
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
                        count: NonZeroU32::new(texture_view_array.len() as u32),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: NonZeroU32::new(sampler_array.len() as u32),
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(
                        &texture_view_array.iter().collect::<Vec<_>>(),
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

        (texture_bind_group_layout, texture_bind_group)
    }

    fn create_or_update(
        texture_mesh: &dyn TextureMesh,
        device: &Device,
        queue: &Queue,
        rendered_texture: &mut Option<RenderedTexture>,
    ) {
        let (sampler, view) = Self::make_texture(texture_mesh, device, queue);

        let (mut texture_view_array, mut sampler_array) = (vec![view], vec![sampler]);

        if let Some(rt) = rendered_texture {
            texture_view_array.append(&mut rt.texture_view_array);
            sampler_array.append(&mut rt.sampler_array);
        }

        let (texture_bind_group_layout, texture_bind_group) =
            Self::make_bind_group(device, &texture_view_array, &sampler_array);

        match rendered_texture {
            Some(rt) => {
                rt.texture_view_array = texture_view_array;
                rt.sampler_array = sampler_array;

                rt.texture_bind_group_layout = texture_bind_group_layout;
                rt.texture_bind_group = texture_bind_group;
            }
            None => {
                *rendered_texture = Some(RenderedTexture {
                    texture_view_array,
                    sampler_array,
                    texture_bind_group,
                    texture_bind_group_layout,
                    cur_tex_idx: 0,
                });
            }
        }
    }
}

// This has a role to update the entity in render process dynamically.
pub(super) struct DynamicRenderer {
    pub(super) rendered_entity: RenderedEntity,
    pub(super) rendered_texture: Option<RenderedTexture>,
    pub(super) device: Device,
    pub(super) queue: Queue,
}

impl DynamicRenderer {
    pub fn new(device: Device, queue: Queue, renderer_builder: &mut RendererBuilder) -> Self {
        let entity_length = renderer_builder.mesh_length();
        let (entity_uniform_size, entity_uniform_buf, entity_uniform_alignment) =
            RenderedEntity::make_uniform(&device, entity_length);

        let mut texture_view_array = vec![];
        let mut sampler_array = vec![];
        let mut i = 0;
        let mut tex_idx = 0;
        let (entities, meta_list) = Self::create_recursive_entity(
            &device,
            &queue,
            std::mem::take(&mut renderer_builder.entities),
            (&mut texture_view_array, &mut sampler_array),
            (&mut i, &mut tex_idx),
            entity_uniform_alignment,
        );

        let (entity_bind_group_layout, entity_bind_group) =
            RenderedEntity::make_bind_group(&device, entity_uniform_size, &entity_uniform_buf);

        let rendered_texture = if !texture_view_array.is_empty() && !sampler_array.is_empty() {
            let (texture_bind_group_layout, texture_bind_group) =
                RenderedTexture::make_bind_group(&device, &texture_view_array, &sampler_array);

            Some(RenderedTexture {
                texture_view_array,
                sampler_array,
                texture_bind_group_layout,
                texture_bind_group,
                cur_tex_idx: tex_idx,
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
            },
            rendered_texture,
        }
    }

    fn create_recursive_entity(
        device: &Device,
        queue: &Queue,
        descriptors: Vec<EntityDescriptor>,
        (texture_view_array, sampler_array): (&mut Vec<TextureView>, &mut Vec<Sampler>),
        (idx, tex_idx): (&mut u64, &mut u32),
        entity_uniform_alignment: u64,
    ) -> (Vec<Entity>, Vec<Option<RenderedEntityMeta>>) {
        let mut entities = vec![];
        let mut meta_list = vec![];
        for EntityDescriptor {
            id,
            mesh,
            fill_color,
            transform,
            state,
            children,
            reflection,
        } in descriptors.into_iter()
        {
            let tex_idx_for_entity = match mesh {
                Some(mesh) => {
                    let (vertex_buf, index_buf, vertex_length) =
                        RenderedEntity::make_entity(&mesh, device);

                    let tex_idx_for_entity = if let Mesh::Texture(texture_mesh) = mesh.as_ref() {
                        let (sampler, view) =
                            RenderedTexture::make_texture(texture_mesh.as_ref(), device, queue);

                        texture_view_array.push(view);
                        sampler_array.push(sampler);

                        Some(*tex_idx as i32)
                    } else {
                        None
                    };

                    meta_list.push(Some(RenderedEntityMeta {
                        uniform_offset: *idx * entity_uniform_alignment,
                        vertex_buf,
                        index_buf,
                        vertex_length,
                        index_length: mesh.index().map_or(0, |i| i.len()) as u32,
                    }));

                    // Must update only when mesh is exist
                    *idx += 1;
                    if tex_idx_for_entity.is_some() {
                        *tex_idx += 1;
                    }

                    tex_idx_for_entity
                }
                None => {
                    meta_list.push(None);
                    None
                }
            };

            let (children, mut meta_list2) = Self::create_recursive_entity(
                device,
                queue,
                children,
                (texture_view_array, sampler_array),
                (idx, tex_idx),
                entity_uniform_alignment,
            );

            meta_list.append(&mut meta_list2);

            entities.push(Entity {
                id,
                fill_color,
                transform,
                state,
                reflection,
                children,
                tex_idx: tex_idx_for_entity,
            });
        }

        (entities, meta_list)
    }

    fn update_recursive_entity(
        &mut self,
        descriptors: Vec<EntityDescriptor>,
        idx: &u64,
        entity_uniform_alignment: &u64,
    ) -> (Vec<Entity>, Vec<Option<RenderedEntityMeta>>) {
        let mut idx = *idx;

        let mut entities = vec![];
        let mut meta_list = vec![];
        for EntityDescriptor {
            id,
            mesh,
            fill_color,
            transform,
            state,
            children,
            reflection,
        } in descriptors.into_iter()
        {
            let tex_idx_for_entity = match mesh {
                Some(mesh) => {
                    let (vertex_buf, index_buf, vertex_length) =
                        RenderedEntity::make_entity(&mesh, &self.device);

                    let tex_idx = if let Mesh::Texture(texture_mesh) = mesh.as_ref() {
                        RenderedTexture::create_or_update(
                            texture_mesh.as_ref(),
                            &self.device,
                            &self.queue,
                            &mut self.rendered_texture,
                        );

                        let tex_idx = self.rendered_texture.as_ref().unwrap().cur_tex_idx;
                        self.rendered_texture.as_mut().unwrap().cur_tex_idx += 1;

                        Some(tex_idx as i32)
                    } else {
                        None
                    };
                    meta_list.push(Some(RenderedEntityMeta {
                        uniform_offset: idx * entity_uniform_alignment,
                        vertex_buf,
                        index_buf,
                        vertex_length,
                        index_length: mesh.index().map_or(0, |i| i.len()) as u32,
                    }));
                    // Must update only when mesh is exist
                    idx += 1;

                    tex_idx
                }
                None => {
                    meta_list.push(None);
                    None
                }
            };

            let (children, mut meta_list2) =
                self.update_recursive_entity(children, &idx, entity_uniform_alignment);

            meta_list.append(&mut meta_list2);

            entities.push(Entity {
                id,
                fill_color,
                transform,
                state,
                reflection,
                children,
                tex_idx: tex_idx_for_entity,
            });
        }

        (entities, meta_list)
    }
}

impl EntityList for DynamicRenderer {
    fn items(&self) -> &[Entity] {
        &self.rendered_entity.entities
    }

    fn items_mut(&mut self) -> &mut [Entity] {
        &mut self.rendered_entity.entities
    }

    // FIXME(@kaiye01): Support updating ShadowBaker when entity is added.
    fn push(&mut self, mut descriptor: EntityDescriptor) {
        descriptor.infer_mesh_type();
        let entity_length = count_some(&self.rendered_entity.meta_list);
        let (entity_uniform_size, entity_uniform_buf, entity_uniform_alignment) =
            RenderedEntity::make_uniform(
                &self.device,
                // Length of `Some` of meta_list will be equal with entity mesh length.
                entity_length + descriptor.flatten_mesh_length(),
            );

        let (entity_bind_group_layout, entity_bind_group) =
            RenderedEntity::make_bind_group(&self.device, entity_uniform_size, &entity_uniform_buf);

        let idx = entity_length as u64;
        let (mut entities, mut metas) =
            self.update_recursive_entity(vec![descriptor], &idx, &entity_uniform_alignment);

        self.rendered_entity.entities.append(&mut entities);
        self.rendered_entity.entity_uniform_buf = entity_uniform_buf;
        self.rendered_entity.entity_bind_group_layout = entity_bind_group_layout;
        self.rendered_entity.entity_bind_group = entity_bind_group;
        self.rendered_entity.meta_list.append(&mut metas);
    }
}

// The struct is immutable basically.
pub struct Renderer<Event> {
    pub(super) dynamic_renderer: DynamicRenderer,
    pub(super) config: SurfaceConfiguration,
    pub(super) surface: Surface,
    pub(super) scene: Scene,
    background: [f64; 4],
    render_pipelines: HashMap<EntityRendererState, RenderPipeline>,
    shadow_baker: ShadowBaker,

    phantom_event: PhantomData<Event>,
}

impl<Event> Renderer<Event> {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        window: &W,
        mut renderer_builder: RendererBuilder,
    ) -> Self {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
        let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            dx12_shader_compiler,
        });

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
        let mut limits =
            wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        // TODO: Use constant variable to reduce group.
        limits.max_bind_groups = 5;
        // Create the logical device and command queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: adapter_features | renderer_builder.renderer_specific_attributes.features,
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits,
            },
            None,
        ))
        .expect("Failed to create device");

        let config = surface
            .get_default_config(&adapter, renderer_builder.width, renderer_builder.height)
            .expect("Surface isn't supported by the adapter.");

        surface.configure(&device, &config);

        let scene = Scene::new(&adapter, &device, renderer_builder.scene.take().unwrap());

        let mesh_length = renderer_builder.mesh_length();
        let dynamic_renderer = DynamicRenderer::new(device, queue, &mut renderer_builder);

        // Load the shaders from disk
        let mut entity_shader: Option<Rc<ShaderModule>> = None;
        let mut texture_shader: Option<Rc<ShaderModule>> = None;

        let shader_str = include_str!("shaders/entity.wgsl");
        let mut processor = Processor::new(shader_str);
        let mut tex_processor = Processor::new(shader_str);

        let lazy_load_shader = |shader: &mut Option<Rc<ShaderModule>>,
                                processor: &mut Processor,
                                option: ProcessOption| match shader {
            Some(ref s) => s.clone(),
            None => {
                let source = processor.process(option);
                let s = Rc::new(dynamic_renderer.device.create_shader_module(
                    wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Wgsl(Cow::Owned(source)),
                    },
                ));
                *shader = Some(s.clone());
                s
            }
        };

        let mut bind_group_layouts = vec![
            &scene.scene_uniform.bind_group_layout,
            &dynamic_renderer.rendered_entity.entity_bind_group_layout,
            &scene.light_uniform.bind_group_layout,
            &scene.shadow_uniform.bind_group_layout,
        ];

        if let Some(rt) = &dynamic_renderer.rendered_texture {
            bind_group_layouts.push(&rt.texture_bind_group_layout);
        }

        let pipeline_layout =
            dynamic_renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let support_storage = is_storage_supported(&adapter, &dynamic_renderer.device);

        let mut render_pipelines = HashMap::new();
        let states = renderer_builder.states.clone();
        for state in states {
            let key = EntityRendererState::from_renderer_state(state);
            if render_pipelines.get(&key).is_some() {
                continue;
            }

            let (shader, vertex_buf_size, vertex_buf_attr) =
                match &key.mesh_type.expect("RendererState should have mesh type.") {
                    MeshType::Entity => (
                        lazy_load_shader(
                            &mut entity_shader,
                            &mut processor,
                            ProcessOption {
                                use_texture: false,
                                support_storage,
                                max_light_num: scene.scene.max_light_num,
                            },
                        ),
                        mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        vertex_attr_array![0 => Float32x4, 1 => Float32x3].to_vec(),
                    ),
                    MeshType::Texture if dynamic_renderer.rendered_texture.is_some() => (
                        lazy_load_shader(
                            &mut texture_shader,
                            &mut tex_processor,
                            ProcessOption {
                                use_texture: true,
                                support_storage,
                                max_light_num: scene.scene.max_light_num,
                            },
                        ),
                        mem::size_of::<TextureVertex>() as wgpu::BufferAddress,
                        vertex_attr_array![0 => Float32x4, 1 => Float32x3, 2 => Float32x2].to_vec(),
                    ),
                    _ => continue,
                };

            let render_pipeline =
                dynamic_renderer
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Renderer"),
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
                            topology: match &key.topology {
                                Topology::PointList => PrimitiveTopology::PointList,
                                Topology::LineList => PrimitiveTopology::LineList,
                                Topology::TriangleList => PrimitiveTopology::TriangleList,
                            },
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: Some(wgpu::Face::Back),
                            polygon_mode: match &key.polygon_mode {
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
            render_pipelines.insert(key, render_pipeline);
        }

        let shadow_baker = ShadowBaker::new(
            &adapter,
            &dynamic_renderer.device,
            mesh_length,
            &scene,
            renderer_builder.states,
        );

        let mut renderer = Self {
            dynamic_renderer,
            config,
            surface,
            scene,
            background: rgba_to_array_64(&renderer_builder.background),
            render_pipelines,
            phantom_event: PhantomData,
            shadow_baker,
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
                view_formats: &[],
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
        self.scene.scene.camera.set_width(width as f32);
        self.scene.scene.camera.set_height(height as f32);
        self.scene.update_scene(&self.dynamic_renderer.queue);

        self.set_depth_texture();
    }

    pub fn update(&mut self, updater: &mut dyn Updater<Event = Event>, ev: Event) {
        updater.update(&mut self.dynamic_renderer, &mut self.scene.scene, ev);
        self.update_scene();
    }

    fn update_scene(&mut self) {
        // TODO: Invoke it only when camera is changed
        self.scene.update_scene(&self.dynamic_renderer.queue);
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

        if self.scene.shadow_uniform.use_shadow {
            // shadow pass
            encoder.push_debug_group("shadow pass");
            {
                for (i, light) in self.scene.scene.lights.iter().enumerate() {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.shadow_baker.views[i],
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });

                    // Set shadow projection dynamically
                    {
                        let size = mem::size_of::<[f32; 16]>() as wgpu::BufferAddress;
                        let uniform_alignment = {
                            let alignment = self
                                .dynamic_renderer
                                .device
                                .limits()
                                .min_uniform_buffer_offset_alignment
                                as wgpu::BufferAddress;
                            align_to(size, alignment)
                        };
                        let offset = i as wgpu::BufferAddress * uniform_alignment;

                        let default = Default::default();
                        let shadow = if let Some(shadow) = light.shadow() {
                            shadow
                        } else {
                            &default
                        };
                        self.shadow_baker.camera.update(
                            &self.dynamic_renderer.queue,
                            shadow.transform(light),
                            offset,
                        );

                        rpass.set_bind_group(
                            0,
                            &self.shadow_baker.camera.bind_group,
                            &[offset as u32],
                        );
                    }

                    let mut i = 0;
                    traverse_entities_with_transform(
                        &rendered_entity.entities,
                        &Transform::default(),
                        &mut |entity, transform| {
                            if entity.state.mesh_type.is_none() {
                                i += 1;
                                return;
                            }

                            rpass.set_pipeline(
                                self.shadow_baker
                                    .render_pipelines
                                    .get(&entity.state)
                                    .expect("Specified renderer state is not found"),
                            );

                            let meta = rendered_entity
                                .meta_list
                                .get(i)
                                .expect("The length of meta_list must match with entities");

                            i += 1;

                            if let Some(meta) = meta {
                                self.prepare_shadow_entity(entity, meta, transform);
                                rpass.set_bind_group(
                                    1,
                                    &self.shadow_baker.entity.entity_bind_group,
                                    &[meta.uniform_offset as u32],
                                );

                                rpass.set_vertex_buffer(0, meta.vertex_buf.slice(..));
                                match &meta.index_buf {
                                    Some(index_buf) => {
                                        rpass.set_index_buffer(
                                            index_buf.slice(..),
                                            wgpu::IndexFormat::Uint16,
                                        );
                                        rpass.draw_indexed(0..meta.index_length, 0, 0..1);
                                    }
                                    None => rpass.draw(0..meta.vertex_length, 0..1),
                                }
                            }
                        },
                    );
                }
            }
            encoder.pop_debug_group();
        }

        // forward pass
        encoder.push_debug_group("forward rendering pass");
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.background[0],
                            g: self.background[1],
                            b: self.background[2],
                            a: self.background[3],
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: self.scene.forward_depth.as_ref().map(|view| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: false,
                        }),
                        stencil_ops: None,
                    }
                }),
            });
            rpass.set_bind_group(0, &self.scene.scene_uniform.bind_group, &[]);
            rpass.set_bind_group(2, &self.scene.light_uniform.bind_group, &[]);
            rpass.set_bind_group(3, &self.scene.shadow_uniform.bind_group, &[]);

            let mut i = 0;
            traverse_entities_with_transform(
                &rendered_entity.entities,
                &Transform::default(),
                &mut |entity, transform| {
                    if entity.state.mesh_type.is_none() {
                        i += 1;
                        return;
                    }

                    rpass.set_pipeline(
                        self.render_pipelines
                            .get(&entity.state)
                            .expect("Specified renderer state is not found"),
                    );
                    let meta = rendered_entity
                        .meta_list
                        .get(i)
                        .expect("The length of meta_list must match with entities");

                    i += 1;

                    if let Some(meta) = meta {
                        self.prepare_entity(entity, meta, transform);

                        rpass.set_bind_group(
                            1,
                            &rendered_entity.entity_bind_group,
                            &[meta.uniform_offset as u32],
                        );

                        if let Some(rt) = &self.dynamic_renderer.rendered_texture {
                            rpass.set_bind_group(4, &rt.texture_bind_group, &[]);
                        }

                        rpass.set_vertex_buffer(0, meta.vertex_buf.slice(..));
                        match &meta.index_buf {
                            Some(index_buf) => {
                                rpass.set_index_buffer(
                                    index_buf.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                rpass.draw_indexed(0..meta.index_length, 0, 0..1);
                            }
                            None => rpass.draw(0..meta.vertex_length, 0..1),
                        }
                    }
                },
            );
        }
        encoder.pop_debug_group();

        self.dynamic_renderer.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn prepare_entity(&self, entity: &Entity, meta: &RenderedEntityMeta, transform: &Transform) {
        let renderer_entity = &self.dynamic_renderer.rendered_entity;
        let buf = EntityUniformBuffer {
            transform: transform.as_mat4().to_cols_array_2d(),
            color: rgba_to_array(&entity.fill_color),
            reflection: Reflection::from_style(&entity.reflection),
            tex_idx: entity.tex_idx.unwrap_or(-1),
            padding: [0.; 3],
        };

        self.dynamic_renderer.queue.write_buffer(
            &renderer_entity.entity_uniform_buf,
            meta.uniform_offset,
            bytemuck::bytes_of(&buf),
        );
    }

    fn prepare_shadow_entity(
        &self,
        entity: &Entity,
        meta: &RenderedEntityMeta,
        transform: &Transform,
    ) {
        let renderer_entity = &self.shadow_baker.entity;
        let buf = EntityUniformBuffer {
            transform: transform.as_mat4().to_cols_array_2d(),
            color: rgba_to_array(&entity.fill_color),
            reflection: Reflection::from_style(&entity.reflection),
            tex_idx: entity.tex_idx.unwrap_or(-1),
            padding: [0.; 3],
        };
        self.dynamic_renderer.queue.write_buffer(
            &renderer_entity.entity_uniform_buf,
            meta.uniform_offset,
            bytemuck::bytes_of(&buf),
        );
    }
}

fn traverse_entities_with_transform<F>(entities: &[Entity], transform: &Transform, f: &mut F)
where
    F: FnMut(&Entity, &Transform),
{
    for entity in entities.iter() {
        let transform = transform.mul(&entity.transform);
        f(entity, &transform);
        traverse_entities_with_transform(&entity.children, &transform, f)
    }
}
