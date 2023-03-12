use shader_processor::ShaderProcessor;

#[derive(Default)]
pub(super) struct ShaderProcessOption {
    pub(super) use_texture: bool,
    pub(super) support_storage: bool,
}

pub(super) fn process_shader(shader: &str, option: ShaderProcessOption) -> String {
    let mut s = ShaderProcessor::from_shader_str(shader);

    // math builtin modules
    let p = make_builtin_path("math/affine");
    s.insert_builtin("math::affine", &p);
    let p = make_builtin_path("math/mod");
    s.insert_builtin("math", &p);

    // light builtin modules
    let p = make_builtin_path("light/uniforms");
    s.insert_builtin("light::uniforms", &p);
    let p = make_builtin_path("light/types");
    s.insert_builtin("light::types", &p);
    let p = make_builtin_path("light/directional");
    s.insert_builtin("light::directional", &p);
    let p = make_builtin_path("light/hemisphere");
    s.insert_builtin("light::hemisphere", &p);
    let p = make_builtin_path("light/mod");
    s.insert_builtin("light", &p);

    // reflection
    let p = make_builtin_path("reflection");
    s.insert_builtin("reflection", &p);

    // shadow builtin modules
    let p = make_builtin_path("light/shadow/uniforms");
    s.insert_builtin("light::shadow::uniforms", &p);
    let p = make_builtin_path("light/shadow/directional");
    s.insert_builtin("light::shadow::directional", &p);
    let p = make_builtin_path("light/shadow/mod");
    s.insert_builtin("light::shadow", &p);

    // condition envs
    s.insert_condition_env("USE_TEXTURE", option.use_texture);
    s.insert_condition_env("SUPPORT_STORAGE", option.support_storage);

    s.process()
}

fn make_builtin_path(path: &str) -> String {
    let common_path =
        std::fs::canonicalize("./threerender/src/renderer/_wgpu/shaders/builtin/").unwrap();
    let common_path = common_path.to_str().unwrap();
    format!("{common_path}/{path}")
}
