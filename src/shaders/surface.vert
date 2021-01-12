#version 450

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in float in_distance_from_surface;

layout(location = 0) out float out_distance_from_surface;
layout(location = 1) out vec3 out_position;
layout(location = 2) out vec3 out_normal;

layout(push_constant) uniform PushConsts {
  mat4 trans_matrix;
} uniforms;


void main() {
    out_distance_from_surface = in_distance_from_surface;
    out_position = in_position;
    out_normal = in_normal;

    gl_Position = uniforms.trans_matrix * vec4(in_position, 1);
}
