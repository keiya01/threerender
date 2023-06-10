const LIGHT_SIZE_UV = 0.881473517949; // 1 / (FOV * MATH.PI / 180)
const NEAR_PLANE = 1.0; // NEAR PLANE
const BLOCKER_SEARCH_NUM_SAMPLES = 16u;
const PCF_NUM_SAMPLES = 16u;

//Parallel plane estimation
fn penumbra_size(z_receiver: f32, z_blocker: f32) -> f32 {
    return (z_receiver - z_blocker) / z_blocker;
}

fn find_blocker(
    avg_blocker_depth: ptr<function, f32>,
    num_blockers: ptr<function, f32>,
    uv: vec2<f32>,
    z_receiver: f32,
    light_id: u32,
    info: UniformShadow,
    t_shadow: texture_depth_2d_array,
    sampler_shadow: sampler,
    poissonDisk: ptr<function, array<vec2<f32>, 16>>,
) {
    //This uses similar triangles to compute what
    //area of the shadow map we should search
    let searchWidth = info.light_uv * (z_receiver - info.near_plane) / z_receiver;
    var blocker_sum: f32 = 0.;
    *num_blockers = 0.;
    for(var i = 0u; i < BLOCKER_SEARCH_NUM_SAMPLES; i += 1u) {
        let shadow_map_depth = textureSampleLevel(
            t_shadow,
            sampler_shadow,
            uv + (*poissonDisk)[i] * searchWidth,
            i32(light_id),
            0.,
        );
        if (shadow_map_depth < z_receiver) {
            blocker_sum += shadow_map_depth;
            *num_blockers += 1.;
        }
    }
    *avg_blocker_depth = blocker_sum / (*num_blockers);
}

fn pcf_filter(
    uv: vec2<f32>,
    z_receiver: f32,
    filter_radius_uv: f32,
    light_id: u32,
    t_shadow: texture_depth_2d_array,
    sampler_shadow_comparison: sampler_comparison,
    poissonDisk: ptr<function, array<vec2<f32>, 16>>,
) -> f32 {
    var sum: f32 = 0.;
    for (var i = 0u; i < PCF_NUM_SAMPLES; i += 1u) {
        let offset = (*poissonDisk)[i] * filter_radius_uv;
        sum += textureSampleCompareLevel(
            t_shadow,
            sampler_shadow_comparison,
            uv + offset,
            i32(light_id),
            z_receiver
        );
    }
    return sum / f32(PCF_NUM_SAMPLES);
}

// Ref: https://developer.download.nvidia.com/whitepapers/2008/PCSS_Integration.pdf
fn percent_closer_soft_shadow(
    light_id: u32,
    homogeneous_coords: vec4<f32>,
    info: UniformShadow,
    t_shadow: texture_depth_2d_array,
    sampler_shadow: sampler,
    sampler_shadow_comparison: sampler_comparison,
) -> f32{
    // TODO: Fix to declare as const var.
    var poissonDisk = array<vec2<f32>, 16>(
        vec2(-0.94201624, -0.39906216),
        vec2(0.94558609, -0.76890725),
        vec2(-0.094184101, -0.92938870),
        vec2(0.34495938, 0.29387760),
        vec2(-0.91588581, 0.45771432),
        vec2(-0.81544232, -0.87912464),
        vec2(-0.38277543, 0.27676845),
        vec2(0.97484398, 0.75648379),
        vec2(0.44323325, -0.97511554),
        vec2(0.53742981, -0.47373420),
        vec2(-0.26496911, -0.41893023),
        vec2(0.79197514, 0.19090188),
        vec2(-0.24188840, 0.99706507),
        vec2(-0.81409955, 0.91437590),
        vec2(0.19984126, 0.78641367),
        vec2(0.14383161, -0.14100790),
    );

    let coords = shadow_coords(homogeneous_coords);

    let uv = coords.xy;
    let z_receiver = coords.z; // Assumed to be eye-space z in this code
    // STEP 1: blocker search
    var avg_blocker_depth: f32 = 0.;
    var num_blockers: f32 = 0.;
    find_blocker(&avg_blocker_depth, &num_blockers, uv, z_receiver, light_id, info, t_shadow, sampler_shadow, &poissonDisk);
    if(num_blockers < 1.) {
        // There are no occluders so early out (this saves filtering)
        return 1.;
    }
    // STEP 2: penumbra size
    let penumbra_ratio = penumbra_size(z_receiver, avg_blocker_depth);
    let filter_radius_uv = penumbra_ratio * info.light_uv * info.near_plane / z_receiver;
    // STEP 3: filtering
    return pcf_filter(uv, z_receiver, filter_radius_uv, light_id, t_shadow, sampler_shadow_comparison, &poissonDisk);
}
