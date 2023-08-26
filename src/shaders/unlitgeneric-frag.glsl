#version 330 core
in vec2 uvcoord_pass;
out vec4 out_color;
uniform sampler2D basetexture;
void main() {
    out_color = texture(basetexture, uvcoord_pass);
}