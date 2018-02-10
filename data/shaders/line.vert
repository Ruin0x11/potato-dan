#version 150

in vec3 position;

uniform mat4 matrix;
uniform uint time;
uniform float rotation;

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

void main()
{
  gl_Position = vec4(position * vec3(0.5, 0.5, 0.5), 1) * matrix * rotate_z(rotation) * rotate_x(0.785398);
}
