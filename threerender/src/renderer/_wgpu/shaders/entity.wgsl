// TODO: Create shared shader to share moules like light module, camera module, entity module, etc.

@group(0)
@binding(0)
var<uniform> umodel: mat4x4<f32>;

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
@group(1)
@binding(0)
var<uniform> ulight: Light;

struct Entity {
    transform: mat4x4<f32>,
    color: vec4<f32>,
}

@group(2)
@binding(0)
var<uniform> entity: Entity;

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) local_position: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOutput {
    let local_position = entity.transform * position;
    let entity_position = umodel * local_position;

    var result: VertexOutput;

    result.color = entity.color;
    result.position = entity_position;
    result.local_position = local_position;
    result.normal = normal;
    return result;
}

fn calc_directional_light(model: mat4x4<f32>, position: vec4<f32>, normal: vec3<f32>) -> vec4<f32> {
    // Normalizing matrix should always be calculated in shader.
    let world_normal = normalize(model * vec4<f32>(normal, 0.0)).xyz;
    let light_position = vec4<f32>(ulight.position, 1.0);
    let blightness = vec4<f32>(ulight.brightness);

    let light_normal = normalize((light_position * position.w - position * light_position.w).xyz);
    let light_power = max(dot(world_normal, light_normal), 0.0) * ulight.color * blightness;
    return light_power;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = vertex.color;
    var light: vec4<f32> = vec4(1.0);
    if ulight.model != 0u {
        let world_position = umodel * entity.transform;
        let entity_position = umodel * vertex.local_position;
        if ulight.model == 1u {
            light = calc_directional_light(world_position, entity_position, vertex.normal);
            light += ulight.ambient;
            color *= light;
        }
    }

    return color;
}