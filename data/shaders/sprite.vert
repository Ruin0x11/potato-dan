#version 150

in uvec2 position;

in vec2 tex_offset;
in vec3 map_coord;
in vec2 inner_offset;
in vec2 tex_ratio;
in uvec2 sprite_size;

uniform mat4 matrix;
uniform uvec2 tile_size;
uniform vec2 angle;
uniform uint time;
uniform float rotation;

out highp vec2 v_TexCoords;

vec2 sprite_texture(vec2 pos) {
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
  //gl_Position = matrix * vec4(map_coord * tile_size + position * sprite_size + soffset + vec2(4, -24), 0.0, 1.0);
  vec4 pixel_pos = vec4(position * sprite_size * vec2(2.0, 2.0) + inner_offset, 0.0, 1.0) * matrix;
  vec4 world_pos = vec4(map_coord, 1.0) * matrix * rotate_z(rotation) * rotate_x(0.785398);
  gl_Position = pixel_pos + world_pos;
  v_TexCoords = sprite_texture(position);
}
