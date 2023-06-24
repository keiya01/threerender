use std::{borrow::Cow, collections::HashMap, io::Write, mem, num::NonZeroU32, rc::Rc};

use glam::Mat3;
use threerender_math::Transform;
use threerender_traits::{
    entity::{EntityDescriptor, EntityRendererState},
    image::Image,
};
use wgpu::{
    util::{align_to, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupLayout, Buffer, BufferAddress, Device, Features,
    PrimitiveTopology, Queue, RenderPipeline, Sampler, ShaderModule, Surface, SurfaceConfiguration,
    Texture, TextureView, VertexBufferLayout,
};

use crate::{
    entity::Entity,
    mesh::{PolygonMode, TextureFormat, Topology, Vertex},
    utils::vec::count_some,
    RendererBuilder,
};

use super::{
    processor::{ProcessOption, Processor},
    scene::{Reflection, Scene},
    shadow::ShadowBaker,
    uniform::{EntityUniformBuffer, ShadowEntityUniformBuffer},
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
    fn make_entity(
        vertex: &[Vertex],
        index: Option<&[u16]>,
        device: &Device,
    ) -> (Buffer, Option<Buffer>, u32) {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertex),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = index.map(|index| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(index),
                usage: wgpu::BufferUsages::INDEX,
            })
        });

        let vertex_length = vertex.len() as u32;

        (vertex_buf, index_buf, vertex_length)
    }

    pub(super) fn make_uniform(
        device: &Device,
        length: usize,
        entity_uniform_size: wgpu::BufferAddress,
    ) -> (wgpu::BufferAddress, Buffer, wgpu::BufferAddress) {
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
    texture_bind_group: Option<BindGroup>,
    texture_bind_group_layout: Option<BindGroupLayout>,

    cur_tex_idx: u32,
}

