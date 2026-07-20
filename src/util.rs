use std::sync::LazyLock;

use macroquad::prelude::*;

const DEFAULT_VERTEX: &str = r#"#version 100
precision highp float;

attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;

const GRID_FRAGMENT: &str = include_str!("grid.frag");

pub static GRID_MATERIAL: LazyLock<Material> = LazyLock::new(|| {
    load_material(
        ShaderSource::Glsl {
            vertex: DEFAULT_VERTEX,
            fragment: GRID_FRAGMENT,
        },
        MaterialParams::default(),
    )
    .unwrap()
});
