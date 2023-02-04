fn calc_affine_normal(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
  return a * b.w - b * a.w;
}
