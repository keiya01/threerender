use std::mem;

use bytemuck::{Pod, Zeroable};
use glam::{Affine3A, Mat4, Quat, Vec3};
use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue, Sampler, Texture,
    TextureView,
};

use crate::{
    CameraStyle, HemisphereLightStyle, LightModel, LightStyle, ReflectionLightStyle, SceneStyle,
    ShadowStyle,
};

use super::unit::rgb_to_array;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ReflectionLight {
    specular: [f32; 4],
    shininess: f32,
    _padding: [f32; 3],
}

impl ReflectionLight {
    fn from_style(style: &Option<ReflectionLightStyle>) -> Self {
        let reflection = style.clone().unwrap_or_default();
        let specular = rgb_to_array(&reflection.specular);
        Self {
            specular: [specular[0], specular[1], specular[2], 1.],
            shininess: reflection.shininess,
            _padding: [0., 0., 0.],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct HemisphereLight {
    ground_color: [f32; 4],
    sky_color: [f32; 4],
}

impl HemisphereLight {
    fn from_style(style: &Option<HemisphereLightStyle>) -> Self {
        let hemisphere = style.clone().unwrap_or_default();
        let ground = rgb_to_array(&hemisphere.ground_color);
        let sky = rgb_to_array(&hemisphere.sky_color);
        Self {
            ground_color: [ground[0], ground[1], ground[2], 1.],
            sky_color: [sky[0], sky[1], sky[2], 1.],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Light {
    // The alpha chanel is always ignored. This is to align buffer for wgsl.
    color: [f32; 4],
    ambient: [f32; 4],
    position: [f32; 3],
    brightness: f32,
    model: u32,

    _padding: [f32; 3],

    reflection: ReflectionLight,
    hemisphere: HemisphereLight,
}

impl Light {
    fn from_light_style(style: &LightStyle) -> Self {
        let color = rgb_to_array(&style.base.color);
        let ambient = rgb_to_array(&style.base.ambient);
        Self {
            color: [color[0], color[1], color[2], 1.],
            ambient: [ambient[0], ambient[1], ambient[2], 1.],
            position: Affine3A::from_rotation_translation(
                Quat::from_rotation_x(style.base.rotation.x)
                    .mul_quat(Quat::from_rotation_y(style.base.rotation.y))
                    .mul_quat(Quat::from_rotation_z(style.base.rotation.z)),
                style.base.position,
            )
            .transform_vector3(style.base.position)
            .to_array(),
            brightness: style.base.brightness,
            model: match style.model {
                LightModel::OFF => 0,
                LightModel::Directional => 1,
                LightModel::Hemisphere => 2,
            },

            _padding: [0., 0., 0.],

            reflection: ReflectionLight::from_style(&style.reflection),
            hemisphere: HemisphereLight::from_style(&style.hemisphere),

        }
    }
}

pub(super) struct LightUniform {
    buf: Buffer,
    data: Light,
    pub(super) bind_group_layout: BindGroupLayout,
    pub(super) bind_group: BindGroup,
}

impl LightUniform {
    fn new(device: &Device, light: Light) -> Self {
        // Create light style uniforms
        let light_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Uniform Buffer"),
            contents: bytemuck::bytes_of(&light),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(mem::size_of::<Light>() as _),
                    },
                    count: None,
                }],
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_uniform_buf.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            buf: light_uniform_buf,
            data: light,
            bind_group_layout: light_bind_group_layout,
            bind_group: light_bind_group,
        }
    }

    pub(super) fn update(&self, queue: &Queue, light: &Light) {
        queue.write_buffer(&self.buf, 0, bytemuck::bytes_of(light));
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Shadow {
    // The alpha chanel is always ignored. This is to align buffer for wgsl.
    projection: [[f32; 4]; 4],
    use_shadow: u32,

    _padding: [f32; 4],
}

impl Shadow {
    fn from_shadow_style(style: &Option<ShadowStyle>, light: &LightStyle) -> Self {
        Self {
            projection: style
                .as_ref()
                .map_or_else(|| Mat4::ZERO, |s| s.transform(light))
                .to_cols_array_2d(),
            use_shadow: style.is_some() as u32,

            _padding: [0., 0., 0., 0.],
        }
    }
}

pub(super) struct ShadowUniform {
    buf: Buffer,
    data: Shadow,
    pub(super) bind_group_layout: BindGroupLayout,
    pub(super) bind_group: BindGroup,
    pub(super) texture: Texture,
    pub(super) use_shadow: bool,
}

impl ShadowUniform {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    const DEFAULT_MAX_LIGHT_LENGTH: u32 = 10;

    fn new(device: &Device, shadow: Shadow, use_shadow: bool, map_size: (u32, u32)) -> Self {
        // Create shadow style uniforms
        let shadow_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shadow Uniform Buffer"),
            contents: bytemuck::bytes_of(&shadow),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let (sampler, texture, view) = Self::create_texture(device, map_size);

        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(mem::size_of::<Shadow>() as _),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            });

        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shadow_uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        Self {
            buf: shadow_uniform_buf,
            data: shadow,
            bind_group_layout: shadow_bind_group_layout,
            bind_group: shadow_bind_group,
            texture,
            use_shadow,
        }
    }

    fn create_texture(device: &Device, map_size: (u32, u32)) -> (Sampler, Texture, TextureView) {
        // Create other resources
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: map_size.0,
                height: map_size.1,
                depth_or_array_layers: Self::DEFAULT_MAX_LIGHT_LENGTH,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        (shadow_sampler, shadow_texture, shadow_view)
    }

    pub(super) fn update(&self, queue: &Queue, shadow: &Shadow) {
        queue.write_buffer(&self.buf, 0, bytemuck::bytes_of(shadow));
    }
}

pub(super) struct CameraUniform {
    buf: Buffer,
    pub(super) bind_group_layout: BindGroupLayout,
    pub(super) bind_group: BindGroup,
    pub(super) model: Mat4,
}

impl CameraUniform {
    fn new(device: &Device, camera: &CameraStyle) -> Self {
        // Create model uniform
        let model = camera.transform();
        Self::with_mat4(device, model)
    }

    pub(super) fn with_mat4(device: &Device, model: Mat4) -> Self {
        let model_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Uniform Buffer"),
            contents: bytemuck::bytes_of(model.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let model_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0, // model
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(mem::size_of::<Mat4>() as _),
                    },
                    count: None,
                }],
            });
        let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_uniform_buf.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            buf: model_uniform_buf,
            bind_group_layout: model_bind_group_layout,
            bind_group: model_bind_group,
            model,
        }
    }

    pub(super) fn update(&self, queue: &Queue, camera: &CameraStyle) {
        let model = camera.transform();
        let mx_ref: &[f32; 16] = model.as_ref();
        queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(mx_ref));
    }
}

