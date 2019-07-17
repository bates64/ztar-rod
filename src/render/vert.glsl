#version 140

in vec3 xyz;
in vec4 rgba;
out vec4 v_color;
uniform mat4 perspective;

void main() {
    gl_Position = perspective * vec4(xyz, 1.0);
    v_color = rgba / 255.0;
}
