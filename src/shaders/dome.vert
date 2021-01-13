#version 450

layout(location = 0) in vec3 i_pos;
layout(location = 0) out vec3 o_pos;

layout(push_constant) uniform PushConsts {
  mat4 trans_matrix;
} uniforms;


void main() {
    o_pos = i_pos;
    gl_Position = uniforms.trans_matrix * vec4(i_pos, 1);
}
