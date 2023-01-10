@group(0)
@binding(0)
var<uniform> umodel: mat4x4<f32>;

struct Light {
    color: vec4<f32>,
    position: vec3<f32>,
    brightness: f32,
    // 0: off
    // 1: diffuse reflection
    model: i32,
}

// Light style
@group(1)
@binding(0)
var<uniform> ulight: Light;

struct Entity {
    transform: mat4x4<f32>,
    // We don't use color property, but we should use it for simplifying render process.
    color: vec4<f32>,
}

@group(2)
@binding(0)
var<uniform> entity: Entity;

fn calc_diffuse_reflection_light(model: mat4x4<f32>, position: vec4<f32>, normal: vec3<f32>) -> vec4<f32> {
    // Normalizing matrix should always be calculated in shader.
    let world_normal = normalize(model * vec4<f32>(normal, 0.0)).xyz;
    let light_position = vec4<f32>(ulight.position, 1.0);
    let blightness = vec4<f32>(ulight.brightness);

    let light_normal = normalize((light_position * position.w - position * light_position.w).xyz);
    let light_power = max(dot(world_normal, light_normal), 0.0) * ulight.color * blightness;
    return light_power;
}

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
) -> VertexOutput {
    let world_position = umodel * entity.transform;
    let entity_position = world_position * position;

    var result: VertexOutput;
    var light: vec4<f32> = vec4(1.0);
    if ulight.model == 1 {
        light = calc_diffuse_reflection_light(world_position, entity_position, normal);
    }
    result.color = light;
    result.position = entity_position;
    result.tex_coords = tex_coords;
    return result;
}

struct TextureInfo {
    idx: u32,
}

@group(3)
@binding(0)
var texs: binding_array<texture_2d<f32>>;
@group(3)
@binding(1)
var sams: binding_array<sampler>;
@group(3)
@binding(2)
var<uniform> tex_info: TextureInfo;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color * textureSampleLevel(texs[tex_info.idx], sams[tex_info.idx], vertex.tex_coords, 0.0);
}
