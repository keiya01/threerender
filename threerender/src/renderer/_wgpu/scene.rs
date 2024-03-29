use std::{f32::consts, mem};

use bytemuck::{Pod, Zeroable};
use glam::{Affine3A, Mat4};
use threerender_math::trs::Translation;
use threerender_traits::entity::ReflectionStyle;
use wgpu::{
    util::DeviceExt, Adapter, BindGroup, BindGroupLayout, Buffer, BufferAddress, Device, Queue,
    Sampler, Texture, TextureView,
};

use crate::{HemisphereLightStyle, LightModel, LightStyle, Scene as AbstractedScene, ShadowStyle};

use super::unit::rgb_to_array;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Reflection {
    intensity: f32,
    specular: f32,
    _padding: [f32; 6],
}

impl Reflection {
    pub(super) fn from_style(style: &ReflectionStyle) -> Self {
        let reflection = style;
        Self {
            intensity: reflection.intensity,
            specular: reflection.specular,
            _padding: [0.; 6],
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
    position: [f32; 3],
    brightness: f32,
    model: u32,

    _padding: [f32; 3],

    hemisphere: HemisphereLight,
    shadow: Shadow,
}

impl Light {
    fn from_light_style(style: &LightStyle) -> Self {
        let color = rgb_to_array(style.base().color());
        Self {
            color: [color[0], color[1], color[2], 1.],
            position: Affine3A::from_rotation_translation(
                style.base().rotation.as_glam(),
                style.base().translation().as_glam(),
            )
            .transform_vector3(style.base().translation().as_glam())
            .to_array(),
            brightness: *style.base().brightness(),
            model: match style.model() {
                LightModel::OFF => 0,
                LightModel::Directional => 1,
                LightModel::Hemisphere => 2,
                LightModel::Ambient => 3,
            },

            _padding: [0., 0., 0.],

            hemisphere: HemisphereLight::from_style(style.hemisphere()),
            shadow: Shadow::from_shadow_style(style),
        }
    }
}

pub(super) struct LightUniform {
    pub(super) buf: Buffer,
    data: Vec<Light>,
    pub(super) bind_group_layout: BindGroupLayout,
    pub(super) bind_group: BindGroup,
}

impl LightUniform {
    fn new(device: &Device, lights: Vec<Light>) -> Self {
        // Create light style uniforms
        let light_uniform_size = (lights.len() * mem::size_of::<Light>()) as wgpu::BufferAddress;
        let light_storage_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Uniform"),
            size: light_uniform_size,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
                resource: light_storage_buf.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            buf: light_storage_buf,
            data: lights,
            bind_group_layout: light_bind_group_layout,
            bind_group: light_bind_group,
        }
    }

    pub(super) fn update(&self, queue: &Queue) {
        for (i, light) in self.data.iter().enumerate() {
            queue.write_buffer(
                &self.buf,
                (i * mem::size_of::<Light>()) as BufferAddress,
                bytemuck::bytes_of(light),
            );
        }
    }

    pub(super) fn len(&self) -> usize {
        self.data.len()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Shadow {
    // The alpha chanel is always ignored. This is to align buffer for wgsl.
    projection: [[f32; 4]; 4],
    use_shadow: u32,
    opacity: f32,

    shadow_type: u32,

    light_uv: f32,
    near_plane: f32,

    _padding: [f32; 3],
}

impl Shadow {
    fn from_shadow_style(light: &LightStyle) -> Self {
        let shadow = light.shadow().as_ref();
        Self {
            projection: shadow
                .map_or_else(|| Mat4::ZERO, |s| s.transform(light))
                .to_cols_array_2d(),
            use_shadow: shadow.is_some() as u32,
            opacity: shadow.map(|s| s.opacity).unwrap_or(1.),
            shadow_type: shadow.map_or(0, |s| s.shadow_type.as_u32()),
            light_uv: shadow.map_or(0., |s| s.fov * consts::PI / 180.),
            near_plane: shadow.map_or(0., |s| s.near),
            _padding: [0., 0., 0.],
        }
    }
}

pub(super) struct ShadowUniform {
    pub(super) bind_group_layout: BindGroupLayout,
    pub(super) bind_group: BindGroup,
    pub(super) texture: Texture,
    pub(super) use_shadow: bool,
}

impl ShadowUniform {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    const DEFAULT_MAX_LIGHT_LENGTH: u32 = 10;

    fn new(device: &Device, use_shadow: bool, map_size: (u32, u32)) -> Self {
        let (sampler, sampler_comparison, texture, view) = Self::create_texture(device, map_size);

        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler_comparison),
                },
            ],
            label: None,
        });

        Self {
            bind_group_layout: shadow_bind_group_layout,
            bind_group: shadow_bind_group,
            texture,
            use_shadow,
        }
    }

    fn create_texture(
        device: &Device,
        map_size: (u32, u32),
    ) -> (Sampler, Sampler, Texture, TextureView) {
        // Create other resources
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        // Create other resources
        let shadow_sampler_comparison = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow sampler comparison"),
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
            view_formats: &[],
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        (
            shadow_sampler,
            shadow_sampler_comparison,
            shadow_texture,
            shadow_view,
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct SceneData {
    pub(super) model: [f32; 16],
    pub(super) eye: [f32; 3],
    pub(super) num_lights: u32,
}

impl SceneData {
    pub(super) fn from_style(style: &AbstractedScene) -> Self {
        Self {
            model: style.camera.transform().to_cols_array(),
            num_lights: style.lights.len() as u32,
            eye: style.camera.calc_position_vec3().as_glam().to_array(),
        }
    }
}

pub(super) struct SceneUniform {
    buf: Buffer,
    pub(super) bind_group_layout: BindGroupLayout,
    pub(super) bind_group: BindGroup,
    pub(super) data: SceneData,
}

impl SceneUniform {
    pub(super) fn new(device: &Device, data: SceneData) -> Self {
        // Create model uniform
        let scene_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Uniform Buffer"),
            contents: bytemuck::bytes_of(&data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(mem::size_of::<SceneData>() as _),
                    },
                    count: None,
                }],
            });
        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniform_buf.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            buf: scene_uniform_buf,
            bind_group_layout: scene_bind_group_layout,
            bind_group: scene_bind_group,
            data,
        }
    }

    pub(super) fn update(&self, queue: &Queue) {
        queue.write_buffer(&self.buf, 0, bytemuck::bytes_of(&self.data));
    }
}