impl RenderedTexture {
    fn make_texture(image: &dyn Image, device: &Device, queue: &Queue) -> (Sampler, TextureView) {
        let texture = {
            let buf = image.data();
            let width = image.width();
            let height = image.height();

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
                format: match image.format() {
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
                    bytes_per_row: Some(width * image.bytes_per_pixel()),
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

    fn update(
        image: &dyn Image,
        device: &Device,
        queue: &Queue,
        rendered_texture: &mut RenderedTexture,
    ) {
        let (sampler, view) = Self::make_texture(image, device, queue);

        let (mut texture_view_array, mut sampler_array) = (vec![view], vec![sampler]);

        texture_view_array.append(&mut rendered_texture.texture_view_array);
        sampler_array.append(&mut rendered_texture.sampler_array);

        let (texture_bind_group_layout, texture_bind_group) =
            if !texture_view_array.is_empty() && !sampler_array.is_empty() {
                let (a, b) = Self::make_bind_group(device, &texture_view_array, &sampler_array);
                (Some(a), Some(b))
            } else {
                (None, None)
            };

        rendered_texture.texture_view_array = texture_view_array;
        rendered_texture.sampler_array = sampler_array;

        rendered_texture.texture_bind_group_layout = texture_bind_group_layout;
        rendered_texture.texture_bind_group = texture_bind_group;
    }
}

// This has a role to update the entity in render process dynamically.
pub(super) struct DynamicRenderer {
    pub(super) rendered_entity: RenderedEntity,
    pub(super) rendered_texture: RenderedTexture,
    pub(super) device: Device,
    pub(super) queue: Queue,
}

impl DynamicRenderer {
    pub fn new(device: Device, queue: Queue, renderer_builder: &mut RendererBuilder) -> Self {
        let entity_length = renderer_builder.mesh_length();
        let (entity_uniform_size, entity_uniform_buf, entity_uniform_alignment) =
            RenderedEntity::make_uniform(
                &device,
                entity_length,
                mem::size_of::<EntityUniformBuffer>() as wgpu::BufferAddress,
            );

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

        let (texture_bind_group_layout, texture_bind_group) =
            if !texture_view_array.is_empty() && !sampler_array.is_empty() {
                let (a, b) =
                    RenderedTexture::make_bind_group(&device, &texture_view_array, &sampler_array);
                (Some(a), Some(b))
            } else {
                (None, None)
            };

        let rendered_texture = RenderedTexture {
            texture_view_array,
            sampler_array,
            texture_bind_group_layout,
            texture_bind_group,
            cur_tex_idx: tex_idx,
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
            texture,
            normal_map,
            receive_shadow,
        } in descriptors.into_iter()
        {
            let (tex_idx_for_entity, normal_map_idx) = match mesh {
                Some(mesh) => {
                    let vertex = match normal_map {
                        Some(_) => mesh.as_ref().as_tangent_space(),
                        None => mesh.as_ref().vertex(),
                    };
                    let (vertex_buf, index_buf, vertex_length) = RenderedEntity::make_entity(
                        vertex.borrow().as_slice(),
                        mesh.as_ref().index(),
                        device,
                    );

                    let tex_idx_for_entity = if let Some(texture) = &texture {
                        let (sampler, view) =
                            RenderedTexture::make_texture(texture.as_ref(), device, queue);

                        texture_view_array.push(view);
                        sampler_array.push(sampler);

                        let idx = *tex_idx as i32;

                        *tex_idx += 1;

                        Some(idx)
                    } else {
                        None
                    };

                    let normal_map_idx = if let Some(normal_map) = &normal_map {
                        let (sampler, view) =
                            RenderedTexture::make_texture(normal_map.as_ref(), device, queue);

                        texture_view_array.push(view);
                        sampler_array.push(sampler);

                        let idx: i32 = *tex_idx as i32;

                        *tex_idx += 1;

                        Some(idx)
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

                    (tex_idx_for_entity, normal_map_idx)
                }
                None => {
                    meta_list.push(None);
                    (None, None)
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

            // Storing all texture(includes the map) into single texture array
            // and access by using the index.
            entities.push(Entity {
                id,
                fill_color,
                transform,
                state,
                reflection,
                children,
                tex_idx: tex_idx_for_entity,
                normal_map_idx,
                receive_shadow,
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
            texture,
            normal_map,
            receive_shadow,
        } in descriptors.into_iter()
        {
            let (tex_idx_for_entity, normal_map_idx) = match mesh {
                Some(mesh) => {
                    let vertex = match normal_map {
                        Some(_) => mesh.as_ref().as_tangent_space(),
                        None => mesh.as_ref().vertex(),
                    };
                    let (vertex_buf, index_buf, vertex_length) = RenderedEntity::make_entity(
                        vertex.borrow().as_slice(),
                        mesh.as_ref().index(),
                        &self.device,
                    );

                    let tex_idx = if let Some(texture) = &texture {
                        RenderedTexture::update(
                            texture.as_ref(),
                            &self.device,
                            &self.queue,
                            &mut self.rendered_texture,
                        );

                        let tex_idx = self.rendered_texture.cur_tex_idx;
                        self.rendered_texture.cur_tex_idx += 1;

                        Some(tex_idx as i32)
                    } else {
                        None
                    };

                    let normal_map_idx = if let Some(normal_map) = &normal_map {
                        // FIXME(@keiya01): Handle image error
                        RenderedTexture::update(
                            normal_map.as_ref(),
                            &self.device,
                            &self.queue,
                            &mut self.rendered_texture,
                        );

                        let tex_idx = self.rendered_texture.cur_tex_idx;
                        self.rendered_texture.cur_tex_idx += 1;

                        Some((tex_idx + 1) as i32)
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

                    (tex_idx, normal_map_idx)
                }
                None => {
                    meta_list.push(None);
                    (None, None)
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
                normal_map_idx,
                receive_shadow,
            });
        }

        (entities, meta_list)
    }
}

// The struct is immutable basically.
pub struct Renderer {
    pub(super) dynamic_renderer: DynamicRenderer,
    pub(super) config: SurfaceConfiguration,
    pub(super) surface: Option<Surface>,
    pub(super) scene: Scene,
    background: [f64; 4],
    render_pipelines: HashMap<EntityRendererState, RenderPipeline>,
    shadow_baker: ShadowBaker,

    dst_texture: Option<Texture>,
}

// Accessible properties
impl Renderer {
    pub fn entities(&self) -> &[Entity] {
        &self.dynamic_renderer.rendered_entity.entities
    }

    pub fn entities_mut(&mut self) -> &mut [Entity] {
        &mut self.dynamic_renderer.rendered_entity.entities
    }

    // FIXME(@kaiye01): Support updating ShadowBaker when entity is added.
    pub fn push_entity(&mut self, descriptor: EntityDescriptor) {
        let entity_length = count_some(&self.dynamic_renderer.rendered_entity.meta_list);
        let (entity_uniform_size, entity_uniform_buf, entity_uniform_alignment) =
            RenderedEntity::make_uniform(
                &self.dynamic_renderer.device,
                // Length of `Some` of meta_list will be equal with entity mesh length.
                entity_length + descriptor.flatten_mesh_length(),
                mem::size_of::<EntityUniformBuffer>() as wgpu::BufferAddress,
            );

        let (entity_bind_group_layout, entity_bind_group) = RenderedEntity::make_bind_group(
            &self.dynamic_renderer.device,
            entity_uniform_size,
            &entity_uniform_buf,
        );

        let idx = entity_length as u64;
        let (mut entities, mut metas) = self.dynamic_renderer.update_recursive_entity(
            vec![descriptor],
            &idx,
            &entity_uniform_alignment,
        );

        self.dynamic_renderer
            .rendered_entity
            .entities
            .append(&mut entities);
        self.dynamic_renderer.rendered_entity.entity_uniform_buf = entity_uniform_buf;
        self.dynamic_renderer
            .rendered_entity
            .entity_bind_group_layout = entity_bind_group_layout;
        self.dynamic_renderer.rendered_entity.entity_bind_group = entity_bind_group;
        self.dynamic_renderer
            .rendered_entity
            .meta_list
            .append(&mut metas);
    }

    pub fn scene(&self) -> &crate::scene::Scene {
        &self.scene.scene
    }

    pub fn scene_mut(&mut self) -> &mut crate::scene::Scene {
        &mut self.scene.scene
    }
}

// Render processes
impl Renderer {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub async fn new<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        mut renderer_builder: RendererBuilder,
        window: Option<&W>,
    ) -> Self {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
        let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            dx12_shader_compiler,
        });

        let surface = window.map(|w| unsafe { instance.create_surface(w) }.unwrap());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: surface.as_ref(),
            })
            .await
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
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: adapter_features
                        | renderer_builder.renderer_specific_attributes.features,
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let config = if let Some(surface) = &surface {
            surface
                .get_default_config(&adapter, renderer_builder.width, renderer_builder.height)
                .expect("Surface isn't supported by the adapter.")
        } else {
            SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                width: renderer_builder.width,
                height: renderer_builder.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![wgpu::TextureFormat::Rgba8Unorm],
            }
        };

        if let Some(surface) = &surface {
            surface.configure(&device, &config);
        }

        let scene = Scene::new(
            &device,
            renderer_builder.scene.take().unwrap(),
            &adapter,
            &config,
        );

        let mesh_length = renderer_builder.mesh_length();
        let dynamic_renderer = DynamicRenderer::new(device, queue, &mut renderer_builder);

        // Load the shaders from disk
        let mut shaders: Option<Rc<ShaderModule>> = None;

        let shader_str = include_str!("shaders/entity.wgsl");
        let mut processor = Processor::new(shader_str);

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

        if let Some(layout) = &dynamic_renderer.rendered_texture.texture_bind_group_layout {
            bind_group_layouts.push(layout);
        }

        let pipeline_layout =
            dynamic_renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let has_tex = dynamic_renderer
            .rendered_texture
            .texture_bind_group_layout
            .is_some();

        let mut render_pipelines = HashMap::new();
        let states = renderer_builder.states.clone();
        for state in states {
            let key = EntityRendererState::from_renderer_state(state);
            if render_pipelines.get(&key).is_some() {
                continue;
            }

            let shader = lazy_load_shader(
                &mut shaders,
                &mut processor,
                ProcessOption {
                    has_texture: has_tex,
                    max_light_num: scene.scene.max_light_num,
                },
            );

            let (vertex_buf_size, vertex_buf_attr) = (mem::size_of::<Vertex>() as wgpu::BufferAddress, vertex_attr_array![0 => Float32x4, 1 => Float32x3, 2 => Float32x2, 3 => Float32x3, 4 => Float32x3].to_vec());

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
                        multisample: wgpu::MultisampleState {
                            count: scene.config.max_samples.min(scene.scene.msaa_samples),
                            ..Default::default()
                        },
                        multiview: None,
                    });
            render_pipelines.insert(key, render_pipeline);
        }

        let shadow_baker = ShadowBaker::new(
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
            shadow_baker,
            dst_texture: None,
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
                sample_count: self
                    .scene
                    .config
                    .max_samples
                    .min(self.scene.scene.msaa_samples),
                dimension: wgpu::TextureDimension::D2,
                format: Self::DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
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
        if let Some(s) = self.surface.as_ref() {
            s.configure(&self.dynamic_renderer.device, &self.config)
        }
        self.scene.scene.camera.set_width(width as f32);
        self.scene.scene.camera.set_height(height as f32);
        self.scene.update_scene(&self.dynamic_renderer.queue);

        self.set_depth_texture();
    }

    fn update_scene(&mut self) {
        // TODO: Invoke it only when camera is changed
        self.scene.update_scene(&self.dynamic_renderer.queue);
        // TODO: Invoke it only when light is changed
        self.scene.update_light(&self.dynamic_renderer.queue);
    }

    fn render_actual(&mut self, view: TextureView) {
        self.update_scene();

        let rendered_entity = &self.dynamic_renderer.rendered_entity;

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
                                self.prepare_shadow_entity(meta, transform);
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

        let msaa_samples = self
            .scene
            .config
            .max_samples
            .min(self.scene.scene.msaa_samples);

        let (view, resolve_target, store) = if msaa_samples <= 1 {
            (view, None, true)
        } else {
            let texture = self
                .dynamic_renderer
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    size: wgpu::Extent3d {
                        width: self.config.width,
                        height: self.config.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: msaa_samples,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.config.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    label: None,
                    view_formats: &[],
                });
            let multi_sampled_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            (multi_sampled_view, Some(&view), false)
        };

        // forward pass
        encoder.push_debug_group("forward rendering pass");
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.background[0],
                            g: self.background[1],
                            b: self.background[2],
                            a: self.background[3],
                        }),
                        store,
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

