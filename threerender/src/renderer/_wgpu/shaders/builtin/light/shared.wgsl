struct Reflection {
  specular: vec4<f32>,
  shininess: f32,
}

fn calc_reflection_light(position: vec4<f32>, normal: vec3<f32>, light_normal: vec3<f32>, reflection: Reflection) -> vec4<f32> {
  let eye_normal = normalize(position.xyz);
  let reflection_normal = normalize(light_normal - eye_normal);
  return pow(max(dot(normal, reflection_normal), 0.0), reflection.shininess) * reflection.specular;
}

struct HemisphereLight {
  ground_color: vec4<f32>,
  sky_color: vec4<f32>,
}

// Culling unused field is difficult, so we need to define field as common definition.
struct UniformLight {
    color: vec4<f32>,
    ambient: vec4<f32>,
    position: vec3<f32>,
    brightness: f32,
    // 0: off
    // 1: directional light
    // 2: hemisphere light
    model: u32,
    reflection: Reflection,
    hemisphere: HemisphereLight,
}
