struct UniformLight {
    color: vec4<f32>,
    ambient: vec4<f32>,
    position: vec3<f32>,
    brightness: f32,
    // 0: off
    // 1: directional light
    model: u32,
}

struct DirectionalLight {
    color: vec4<f32>,
    normal: vec3<f32>,
}

fn calc_directional_light(model: mat4x4<f32>, position: vec4<f32>, normal: vec3<f32>, ulight: UniformLight) -> DirectionalLight {
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
