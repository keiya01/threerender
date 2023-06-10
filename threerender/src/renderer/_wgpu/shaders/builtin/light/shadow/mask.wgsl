fn calc_shadow_mask(
  idx: u32,
  shadow_position: vec4<f32>,
  t_shadow: texture_depth_2d_array,
  sampler_shadow: sampler_comparison
) -> f32 {
    // project into the light space
    let shadow = fetch_shadow(
      idx,
      shadow_position,
      t_shadow,
      sampler_shadow
    );
    // add light contribution
    return shadow;
}
