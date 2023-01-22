// TODO: Create shared shader to share moules like light module, camera module, entity module, etc.

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
    @location(4) tex_coords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

// Vertex entry point

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
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
    result.tex_coords = tex_coords;
    return result;
}

struct Light {
    color: vec4<f32>,
    ambient: vec4<f32>,
    position: vec3<f32>,
    brightness: f32,
    // 0: off
    // 1: directional light
    model: u32,
}

// Light style
@group(2)
@binding(0)
var<uniform> ulight: Light;

struct Shadow {
    projection: mat4x4<f32>,
    // 0 or 1
    use_shadow: u32,
}

@group(3)
@binding(0)
var<uniform> ushadow: Shadow;

@group(3)
@binding(1)
var t_shadow: texture_depth_2d_array;

@group(3)
@binding(2)
var sampler_shadow: sampler_comparison;

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

// Common

fn fetch_shadow(light_id: u32, homogeneous_coords: vec4<f32>) -> f32 {
    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }

    // compensate for the Y-flip difference between the NDC and texture coordinates
    let flip_correction = vec2<f32>(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    let proj_correction = 1.0 / homogeneous_coords.w;
    let light_local = homogeneous_coords.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
    // do the lookup, using HW PCF and comparison
    return textureSampleCompareLevel(t_shadow, sampler_shadow, light_local, i32(light_id), homogeneous_coords.z * proj_correction);
}

// Directional light

struct DirectionalLight {
    color: vec4<f32>,
    normal: vec3<f32>,
}

fn calc_directional_light(model: mat4x4<f32>, position: vec4<f32>, normal: vec3<f32>) -> DirectionalLight {
    // Normalizing matrix should always be calculated in shader.
    let world_normal = normalize(model * vec4<f32>(normal, 0.0)).xyz;
    let light_position = vec4<f32>(ulight.position, 1.0);
    let blightness = vec4<f32>(ulight.brightness);

    let light_normal = normalize((light_position * position.w - position * light_position.w).xyz);
    let light_color = max(dot(world_normal, light_normal), 0.0) * ulight.color * blightness;

    var d: DirectionalLight;
    d.color = light_color;
    d.normal = light_normal;

    return d;
}

fn make_directional_shadow_mask(vertex: VertexOutput, light_dir: vec3<f32>) -> f32 {
    let normal = normalize(vertex.local_normal);
    // project into the light space
    let shadow = fetch_shadow(0u, ushadow.projection * vertex.local_position);
    // compute Lambertian diffuse term
    let diffuse = max(0.0, dot(normal, light_dir));
    // add light contribution
    return shadow * diffuse;
}

fn calc_directional_shadow(vertex: VertexOutput, d: DirectionalLight) -> vec4<f32> {
    return vec4<f32>(make_directional_shadow_mask(vertex, d.normal) * d.color.xyz, 1.0);
}

// Fragment entry point

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = vertex.color;
    if ulight.model != 0u {
        let world_position = umodel * entity.transform;
        let entity_position = umodel * vertex.local_position;

        // Directional light
        if ulight.model == 1u {
            let light = calc_directional_light(world_position, entity_position, vertex.normal);

            // shadow
            if ushadow.use_shadow == 1u {
                color *= ulight.ambient + calc_directional_shadow(vertex, light);
            } else {
                color *= ulight.ambient + light.color;
            }
        }
    }

    // For texture
    return color * textureSampleLevel(texs[tex_info.idx], sams[tex_info.idx], vertex.tex_coords, 0.0);
}
