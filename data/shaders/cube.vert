in vec3 position;
in vec3 scale;
in vec3 offset;

uniform mat4 matrix;
uniform uint time;

mat4 rotate_x(float theta)
{
    return mat4(
        vec4(1.0,         0.0,         0.0, 0.0),
        vec4(0.0,  cos(theta),  sin(theta), 0.0),
        vec4(0.0, -sin(theta),  cos(theta), 0.0),
        vec4(0.0,         0.0,         0.0, 1.0)
    );
}

void main()
{
  gl_Position = vec4(position * scale * vec3(0.5, 0.5, 0.5) + offset, 1) * matrix * rotate_x(0.785398);
}
