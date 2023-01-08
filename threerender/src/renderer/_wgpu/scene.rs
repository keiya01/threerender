use std::{f32::consts, mem};

use bytemuck::{Pod, Zeroable};
use glam::{Affine3A, Mat4, Quat, Vec3};
use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue, SurfaceConfiguration,
    TextureView,
};

use crate::{LightModel, LightStyle, SceneStyle};

use super::unit::rgb_to_array;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Light {
    // The alpha chanel is always ignored. This is to align buffer for wgsl.
    color: [f32; 4],
    position: [f32; 3],
    brightness: f32,
    model: i32,

    _padding: [f32; 4],
}

impl Light {
    fn from_light_style(style: &LightStyle) -> Self {
        let arr3 = rgb_to_array(&style.color);
        Self {
            color: [arr3[0], arr3[1], arr3[2], 1.],
            position: Affine3A::from_rotation_translation(
                Quat::from_rotation_x(style.rotation.x)
                    .mul_quat(Quat::from_rotation_y(style.rotation.y))
                    .mul_quat(Quat::from_rotation_z(style.rotation.z)),
                style.position,
            )
            .transform_vector3(Vec3::ONE)
            .to_array(),
            brightness: style.brightness,
            model: match style.model {
                LightModel::OFF => 0,
                LightModel::DiffuseReflection => 1,
            },

            _padding: [0., 0., 0., 0.],
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
                    visibility: wgpu::ShaderStages::VERTEX,
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

pub(super) struct ModelUniform {
    buf: Buffer,
    pub(super) bind_group_layout: BindGroupLayout,
    pub(super) bind_group: BindGroup,
}

impl ModelUniform {
    fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        // Create model uniform
        let model = transform((config.width / config.height) as f32);
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
                    visibility: wgpu::ShaderStages::VERTEX,
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
        }
    }

    pub(super) fn update(&self, queue: &Queue, config: &SurfaceConfiguration) {
        let model = transform(config.width as f32 / config.height as f32);
        let mx_ref: &[f32; 16] = model.as_ref();
        queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(mx_ref));
    }
}

pub struct Scene {
    pub(super) model_uniform: ModelUniform,
    pub(super) light_uniform: LightUniform,
    pub(super) forward_depth: Option<TextureView>,
    pub style: SceneStyle,
}

impl Scene {
    pub(super) fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        scene_style: SceneStyle,
    ) -> Self {
        let model_uniform = ModelUniform::new(device, config);
        let light_uniform = LightUniform::new(device, Light::from_light_style(&scene_style.light));

        Scene {
            model_uniform,
            light_uniform,
            forward_depth: None,
            style: scene_style,
        }
    }

    pub(super) fn update_model(&self, queue: &Queue, config: &SurfaceConfiguration) {
        self.model_uniform.update(queue, config);
    }

    pub(super) fn update_light(&mut self, queue: &Queue) {
        self.light_uniform.data = Light::from_light_style(&self.style.light);
        self.light_uniform.update(queue, &self.light_uniform.data);
    }
}

fn transform(aspect_ratio: f32) -> Mat4 {
    let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
    let view = glam::Mat4::look_at_rh(glam::Vec3::new(3., 4., 5.), glam::Vec3::ZERO, glam::Vec3::Y);
    projection * view
}