pub(super) struct SceneConfig {
    pub(super) max_samples: u32,
}

pub struct Scene {
    pub(super) scene_uniform: SceneUniform,
    pub(super) light_uniform: LightUniform,
    pub(super) shadow_uniform: ShadowUniform,
    pub(super) forward_depth: Option<TextureView>,
    pub(super) config: SceneConfig,
    pub scene: AbstractedScene,
}

impl Scene {
    pub(super) fn new(
        device: &Device,
        scene: AbstractedScene,
        adapter: &Adapter,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let scene_uniform = SceneUniform::new(device, SceneData::from_style(&scene));
        let mut has_shadow = false;
        let light_data = scene
            .lights
            .iter()
            .map(|light| {
                has_shadow = if has_shadow {
                    has_shadow
                } else {
                    light.shadow().is_some()
                };
                Light::from_light_style(light)
            })
            .collect();
        let light_uniform = LightUniform::new(device, light_data);
        let shadow_uniform = ShadowUniform::new(
            device,
            has_shadow,
            scene
                .shadow_options
                .as_ref()
                .map_or_else(|| ShadowStyle::DEFAULT_MAP_SIZE, |s| *s.map_size()),
        );
        let sample_flags = adapter.get_texture_format_features(config.format).flags;

        Scene {
            scene_uniform,
            light_uniform,
            shadow_uniform,
            forward_depth: None,
            scene,
            config: SceneConfig {
                max_samples: max_samples(&sample_flags),
            },
        }
    }

    pub(super) fn update_scene(&mut self, queue: &Queue) {
        self.scene_uniform.data = SceneData::from_style(&self.scene);
        self.scene_uniform.update(queue);
    }

    pub(super) fn update_light(&mut self, queue: &Queue) {
        self.light_uniform.data = self
            .scene
            .lights
            .iter()
            .map(Light::from_light_style)
            .collect();
        self.light_uniform.update(queue);
    }
}

fn max_samples(sample_flags: &wgpu::TextureFormatFeatureFlags) -> u32 {
    if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X16) {
        16
    } else if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X8) {
        8
    } else if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4) {
        4
    } else if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X2) {
        2
    } else {
        1
    }
}
