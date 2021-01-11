#version 450

// uniform mat4 view_matrix;
// uniform mat4 proj_matrix;

layout(location = 0) in vec3 i_pos;
layout(location = 0) out vec3 o_pos;


void main() {
    o_pos = i_pos;
    gl_Position = vec4(i_pos.xy, 1, 1);
}
