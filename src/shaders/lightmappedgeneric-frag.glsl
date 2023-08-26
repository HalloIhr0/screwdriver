#version 330 core
in vec2 uvcoord_pass;
in float light;
out vec4 out_color;
uniform sampler2D basetexture;
void main() {
    vec3 base_color = texture(basetexture, uvcoord_pass).rgb;
    out_color = vec4(base_color*light, 1.0);
}