fn calc_hemisphere_light(light_normal: vec3<f32>, local_normal: vec3<f32>, ulight: UniformLight) -> HemisphereLight {
    let hemisphere = (dot(local_normal, light_normal) + 1.0) * 0.5;
    let ambient: vec4<f32> = vec4(mix(ulight.hemisphere.ground_color, ulight.hemisphere.sky_color, hemisphere).xyz, 1.0);

    var h: HemisphereLight;
    h.color = ambient;

    return h;
}
