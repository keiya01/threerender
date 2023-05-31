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
    tex_idx: i32,
    normal_idx: i32,
}

@group(1)
@binding(0)
var<uniform> entity: Entity;

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @location(1) local_normal: vec3<f32>,
    @location(2) local_position: vec4<f32>,
    @location(3) world_view: vec4<f32>,
    @location(4) tex_coords: vec2<f32>,
    @location(5) tangent_matrix0: vec3<f32>,
    @location(6) tangent_matrix1: vec3<f32>,
    @location(7) tangent_matrix2: vec3<f32>,
    @builtin(position) world_position: vec4<f32>,
};

// Vertex entry point

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
) -> VertexOutput {
    let has_normal_map = entity.normal_idx != -1;
    var world_normal = normal;
    var world_tangent = tangent;
    var world_bitangent = bitangent;
    var tangent_matrix: mat3x3<f32>;
    if has_normal_map {
        world_normal = normalize(entity.transform * vec4<f32>(normal, 1.0)).xyz;
        world_tangent = normalize(entity.transform * vec4<f32>(tangent, 1.0)).xyz;
        world_bitangent = normalize(entity.transform * vec4<f32>(bitangent, 1.0)).xyz;
        tangent_matrix = transpose(mat3x3<f32>(
            world_tangent,
            world_bitangent,
            world_normal,
        ));
    }

    var local_normal = normalize(entity.transform * vec4<f32>(normal, 0.0)).xyz;
    var world_view = vec4(uscene.eye, 1.0);

    var local_position: vec4<f32>;
    if has_normal_map {
        local_position = vec4<f32>(tangent_matrix * position.xyz, position.w);
        world_view = vec4<f32>(tangent_matrix * world_view.xyz, 1.0);
    } else {
        local_position = entity.transform * position;
    };

    let entity_position = uscene.model * entity.transform * position;

    var result: VertexOutput;

    result.color = entity.color;
    result.world_position = entity_position;
    result.local_normal = local_normal;
    result.local_position = local_position;
    result.world_view = world_view;
    result.tex_coords = tex_coords;
    result.tangent_matrix0 = tangent_matrix[0];
    result.tangent_matrix1 = tangent_matrix[1];
    result.tangent_matrix2 = tangent_matrix[2];
    return result;
}

// Variables for fragment

// Light style
@group(2)
@binding(0)
var<uniform> ulights: array<UniformLight, #{MAX_LIGHT_NUM}>;

@group(3)
@binding(0)
var t_shadow: texture_depth_2d_array;

@group(3)
@binding(1)
var sampler_shadow: sampler_comparison;

#ifdef HAS_TEXTURE
@group(4)
@binding(0)
var texs: binding_array<texture_2d<f32>>;
@group(4)
@binding(1)
var sams: binding_array<sampler>;
#end

// Fragment entry point

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let has_normal_map = entity.normal_idx != -1;
    var normal: vec3<f32> = vertex.local_normal;
    if has_normal_map {
        normal = normalize(textureSampleLevel(texs[entity.normal_idx], sams[entity.normal_idx], vertex.tex_coords, 0.0)).xyz * 2.0 - 1.0;
    }

    var tangent_matrix: mat3x3<f32>;
    if has_normal_map {
        tangent_matrix = mat3x3<f32>(
            vertex.tangent_matrix0,
            vertex.tangent_matrix1,
            vertex.tangent_matrix2,
        );
    }

    var color: vec4<f32> = vec4(0.);
    let camera_position = vertex.world_view;
    for(var i = 0u; i < min(uscene.num_lights, #{MAX_LIGHT_NUM}u); i += 1u) {
        let ulight = ulights[i];
        if ulight.model != 0u {
            var light_position: vec4<f32>;
            if has_normal_map {
                light_position = vec4<f32>(tangent_matrix * ulight.position, 1.0);
            } else {
                light_position = vec4<f32>(ulight.position, 1.0);
            }

            // Directional light
            if ulight.model == 1u {
                let light = calc_directional_light(normal, vertex.local_position, ulight, light_position);

                let reflection = calc_specular_reflection(camera_position, vertex.local_position, normal, light_position, entity.reflection);

                // shadow
                if ulight.shadow.use_shadow == 1u {
                    color += ulight.ambient + calc_directional_shadow(
                        i,
                        normal,
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
                let light = calc_hemisphere_light(vertex.local_position, normal, ulight, light_position);

                color += ulight.ambient + light.color;
            }
        }
    }

    color *= vertex.color;

#ifdef HAS_TEXTURE
    if entity.tex_idx != -1 {
        color *= textureSampleLevel(texs[entity.tex_idx], sams[entity.tex_idx], vertex.tex_coords, 0.0);
    }
#end

    return color;
}