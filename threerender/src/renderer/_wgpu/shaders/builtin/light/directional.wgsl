fn calc_directional_light(world_normal: vec3<f32>, light_normal: vec3<f32>, ulight: UniformLight) -> DirectionalLight {
    let blightness = vec4<f32>(ulight.brightness);

    let diffuse = vec4((max(dot(world_normal, light_normal), 0.0) * ulight.color * blightness).xyz, 1.0);

    var d: DirectionalLight;
    d.color = diffuse;
    d.normal = light_normal;

    return d;
}
