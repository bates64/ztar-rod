#version 140

in vec3 xyz;
in vec4 rgba;
in vec2 uv;
out vec4 v_color;
out vec2 v_tex_coords;
uniform mat4 perspective;

void main() {
    gl_Position = perspective * vec4(xyz, 1.0);
    v_color = rgba / 255.0;
    v_tex_coords = uv / 32.0;
}
