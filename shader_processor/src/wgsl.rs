use std::{
    collections::HashMap,
    fmt::Debug,
    fs::File,
    io::Read,
    iter::Peekable,
    str::{Chars, Lines},
};

enum IFStatement {
    Ifdef(bool),
    Else(bool),
}

const EXTENSION: &str = "wgsl";

#[derive(Debug, Clone)]
pub enum EnvType {
    Bool(bool),
    Number(u32),
    Str(String),
}

impl Eq for EnvType {}

impl PartialEq for EnvType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EnvType::Bool(a), EnvType::Bool(b)) => a == b,
            (EnvType::Number(a), EnvType::Number(b)) => a == b,
            (EnvType::Str(a), EnvType::Str(b)) => a == b,
            _ => false,
        }
    }
}

pub enum ProcessError {
    Unexpected(EnvType),
    CouldNotFindBuiltinModule(String),
    CouldNotFindEnv(String),
    InValidStatement(String),
}

impl Debug for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unexpected(env) => write!(f, "{:?}", env),
            Self::CouldNotFindBuiltinModule(line) => {
                write!(f, "Could not find specified builtin module: {line}")
            }
            Self::CouldNotFindEnv(env) => {
                write!(f, "Could not find specified env: {env}")
            }
            Self::InValidStatement(statement) => write!(f, "Invalid statement: {statement}"),
        }
    }
}

pub struct ShaderProcessor<'a> {
    shader: &'a str,
    result: Option<String>,
    envs: HashMap<&'a str, EnvType>,
    builtin: HashMap<&'a str, String>,
    lines: Vec<String>,
    if_nest_order: Vec<IFStatement>,
    is_dirty: bool,
}

impl<'a> ShaderProcessor<'a> {
    pub fn from_shader_str(shader: &'a str) -> Self {
        Self {
            shader,
            result: None,
            envs: HashMap::new(),
            builtin: HashMap::new(),
            lines: vec![],
            if_nest_order: vec![],
            is_dirty: true,
        }
    }

    pub fn insert_env(&mut self, key: &'a str, val: EnvType) {
        self.is_dirty = self.envs.get(key).map_or(true, |v| v != &val);
        self.envs.insert(key, val);
    }

    pub fn insert_builtin(&mut self, key: &'a str, val: String) {
        self.is_dirty = self.builtin.get(key).map_or(true, |v| v != &val);
        self.builtin.insert(key, val);
    }

    pub fn process(&mut self) -> Result<String, ProcessError> {
        if !self.is_dirty {
            return Ok(self.result.as_ref().unwrap().clone());
        }

        let lines = self.shader.lines();
        self.process_lines(lines)?;
        self.is_dirty = false;
        self.result = Some(self.lines.concat());
        self.lines.clear();

        Ok(self.result.as_ref().unwrap().clone())
    }

    fn process_lines(&mut self, lines: Lines) -> Result<(), ProcessError> {
        for (_, line) in lines.enumerate() {
            if self.handle_ifdef_statement(line)? {
                continue;
            }
            match self.if_nest_order.last() {
                Some(IFStatement::Ifdef(matched) | IFStatement::Else(matched)) if !*matched => {
                    continue
                }
                _ => {}
            };

            if self.handle_include_statement(line)? {
                continue;
            }

            let mut in_env_syntax = false;
            let mut env_variable_names: Vec<String> = vec![];
            let mut env_variable_name = String::new();
            self.handle_chars(line, |ch, chars| {
                if find_env_variable(&mut in_env_syntax, ch, chars, &mut env_variable_name) {
                    env_variable_names.push(env_variable_name.clone());
                    env_variable_name = String::new();
                }

                false
            });

            let line = self.handle_env_variable(line, env_variable_names)?;

            self.lines.push(format!("{}{}", line, "\n"));
        }
        Ok(())
    }

