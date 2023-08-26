#version 330 core
layout (location=0) in vec3 pos;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 uvcoord;
out vec2 uvcoord_pass;
uniform mat4 view;
uniform mat4 projection;
uniform vec2 tex_size;
void main() {
    // remap 1hu to 1 pixel
    uvcoord_pass = uvcoord/tex_size;
    gl_Position = projection*(view*vec4(pos, 1.0));
}