use std::{borrow::Cow, collections::HashMap, mem, num::NonZeroU32, rc::Rc};

use glam::Mat4;
use wgpu::{
    vertex_attr_array, BindGroup, Buffer, Device, PrimitiveTopology, RenderPipeline, ShaderModule,
    Texture, TextureView,
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
    scene::CameraUniform,
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
    pub(super) view: TextureView,
}

impl ShadowBaker {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub(super) fn new(
        device: &Device,
        entity_len: usize,
        camera: Mat4,
        shadow_texture: &Texture,
        states: Vec<RendererState>,
    ) -> Self {
        let camera = CameraUniform::with_mat4(device, camera);
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
                            use_lights: vec![],
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
                            use_lights: vec![],
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

        let bake_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow"),
            format: None,
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0_u32,
            array_layer_count: NonZeroU32::new(1),
        });

        Self {
            render_pipelines,
            entity: ShadowEntityUniform {
                entity_uniform_buf,
                entity_bind_group,
            },
            camera,
            view: bake_view,
        }
    }
}
