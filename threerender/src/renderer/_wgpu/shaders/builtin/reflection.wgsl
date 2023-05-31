struct Reflection {
  intensity: f32,
  specular: f32,
  padding: vec2<f32>,
}

fn calc_specular_reflection(camera_position: vec4<f32>, position: vec4<f32>, normal: vec3<f32>, light_position: vec4<f32>, reflection: Reflection) -> vec4<f32> {
  let light_normal = normalize(calc_affine_normal(light_position, position).xyz);

  let eye_normal = normalize(calc_affine_normal(camera_position, position).xyz);

  let reflection_normal = normalize(light_normal + eye_normal);
  return pow(max(dot(normal, reflection_normal), 0.0), reflection.intensity) * vec4<f32>(reflection.specular);
}
