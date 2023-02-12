fn make_directional_shadow_mask(
  idx: u32,
  local_normal: vec3<f32>,
  local_position: vec4<f32>,
  light_dir: vec3<f32>,
  ushadow: UniformShadow,
  t_shadow: texture_depth_2d_array,
  sampler_shadow: sampler_comparison
) -> f32 {
    let normal = normalize(local_normal);
    // project into the light space
    let shadow = fetch_shadow(
      idx,
      ushadow.projection * local_position,
      t_shadow,
      sampler_shadow
    );
    // compute Lambertian diffuse term
    let diffuse = max(0.0, dot(normal, light_dir));
    // add light contribution
    return shadow * diffuse;
}

fn calc_directional_shadow(
  idx: u32,
  local_normal: vec3<f32>,
  local_position: vec4<f32>,
  d: DirectionalLight,
  ushadow: UniformShadow,
  t_shadow: texture_depth_2d_array,
  sampler_shadow: sampler_comparison
) -> vec4<f32> {
    return vec4<f32>(
      make_directional_shadow_mask(
        idx,
        local_normal,
        local_position,
        d.normal,
        ushadow,
        t_shadow,
        sampler_shadow
      ) * d.color.xyz, 1.0
    );
}
