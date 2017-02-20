#version 400

uniform mat4 world_matrix;
uniform mat4 view_matrix;
uniform mat4 proj_matrix;

in vec3 pos;
in vec2 unit_pos;

out vec2 x_unit_pos;

void main() {
    x_unit_pos = unit_pos;

    gl_Position = vec4(
        proj_matrix
            * view_matrix
            * world_matrix
            * vec4(pos, 1.0)
    );
}
