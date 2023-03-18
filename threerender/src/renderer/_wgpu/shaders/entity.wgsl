#include builtin::math
#include builtin::light
#include builtin::reflection

// Variables for vertex

struct Scene {
    model: mat4x4<f32>,
    eye: vec3<f32>,
    num_lights: u32,
}

@group(0)
@binding(0)
var<uniform> uscene: Scene;

struct Entity {
    transform: mat4x4<f32>,
    color: vec4<f32>,
    reflection: Reflection,
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
    let entity_position = uscene.model * local_position;

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
#ifdef SUPPORT_STORAGE
@group(2)
@binding(0)
var<storage, read> ulights: array<UniformLight>;
#else
@group(2)
@binding(0)
var<uniform> ulights: array<UniformLight, #{MAX_LIGHT_NUM}>;
#end

@group(3)
@binding(0)
var t_shadow: texture_depth_2d_array;

@group(3)
@binding(1)
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
    var color: vec4<f32> = vec4(0.);
    let camera_position = vec4(uscene.eye, 1.0);
    let world_normal = normalize(entity.transform * vec4<f32>(vertex.normal, 0.0)).xyz;
    for(var i = 0u; i < min(uscene.num_lights, #{MAX_LIGHT_NUM}u); i += 1u) {
        let ulight = ulights[i];
        if ulight.model != 0u {

            // Directional light
            if ulight.model == 1u {
                let light = calc_directional_light(world_normal, vertex.local_position, vertex.normal, ulight);

                let reflection = calc_reflection(camera_position, vertex.local_position, world_normal, ulight.position, entity.reflection);

                // shadow
                if ulight.shadow.use_shadow == 1u {
                    color += ulight.ambient + calc_directional_shadow(
                        i,
                        vertex.local_normal,
                        vertex.local_position,
                        light,
                        ulight.shadow,
                        t_shadow,
                        sampler_shadow
                    ) + reflection;
                } else {
                    color += ulight.ambient + light.color + reflection;
                }
            }

            // Hemisphere light
            if ulight.model == 2u {
                let light = calc_hemisphere_light(entity.transform, vertex.local_position, vertex.normal, ulight);

                color += ulight.ambient + light.color;
            }
        }
    }

    color *= vertex.color;

#ifdef USE_TEXTURE
    color *= textureSampleLevel(texs[tex_info.idx], sams[tex_info.idx], vertex.tex_coords, 0.0);
#end

    return color;
}