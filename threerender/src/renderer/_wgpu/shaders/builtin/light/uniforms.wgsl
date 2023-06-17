struct UniformHemisphereLight {
  ground_color: vec4<f32>,
  sky_color: vec4<f32>,
}

// Culling unused field is difficult, so we need to define field as common definition.
struct UniformLight {
    color: vec4<f32>,
    position: vec3<f32>,
    brightness: f32,
    // 0: off
    // 1: directional light
    // 2: hemisphere light
    // 3: ambient light
    model: u32,
    hemisphere: UniformHemisphereLight,
    shadow: UniformShadow,
}
