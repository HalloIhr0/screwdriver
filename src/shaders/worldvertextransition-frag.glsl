#version 330 core
in vec2 uvcoord_pass;
in float alpha_pass;
in float light;
out vec4 out_color;
uniform sampler2D basetexture;
uniform sampler2D basetexture2;
void main() {
    vec3 base_color1 = texture(basetexture, uvcoord_pass).rgb;
    vec3 base_color2 = texture(basetexture2, uvcoord_pass).rgb;
    vec3 base_color = mix(base_color1, base_color2, alpha_pass);
    out_color = vec4(base_color*light, 1.0);
}