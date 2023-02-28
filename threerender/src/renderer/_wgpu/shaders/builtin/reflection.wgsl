struct Reflection {
  brightness: f32,
  shininess: f32,
  specular: f32,
  padding: vec4<f32>,
}

fn calc_reflection(camera_position: vec4<f32>, position: vec4<f32>, normal: vec3<f32>, light_position: vec3<f32>, reflection: Reflection) -> vec4<f32> {
  let light_position = vec4<f32>(light_position, 1.0);
  let light_normal = normalize(calc_affine_normal(light_position, position).xyz);

  let eye_normal = normalize(calc_affine_normal(camera_position, position).xyz);

  let reflection_normal = normalize(light_normal + eye_normal);
  return pow(max(dot(normal, reflection_normal), 0.0), reflection.shininess) * vec4<f32>(reflection.brightness);
}