    fn handle_ifdef_statement(&mut self, line: &str) -> Result<bool, ProcessError> {
        if line.starts_with("#ifdef") {
            match self.if_nest_order.last() {
                Some(IFStatement::Ifdef(matched) | IFStatement::Else(matched)) if !*matched => {
                    self.if_nest_order.push(IFStatement::Ifdef(false));
                    return Ok(true);
                }
                _ => {}
            };

            let split = line.split(' ').collect::<Vec<&str>>();
            let env = split.get(1);
            let matched = match env {
                Some(env) => {
                    let env = match self.envs.get(env) {
                        Some(e) => e,
                        None => &EnvType::Bool(false),
                    };
                    match env {
                        EnvType::Bool(b) => *b,
                        _ => return Err(ProcessError::Unexpected(env.clone())),
                    }
                }
                None => false,
            };
            self.if_nest_order.push(IFStatement::Ifdef(matched));
            return Ok(true);
        }

        if line.starts_with("#else") {
            let i = self.if_nest_order.pop();
            let matched = match i {
                Some(IFStatement::Ifdef(matched)) => !matched,
                _ => unreachable!(),
            };
            self.if_nest_order.push(IFStatement::Else(matched));
            return Ok(true);
        }

        if line.starts_with("#end") {
            let i = self.if_nest_order.pop();
            // assert if statement
            match i {
                Some(IFStatement::Ifdef(_) | IFStatement::Else(_)) => {}
                None => unreachable!(),
            };
            return Ok(true);
        }

        Ok(false)
    }

    fn handle_include_statement(&mut self, line: &str) -> Result<bool, ProcessError> {
        if !line.starts_with("#include") {
            return Ok(false);
        }

        let split_line = line.split(' ').collect::<Vec<&str>>();
        let mut path = match split_line.get(1) {
            Some(s) => *s,
            None => return Err(ProcessError::InValidStatement(line.to_owned())),
        };

        if path.starts_with("builtin") {
            let include = path.trim_start_matches("builtin::");
            match self.builtin.get(include) {
                Some(s) => path = s,
                None => return Err(ProcessError::CouldNotFindBuiltinModule(include.to_owned())),
            };
        }

        let path = &format!("{path}.{EXTENSION}");

        let mut file = File::open(path).unwrap_or_else(|_| panic!("Could not find {path}"));
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .unwrap_or_else(|_| panic!("Failed to read file {path}"));
        self.process_lines(contents.lines())?;

        Ok(true)
    }

    fn handle_chars<F>(&mut self, line: &str, mut f: F)
    where
        F: FnMut(&char, &mut Peekable<Chars>) -> bool,
    {
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if f(&ch, &mut chars) {
                break;
            }
        }
    }

    fn handle_env_variable(
        &mut self,
        line: &str,
        env_variable_names: Vec<String>,
    ) -> Result<String, ProcessError> {
        let mut next_line = line.to_owned();
        for env_variable_name in env_variable_names {
            if env_variable_name.is_empty() {
                return Ok(next_line);
            }

            let env_variable = self.envs.get(&env_variable_name[..]);
            let var = match env_variable {
                Some(var) => var,
                None => return Err(ProcessError::CouldNotFindEnv(env_variable_name)),
            };
            let str_var_val = match var {
                EnvType::Number(n) => n.to_string(),
                EnvType::Str(n) => n.to_string(),
                EnvType::Bool(n) => n.to_string(),
            };

            let split_line = next_line.split_once(&format!("#{{{env_variable_name}}}"));
            next_line = match split_line {
                Some((a, b)) => {
                    let mut line = String::new();
                    line.push_str(a);
                    line.push_str(&str_var_val);
                    line.push_str(b);
                    line
                }
                None => unreachable!(),
            };
        }

        Ok(next_line)
    }
}

