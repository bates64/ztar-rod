#version 140

in vec4 v_color;
in vec2 v_tex_coords;
out vec4 color;
uniform sampler2D tex;

void main() {
    color = v_color * texture(tex, v_tex_coords / textureSize(tex, 0));
}
