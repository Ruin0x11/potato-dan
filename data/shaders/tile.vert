#version 150

in uvec2 position;
in vec2 tex_offset;
in vec2 map_coord;

uniform mat4 matrix;
uniform vec2 tex_ratio;
uniform vec2 camera;
uniform float rotation;

out highp vec2 v_TexCoords;

vec2 normal_tile(vec2 pos) {
  float u = pos.x * tex_ratio.x + tex_offset.x;
  float v = 1.0 - (pos.y * tex_ratio.y + tex_offset.y);
  return vec2(u, v);
}

mat4 rotate_x(float theta)
{
    return mat4(
        vec4(1.0,         0.0,         0.0, 0.0),
        vec4(0.0,  cos(theta),  sin(theta), 0.0),
        vec4(0.0, -sin(theta),  cos(theta), 0.0),
        vec4(0.0,         0.0,         0.0, 1.0)
    );
}

mat4 rotate_z(float theta)
{
    return mat4(
        vec4( cos(theta), sin(theta), 0.0, 0.0),
        vec4(-sin(theta), cos(theta), 0.0, 0.0),
        vec4(0.0,                0.0, 0.0, 0.0),
        vec4(0.0,                0.0, 0.0, 1.0)
    );
}

void main() {
  gl_Position = vec4(map_coord + position - camera, 0.0, 1.0) * matrix * rotate_z(rotation) * rotate_x(0.785398);
  v_TexCoords = normal_tile(position);
}
