use std::{collections::HashMap, fs::File, io::{Read}, str::Lines};

enum IFStatement {
    IFDEF(bool),
    ELSE(bool),
}

const EXTENSION: &'static str = "wgsl";

pub struct ShaderProcessor<'a> {
    shader: &'a str,
    envs: HashMap<&'a str, bool>,
    builtin: HashMap<&'a str, &'a str>,
    lines: Vec<String>,
    if_nest_order: Vec<IFStatement>,
}

impl<'a> ShaderProcessor<'a> {
    pub fn from_shader_str(shader: &'a str) -> Self {
        Self {
            shader,
            envs: HashMap::new(),
            builtin: HashMap::new(),
            lines: vec![],
            if_nest_order: vec![],
        }
    }

    pub fn insert_env(&mut self, key: &'a str, val: bool) {
        self.envs.insert(key, val);
    }

    pub fn insert_builtin(&mut self, key: &'a str, val: &'a str) {
        self.builtin.insert(key, val);
    }

    pub fn process(&mut self) -> String {
        let lines = self.shader.lines();
        self.process_lines(lines);
        self.lines.concat()
    }

    fn process_lines(&mut self, lines: Lines) {
      for (_, line) in lines.enumerate() {
        if self.handle_ifdef_statement(line) {
            continue;
        }
        match self.if_nest_order.last() {
            Some(IFStatement::IFDEF(matched) | IFStatement::ELSE(matched)) if !*matched => {
                continue
            }
            _ => {}
        };

        if self.handle_include_statement(line) {
          continue;
        }

        self.lines.push(format!("{}{}", line, "\n"));
    }
    }

    fn handle_ifdef_statement(&mut self, line: &str) -> bool {
        if line.starts_with("#ifdef") {
            match self.if_nest_order.last() {
                Some(IFStatement::IFDEF(matched) | IFStatement::ELSE(matched)) if !*matched => {
                    self.if_nest_order.push(IFStatement::IFDEF(false));
                    return true;
                }
                _ => {}
            };

            let split = line.split(" ").collect::<Vec<&str>>();
            let env = split.get(1);
            let matched = match env {
                Some(env) => self.envs.get(env).map_or(false, |e| *e),
                None => false,
            };
            self.if_nest_order.push(IFStatement::IFDEF(matched));
            return true;
        }

        if line.starts_with("#else") {
            let i = self.if_nest_order.pop();
            let matched = match i {
                Some(IFStatement::IFDEF(matched)) => !matched,
                _ => unreachable!(),
            };
            self.if_nest_order.push(IFStatement::ELSE(matched));
            return true;
        }

        if line.starts_with("#end") {
            let i = self.if_nest_order.pop();
            // assert if statement
            match i {
                Some(IFStatement::IFDEF(_) | IFStatement::ELSE(_)) => {}
                None => unreachable!(),
            };
            return true;
        }

        false
    }

    fn handle_include_statement(&mut self, line: &str) -> bool {
      if !line.starts_with("#include") {
        return false;
      }

      let split_line = line.split(" ").collect::<Vec<&str>>();
      let mut path = match split_line.get(1) {
        Some(s) => *s,
        None => panic!("Invalid include statement"),
      };

      if path.starts_with("builtin") {
        let include = path.trim_start_matches("builtin::");
        match self.builtin.get(include) {
          Some(s) => path = s,
          None => panic!("Could not find {} builtin module", include),
        };
      }

      let path = &format!("{}.{}", path, EXTENSION);

      let mut file = File::open(path).expect(&format!("Could not find {}", path));
      let mut contents = String::new();
      file.read_to_string(&mut contents).expect(&format!("Failed to read file {}", path));
      self.process_lines(contents.lines());

      return true;
    }
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
        p.insert_env("ENABLE_ENTITY", true);
        assert_eq!(
            &p.process(),
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
        p.insert_env("ENABLE_ENTITY", true);
        p.insert_env("ENABLE_TEXTURE", false);
        assert_eq!(
            &p.process(),
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
        p.insert_env("ENABLE_ENTITY", true);
        p.insert_env("ENABLE_TEXTURE", true);
        assert_eq!(
            &p.process(),
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
        p.insert_env("ENABLE_ENTITY", false);
        p.insert_env("ENABLE_TEXTURE", true);
        p.insert_env("BIAS", true);
        assert_eq!(
            &p.process(),
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
        p.insert_env("USE_BIAS", false);
        p.insert_builtin("light", "./assets/builtin/light");
        assert_eq!(
            &p.process(),
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
        p.insert_env("USE_BIAS", true);
        p.insert_builtin("light", "./assets/builtin/light");
        assert_eq!(
            &p.process(),
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
}