fn find_env_variable(
    in_env_syntax: &mut bool,
    ch: &char,
    chars: &mut Peekable<Chars>,
    env_variable_name: &mut String,
) -> bool {
    if ch == &'#' && !*in_env_syntax {
        let ch = match chars.peek() {
            Some(c) => c,
            None => return false,
        };
        if ch == &'{' {
            *in_env_syntax = true;
            chars.next();
            return false;
        }
    }

    if *in_env_syntax && ch == &'}' {
        *in_env_syntax = false;
        return true;
    }

    if *in_env_syntax && (ch.is_uppercase() || ch == &'_') {
        env_variable_name.push(*ch);
        return false;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_ifdef() {
        const IFDEF_SHADER_INPUT: &str = r"
#ifdef ENABLE_ENTITY
struct Entity {
    pos: vec4<f32>,
}
var<uniform> uentity: Entity;
#ifdef ENABLE_TEXTURE
struct Texture {
    pos: vec4<f32>,
}
var<uniform> utexture: Texture;
#end
#else
#ifdef BIAS
var<uniform> ubias: f32;
#end
var<uniform> upos: vec4<f32>;
#end

@vertex
fn main(@location(0) pos: vec4<f32>) -> vec4<f32> {
    var pos: vec4<f32> = pos;
#ifdef ENABLE_ENTITY
    pos *= uentity.pos;
#ifdef ENABLE_TEXTURE
    pos *= utexture.pos;
#end

#else
#ifdef BIAS
    pos *= ubias;
#end
    pos *= upos;
#end
    return pos;
}
";
        // Check only entity
        let mut p = ShaderProcessor::from_shader_str(IFDEF_SHADER_INPUT);
        p.insert_env("ENABLE_ENTITY", EnvType::Bool(true));
        assert_eq!(
            &p.process().unwrap(),
            r"
struct Entity {
    pos: vec4<f32>,
}
var<uniform> uentity: Entity;

@vertex
fn main(@location(0) pos: vec4<f32>) -> vec4<f32> {
    var pos: vec4<f32> = pos;
    pos *= uentity.pos;

    return pos;
}
"
        );

        // Check only entity
        let mut p = ShaderProcessor::from_shader_str(IFDEF_SHADER_INPUT);
        p.insert_env("ENABLE_ENTITY", EnvType::Bool(true));
        p.insert_env("ENABLE_TEXTURE", EnvType::Bool(false));
        assert_eq!(
            &p.process().unwrap(),
            r"
struct Entity {
    pos: vec4<f32>,
}
var<uniform> uentity: Entity;

@vertex
fn main(@location(0) pos: vec4<f32>) -> vec4<f32> {
    var pos: vec4<f32> = pos;
    pos *= uentity.pos;

    return pos;
}
"
        );

        let mut p = ShaderProcessor::from_shader_str(IFDEF_SHADER_INPUT);
        p.insert_env("ENABLE_ENTITY", EnvType::Bool(true));
        p.insert_env("ENABLE_TEXTURE", EnvType::Bool(true));
        assert_eq!(
            &p.process().unwrap(),
            r"
struct Entity {
    pos: vec4<f32>,
}
var<uniform> uentity: Entity;
struct Texture {
    pos: vec4<f32>,
}
var<uniform> utexture: Texture;

@vertex
fn main(@location(0) pos: vec4<f32>) -> vec4<f32> {
    var pos: vec4<f32> = pos;
    pos *= uentity.pos;
    pos *= utexture.pos;

    return pos;
}
"
        );
        // Check ubias
        let mut p = ShaderProcessor::from_shader_str(IFDEF_SHADER_INPUT);
        p.insert_env("ENABLE_ENTITY", EnvType::Bool(false));
        p.insert_env("ENABLE_TEXTURE", EnvType::Bool(true));
        p.insert_env("BIAS", EnvType::Bool(true));
        assert_eq!(
            &p.process().unwrap(),
            r"
var<uniform> ubias: f32;
var<uniform> upos: vec4<f32>;

@vertex
fn main(@location(0) pos: vec4<f32>) -> vec4<f32> {
    var pos: vec4<f32> = pos;
    pos *= ubias;
    pos *= upos;
    return pos;
}
"
        );
    }

    #[test]
    fn process_runtime_env() {
        const ENV_SHADER_INPUT: &str = r"
struct Entity {
    pos: vec4<#{POSITION_TYPE}>,
    lights: array<f32, #{MAX_LIGHT_NUM}>,
}
var<uniform> uentity: Entity;

@vertex
fn main(@location(0) pos: vec4<f32>) -> vec4<f32> {
    var pos: vec4<f32> = pos;
    for(var i = 0u; i < #{MAX_LIGHT_NUM}u; i += #{ADDITIONAL_NUM}u) {
        pos *= uentity.lights[i];
    }
    return pos;
}
";
        // Check only entity
        let mut p = ShaderProcessor::from_shader_str(ENV_SHADER_INPUT);
        p.insert_env("MAX_LIGHT_NUM", EnvType::Number(10));
        p.insert_env("ADDITIONAL_NUM", EnvType::Number(1));
        p.insert_env("POSITION_TYPE", EnvType::Str("f32".to_owned()));
        p.insert_env("POSITION", EnvType::Number(100));
        assert_eq!(
            &p.process().unwrap(),
            r"
struct Entity {
    pos: vec4<f32>,
    lights: array<f32, 10>,
}
var<uniform> uentity: Entity;

@vertex
fn main(@location(0) pos: vec4<f32>) -> vec4<f32> {
    var pos: vec4<f32> = pos;
    for(var i = 0u; i < 10u; i += 1u) {
        pos *= uentity.lights[i];
    }
    return pos;
}
"
        );
    }

    #[test]
    fn process_includes() {
        const INCLUDE_SHADER_INPUT: &str = r#"
#include builtin::light
#include ./assets/test

@vertex
fn main() vec4<f32> {
    var bias: f32 = 0.0;
#ifdef USE_BIAS
    bias = calc_bias();
#end
    return calc_light() * calc_test() * bias; 
}
"#;
        let mut p = ShaderProcessor::from_shader_str(INCLUDE_SHADER_INPUT);
        p.insert_env("USE_BIAS", EnvType::Bool(false));
        p.insert_builtin("light", "./assets/builtin/light".to_owned());
        assert_eq!(
            &p.process().unwrap(),
            r#"
fn calc_light() -> vec4<f32> {
    return vec4<f32>(5.0);
}

fn calc_test() -> vec4<f32> {
    return vec4<f32>(3.0);
}

@vertex
fn main() vec4<f32> {
    var bias: f32 = 0.0;
    return calc_light() * calc_test() * bias; 
}
"#
        );
        let mut p = ShaderProcessor::from_shader_str(INCLUDE_SHADER_INPUT);
        p.insert_env("USE_BIAS", EnvType::Bool(true));
        p.insert_env("BIAS_TYPE", EnvType::Str("f32".to_owned()));
        p.insert_builtin("light", "./assets/builtin/light".to_owned());
        assert_eq!(
            &p.process().unwrap(),
            r#"
fn calc_light() -> vec4<f32> {
    return vec4<f32>(5.0);
}
fn calc_test_bias() -> f32 {
    return 3.0;
}

fn calc_test() -> vec4<f32> {
    return vec4<f32>(3.0);
}

@vertex
fn main() vec4<f32> {
    var bias: f32 = 0.0;
    bias = calc_bias();
    return calc_light() * calc_test() * bias; 
}
"#
        );
    }

    #[test]
    fn check_dirty() {
        const SHADER_INPUT: &str = r#"
#include builtin::light
#include ./assets/test

@vertex
fn main() vec4<f32> {
    var bias: f32 = 0.0;
#ifdef USE_BIAS
    bias = calc_bias();
#end
    return calc_light() * calc_test() * bias; 
}
"#;
        let mut p = ShaderProcessor::from_shader_str(SHADER_INPUT);
        p.insert_env("USE_BIAS", EnvType::Bool(false));
        p.insert_builtin("light", "./assets/builtin/light".to_owned());
        assert_eq!(
            &p.process().unwrap(),
            r#"
fn calc_light() -> vec4<f32> {
    return vec4<f32>(5.0);
}

fn calc_test() -> vec4<f32> {
    return vec4<f32>(3.0);
}

@vertex
fn main() vec4<f32> {
    var bias: f32 = 0.0;
    return calc_light() * calc_test() * bias; 
}
"#
        );

        assert!(!p.is_dirty);

        p.insert_builtin("light", "./assets/builtin/light".to_owned());
        assert!(!p.is_dirty);

        p.insert_env("USE_BIAS", EnvType::Bool(true));
        p.insert_env("BIAS_TYPE", EnvType::Str("u32".to_owned()));
        assert!(p.is_dirty);

        assert_eq!(
            &p.process().unwrap(),
            r#"
fn calc_light() -> vec4<f32> {
    return vec4<f32>(5.0);
}
fn calc_test_bias() -> u32 {
    return 3.0;
}

fn calc_test() -> vec4<f32> {
    return vec4<f32>(3.0);
}

@vertex
fn main() vec4<f32> {
    var bias: f32 = 0.0;
    bias = calc_bias();
    return calc_light() * calc_test() * bias; 
}
"#
        );

        assert!(!p.is_dirty);

        p.insert_env("USE_BIAS", EnvType::Bool(true));
        p.insert_env("BIAS_TYPE", EnvType::Str("u32".to_owned()));
        assert!(!p.is_dirty);

        p.insert_builtin("light", "./assets/builtin/light2".to_owned());
        assert!(p.is_dirty);

        assert_eq!(
            &p.process().unwrap(),
            r#"
fn calc_light() -> vec4<f32> {
    return vec4<f32>(10.0);
}
fn calc_test_bias() -> u32 {
    return 3.0;
}

fn calc_test() -> vec4<f32> {
    return vec4<f32>(3.0);
}

@vertex
fn main() vec4<f32> {
    var bias: f32 = 0.0;
    bias = calc_bias();
    return calc_light() * calc_test() * bias; 
}
"#
        );
    }
}
