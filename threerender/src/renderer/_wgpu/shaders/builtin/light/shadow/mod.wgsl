#include builtin::light::shadow::uniforms
#include builtin::light::shadow::utils
#include builtin::light::shadow::normal
#include builtin::light::shadow::pcss

fn fetch_shadow(light_id: u32, homogeneous_coords: vec4<f32>, info: UniformShadow, t_shadow: texture_depth_2d_array, sampler_shadow: sampler, sampler_shadow_comparison: sampler_comparison) -> f32 {
    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }

    var visible: f32;
    if (info.shadow_type == 1u) {
      visible = percent_closer_soft_shadow(light_id, homogeneous_coords, info, t_shadow, sampler_shadow, sampler_shadow_comparison);    
    } else {
      visible = normal_shadow(light_id, homogeneous_coords, t_shadow, sampler_shadow, sampler_shadow_comparison);    
    }
    return visible;
}

fn calc_shadow_mask(
  idx: u32,
  shadow_position: vec4<f32>,
  info: UniformShadow,
  t_shadow: texture_depth_2d_array,
  sampler_shadow: sampler,
  sampler_comparison_shadow: sampler_comparison,
) -> f32 {
    // project into the light space
    let shadow = fetch_shadow(
      idx,
      shadow_position,
      info,
      t_shadow,
      sampler_shadow,
      sampler_comparison_shadow,
    );
    // add light contribution
    return shadow;
}
