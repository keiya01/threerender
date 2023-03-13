use std::{borrow::Cow, collections::HashMap, mem, rc::Rc};

use glam::Mat4;
use wgpu::{
    util::align_to, vertex_attr_array, Adapter, BindGroup, BindGroupLayout, Buffer, Device,
    PrimitiveTopology, Queue, RenderPipeline, ShaderModule, TextureView,
};

use crate::{
    entity::EntityRendererState,
    mesh::{
        util::{TextureVertex, Vertex},
        MeshType, PolygonMode, Topology,
    },
    RendererState,
};

use super::{
    processor::{process_shader, ShaderProcessOption},
    scene::{is_storage_supported, Scene},
    RenderedEntity,
};

pub(super) struct ShadowEntityUniform {
    pub(super) entity_uniform_buf: Buffer,
    pub(super) entity_bind_group: BindGroup,
}

pub(super) struct ShadowBaker {
    pub(super) render_pipelines: HashMap<EntityRendererState, RenderPipeline>,
    pub(super) entity: ShadowEntityUniform,
    pub(super) camera: CameraUniform,
    pub(super) views: Vec<TextureView>,
}

impl ShadowBaker {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub(super) fn new(
        adapter: &Adapter,
        device: &Device,
        entity_len: usize,
        scene: &Scene,
        states: Vec<RendererState>,
    ) -> Self {
        let camera = CameraUniform::with_mat4(device, scene.light_uniform.len());
        let (entity_uniform_size, entity_uniform_buf, _) =
            RenderedEntity::make_uniform(device, entity_len);
        let (entity_bind_group_layout, entity_bind_group) =
            RenderedEntity::make_bind_group(device, entity_uniform_size, &entity_uniform_buf);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow"),
            bind_group_layouts: &[&camera.bind_group_layout, &entity_bind_group_layout],
            push_constant_ranges: &[],
        });

        let lazy_load_shader =
            |shader: &mut Option<Rc<ShaderModule>>, option: ShaderProcessOption| match shader {
                Some(ref s) => s.clone(),
                None => {
                    // TODO: Cache source
                    let source = process_shader(include_str!("shaders/shadow.wgsl"), option);
                    let s = Rc::new(device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Wgsl(Cow::Owned(source)),
                    }));
                    *shader = Some(s.clone());
                    s
                }
            };

        let support_storage = is_storage_supported(adapter, device);

        // Load the shaders from disk
        let mut entity_shader: Option<Rc<ShaderModule>> = None;
        let mut texture_shader: Option<Rc<ShaderModule>> = None;

        let mut render_pipelines = HashMap::new();

        // TODO: commonize with renderer
        for state in states {
            let key = EntityRendererState::from_renderer_state(state);
            if render_pipelines.get(&key).is_some() {
                continue;
            }

            let (shader, vertex_buf_size, vertex_buf_attr) = match &key.mesh_type {
                MeshType::Entity => (
                    lazy_load_shader(
                        &mut entity_shader,
                        ShaderProcessOption {
                            use_texture: false,
                            support_storage,
                        },
                    ),
                    mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    vertex_attr_array![0 => Float32x4, 1 => Float32x3].to_vec(),
                ),
                MeshType::Texture => (
                    lazy_load_shader(
                        &mut texture_shader,
                        ShaderProcessOption {
                            use_texture: true,
                            support_storage,
                        },
                    ),
                    mem::size_of::<TextureVertex>() as wgpu::BufferAddress,
                    vertex_attr_array![0 => Float32x4, 1 => Float32x3, 2 => Float32x2].to_vec(),
                ),
            };

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ShadowBaker"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_bake",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: vertex_buf_size,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_buf_attr,
                    }],
                },
                fragment: None,
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
                    unclipped_depth: device
                        .features()
                        .contains(wgpu::Features::DEPTH_CLIP_CONTROL),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Self::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState {
                        constant: 2, // corresponds to bilinear filtering
                        slope_scale: 2.0,
                        clamp: 0.0,
                    },
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

            render_pipelines.insert(key, pipeline);
        }

        let mut views = vec![];

        for i in 0..scene.light_uniform.len() {
            let bake_view =
                scene
                    .shadow_uniform
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        label: Some("shadow"),
                        format: None,
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: i as u32,
                        array_layer_count: Some(1),
                    });
            views.push(bake_view);
        }

        Self {
            render_pipelines,
            entity: ShadowEntityUniform {
                entity_uniform_buf,
                entity_bind_group,
            },
            camera,
            views,
        }
    }
}

pub(super) struct CameraUniform {
    pub(super) buf: Buffer,
    pub(super) bind_group_layout: BindGroupLayout,
    pub(super) bind_group: BindGroup,
}

impl CameraUniform {
    pub(super) fn with_mat4(device: &Device, light_length: usize) -> Self {
        let camera_uniform_size = mem::size_of::<[f32; 16]>() as wgpu::BufferAddress;
        let camera_uniform_alignment = {
            let alignment =
                device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(camera_uniform_size, alignment)
        };
        let camera_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: light_length as wgpu::BufferAddress * camera_uniform_alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0, // camera
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(camera_uniform_size),
                    },
                    count: None,
                }],
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_uniform_buf,
                    offset: 0,
                    size: wgpu::BufferSize::new(camera_uniform_size),
                }),
            }],
            label: None,
        });

        Self {
            buf: camera_uniform_buf,
            bind_group_layout: camera_bind_group_layout,
            bind_group: camera_bind_group,
        }
    }

    pub(super) fn update(&self, queue: &Queue, model: Mat4, offset: wgpu::BufferAddress) {
        let mx_ref: &[f32; 16] = model.as_ref();
        queue.write_buffer(&self.buf, offset, bytemuck::cast_slice(mx_ref));
    }
}
