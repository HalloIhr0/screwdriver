#version 330 core
layout (location=0) in vec3 pos;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 uvcoord;
layout (location=3) in float alpha;

out vec2 uvcoord_pass;
out float light;
uniform mat4 view;
uniform mat4 projection;
uniform mat3 normal_transform;
uniform vec2 tex_size;
void main() {
    vec3 view_dir = vec3(0, 0, 1);
    light = clamp(dot(normalize(normal_transform * normal), view_dir), 0.0, 1.0)*0.8 + 0.2;
    // remap 1hu to 1 pixel
    uvcoord_pass = uvcoord/tex_size;
    gl_Position = projection*(view*vec4(pos, 1.0));
}