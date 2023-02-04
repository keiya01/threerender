#include builtin::light
#include builtin::shadow

// Variables for vertex

@group(0)
@binding(0)
var<uniform> umodel: mat4x4<f32>;

struct Entity {
    transform: mat4x4<f32>,
    color: vec4<f32>,
}

@group(1)
@binding(0)
var<uniform> entity: Entity;

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) local_normal: vec3<f32>,
    @location(3) local_position: vec4<f32>,
#ifdef USE_TEXTURE
    @location(4) tex_coords: vec2<f32>,
#end
    @builtin(position) position: vec4<f32>,
};

// Vertex entry point

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
#ifdef USE_TEXTURE
    @location(2) tex_coords: vec2<f32>,
#end
) -> VertexOutput {
    let w = entity.transform;
    let local_position = entity.transform * position;
    let entity_position = umodel * local_position;

    var result: VertexOutput;

    result.color = entity.color;
    result.position = entity_position;
    result.local_normal = mat3x3<f32>(w.x.xyz, w.y.xyz, w.z.xyz) * vec3<f32>(normal.xyz);
    result.local_position = local_position;
    result.normal = normal;
#ifdef USE_TEXTURE
    result.tex_coords = tex_coords;
#end
    return result;
}

// Variables for fragment

// Light style
@group(2)
@binding(0)
var<uniform> ulight: UniformLight;

@group(3)
@binding(0)
var<uniform> ushadow: UniformShadow;

@group(3)
@binding(1)
var t_shadow: texture_depth_2d_array;

@group(3)
@binding(2)
var sampler_shadow: sampler_comparison;

#ifdef USE_TEXTURE
// For texture
struct TextureInfo {
    idx: u32,
}

@group(4)
@binding(0)
var texs: binding_array<texture_2d<f32>>;
@group(4)
@binding(1)
var sams: binding_array<sampler>;
@group(4)
@binding(2)
var<uniform> tex_info: TextureInfo;
#end

// Fragment entry point

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = vertex.color;
    if ulight.model != 0u {
        let world_position = umodel * entity.transform;
        let entity_position = umodel * vertex.local_position;

        // Directional light
        if ulight.model == 1u {
            let light = calc_directional_light(world_position, entity_position, vertex.normal, ulight);

            // shadow
            if ushadow.use_shadow == 1u {
                color *= ulight.ambient + calc_directional_shadow(
                    vertex.local_normal,
                    vertex.local_position,
                    light,
                    ushadow,
                    t_shadow,
                    sampler_shadow
                );
            } else {
                color *= ulight.ambient + light.color;
            }
        }

        // Hemisphere light
        if ulight.model == 2u {
            let light = calc_hemisphere_light(world_position, entity_position, vertex.normal, ulight);

            color *= ulight.ambient + light.color;
        }
    }

#ifdef USE_TEXTURE
    color *= textureSampleLevel(texs[tex_info.idx], sams[tex_info.idx], vertex.tex_coords, 0.0);
#end

    return color;
}