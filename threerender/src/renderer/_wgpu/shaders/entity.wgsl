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
    normal_transform: mat4x4<f32>,
    color: vec4<f32>,
    tex_idx: vec4<i32>,
    normal_idx: vec4<i32>,
    reflection: Reflection,
}

@group(1)
@binding(0)
var<uniform> entity: Entity;

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @location(1) local_normal: vec3<f32>,
    @location(2) local_position: vec4<f32>,
    @location(3) tangent_or_local_position: vec4<f32>,
    @location(4) world_view: vec4<f32>,
    @location(5) tex_coords: vec2<f32>,
    @location(6) tangent_matrix0: vec3<f32>,
    @location(7) tangent_matrix1: vec3<f32>,
    @location(8) tangent_matrix2: vec3<f32>,
    @builtin(position) world_position: vec4<f32>,
};

fn convert_normal_transform() -> mat3x3<f32> {
    return mat3x3<f32>(
        entity.normal_transform.x.xyz,
        entity.normal_transform.y.xyz,
        entity.normal_transform.z.xyz,
    );
}

// Vertex entry point

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
) -> VertexOutput {
    let normal_transform = convert_normal_transform();
    let has_normal_map = entity.normal_idx.x != -1;
    var tangent_matrix: mat3x3<f32>;
    if has_normal_map {
        let world_normal = normalize(normal_transform * normal);
        let world_tangent = normalize(normal_transform * tangent);
        let world_bitangent = normalize(normal_transform * bitangent);
        tangent_matrix = transpose(mat3x3(
            world_tangent,
            world_bitangent,
            world_normal,
        ));
    }

    var local_normal = normalize(normal_transform * normal);
    var world_view = vec4(uscene.eye, 1.0);

    let local_position = entity.transform * position;

    var tangent_or_local_position: vec4<f32>;
    if has_normal_map {
        tangent_or_local_position = vec4<f32>(local_position.xyz, local_position.w);
        world_view = vec4<f32>(tangent_matrix * world_view.xyz, 1.0);
    } else {
        tangent_or_local_position = local_position;
    };

    let entity_position = uscene.model * entity.transform * position;

    var result: VertexOutput;

    result.color = entity.color;
    result.local_normal = local_normal;
    result.local_position = local_position;
    result.tangent_or_local_position = tangent_or_local_position;
    result.world_view = world_view;
    result.tex_coords = tex_coords;
    result.tangent_matrix0 = tangent_matrix.x;
    result.tangent_matrix1 = tangent_matrix.y;
    result.tangent_matrix2 = tangent_matrix.z;
    result.world_position = entity_position;
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
    let normal_transform = convert_normal_transform();
    let has_normal_map = entity.normal_idx.x != -1;
    var normal: vec3<f32> = vertex.local_normal;
    var tangent_matrix: mat3x3<f32>;
#ifdef HAS_TEXTURE
    if has_normal_map {
        normal = normalize(textureSample(texs[entity.normal_idx.x], sams[entity.normal_idx.x], vertex.tex_coords)).xyz * 2.0 - 1.0;
        tangent_matrix = mat3x3<f32>(
            vertex.tangent_matrix0,
            vertex.tangent_matrix1,
            vertex.tangent_matrix2,
        );
    }
#end

    var color: vec4<f32> = vec4(0.);
    let camera_position = vertex.world_view;
    for(var i = 0u; i < min(uscene.num_lights, #{MAX_LIGHT_NUM}u); i += 1u) {
        let ulight = ulights[i];
        if ulight.model != 0u {
            let light_position = vec4<f32>(ulight.position, 1.0);
            var light_normal: vec3<f32> = calc_affine_normal(light_position, vertex.local_position).xyz;
            if has_normal_map {
                light_normal = tangent_matrix * light_normal;
            }
            light_normal = normalize(light_normal);

            // Directional light
            if ulight.model == 1u {
                let light = calc_directional_light(normal, light_normal, ulight);

                let reflection = calc_specular_reflection(camera_position, vertex.tangent_or_local_position, normal, light_normal, entity.reflection);

                // shadow
                if ulight.shadow.use_shadow == 1u {
                    color += ulight.ambient + vec4(calc_shadow_mask(
                        i,
                        ulight.shadow.projection * vertex.local_position,
                        t_shadow,
                        sampler_shadow
                    ) * light.color.xyz * ulight.shadow.alpha, 1.0) + reflection;
                } else {
                    color += ulight.ambient + light.color + reflection;
                }
            }

            // Hemisphere light
            if ulight.model == 2u {
                let light = calc_hemisphere_light(light_normal, normal, ulight);

                color += ulight.ambient + light.color;
            }
        }
    }

    color *= vertex.color;

#ifdef HAS_TEXTURE
    if entity.tex_idx.x != -1 {
        color *= textureSample(texs[entity.tex_idx.x], sams[entity.tex_idx.x], vertex.tex_coords);
    }
#end

    return color;
}