                        if let Some(bind_group) =
                            &self.dynamic_renderer.rendered_texture.texture_bind_group
                        {
                            rpass.set_bind_group(4, bind_group, &[]);
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
    }

    pub fn render(&mut self) {
        let (view, frame) = if let Some(surface) = &self.surface {
            let frame = surface.get_current_texture().unwrap();
            (
                frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                Some(frame),
            )
        } else {
            let dst_texture =
                self.dynamic_renderer
                    .device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some("destination"),
                        size: wgpu::Extent3d {
                            width: self.config.width,
                            height: self.config.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::COPY_SRC,
                        view_formats: &[],
                    });

            self.dst_texture = Some(dst_texture);

            (
                self.dst_texture
                    .as_ref()
                    .unwrap()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                None,
            )
        };

        self.render_actual(view);

        if let Some(frame) = frame {
            frame.present();
        }
    }

    pub fn load_as_image(&mut self) -> Vec<u8> {
        if self.surface.is_some() {
            panic!("You already have a window as render target view.");
        }

        let dst_texture = match &self.dst_texture {
            Some(txt) => txt,
            None => return vec![],
        };

        // Need to handle bytes per row due to wgpu restriction
        let bytes_per_pixel = std::mem::size_of::<u32>() as u32;
        let unpadded_bytes_per_row = self.config.width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;

        let dst_buffer = self
            .dynamic_renderer
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("image map buffer"),
                size: padded_bytes_per_row as u64 * self.config.height as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });

        let mut cmd_buf = self
            .dynamic_renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        cmd_buf.copy_texture_to_buffer(
            dst_texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &dst_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
        );

        self.dynamic_renderer.queue.submit(Some(cmd_buf.finish()));

        let dst_buffer_slice = dst_buffer.slice(..);
        dst_buffer_slice.map_async(wgpu::MapMode::Read, |_| ());
        self.dynamic_renderer.device.poll(wgpu::Maintain::Wait);
        let buf = dst_buffer_slice.get_mapped_range().to_vec();

        let mut result = vec![];
        for chunk in buf.chunks(padded_bytes_per_row as usize) {
            result
                .write_all(&chunk[..(unpadded_bytes_per_row as usize)])
                .unwrap();
        }
        result
    }

    // FIXME(@keiya01): Dirty check
    fn prepare_entity(&self, entity: &Entity, meta: &RenderedEntityMeta, transform: &Transform) {
        let renderer_entity = &self.dynamic_renderer.rendered_entity;
        let transform = transform.as_mat4();
        let normal_transform = Mat3::from_mat4(transform)
            .inverse()
            .transpose()
            .to_cols_array_2d();
        let buf = EntityUniformBuffer {
            transform: transform.to_cols_array_2d(),
            color: rgba_to_array(&entity.fill_color),
            reflection: Reflection::from_style(&entity.reflection),
            tex_idx: [entity.tex_idx.unwrap_or(-1), 0, 0, 0],
            normal_idx: [entity.normal_map_idx.unwrap_or(-1), 0, 0, 0],
            #[rustfmt::skip]
            normal_transform: [
                [normal_transform[0][0], normal_transform[0][1], normal_transform[0][2], 0.],
                [normal_transform[1][0], normal_transform[1][1], normal_transform[1][2], 0.],
                [normal_transform[2][0], normal_transform[2][1], normal_transform[2][2], 0.],
                [0., 0., 0., 0.],
            ],
            receive_shadow: [entity.receive_shadow as u32, 0, 0, 0],
        };

        self.dynamic_renderer.queue.write_buffer(
            &renderer_entity.entity_uniform_buf,
            meta.uniform_offset,
            bytemuck::bytes_of(&buf),
        );
    }

    // FIXME(@keiya01): Dirty check
    fn prepare_shadow_entity(&self, meta: &RenderedEntityMeta, transform: &Transform) {
        let renderer_entity = &self.shadow_baker.entity;
        let transform = transform.as_mat4();
        let buf = ShadowEntityUniformBuffer {
            transform: transform.to_cols_array_2d(),
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