pub struct Scene {
    pub(super) camera_uniform: CameraUniform,
    pub(super) light_uniform: LightUniform,
    pub(super) shadow_uniform: ShadowUniform,
    pub(super) forward_depth: Option<TextureView>,
    pub style: SceneStyle,
}

impl Scene {
    pub(super) fn new(device: &Device, scene_style: SceneStyle) -> Self {
        let camera_uniform = CameraUniform::new(device, &scene_style.camera);
        let light_uniform = LightUniform::new(device, Light::from_light_style(&scene_style.light));
        let shadow_uniform = ShadowUniform::new(
            device,
            Shadow::from_shadow_style(&scene_style.shadow, &scene_style.light),
            scene_style.shadow.is_some(),
            scene_style
                .shadow
                .as_ref()
                .map_or_else(|| ShadowStyle::DEFAULT_MAP_SIZE, |s| s.map_size),
        );

        Scene {
            camera_uniform,
            light_uniform,
            shadow_uniform,
            forward_depth: None,
            style: scene_style,
        }
    }

    pub(super) fn update_camera(&self, queue: &Queue) {
        self.camera_uniform.update(queue, &self.style.camera);
    }

    pub(super) fn update_light(&mut self, queue: &Queue) {
        self.light_uniform.data = Light::from_light_style(&self.style.light);
        self.light_uniform.update(queue, &self.light_uniform.data);
    }

    pub(super) fn update_shadow(&mut self, queue: &Queue) {
        self.shadow_uniform.data = Shadow::from_shadow_style(&self.style.shadow, &self.style.light);
        self.shadow_uniform.update(queue, &self.shadow_uniform.data);
    }
}
