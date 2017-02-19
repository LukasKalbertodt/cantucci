#version 400
uniform mat4 view_matrix;
uniform mat4 proj_matrix;

out vec3 world_pos;

in vec3 pos;

void main() {
    world_pos = pos;

    gl_Position = vec4(
        proj_matrix
            * view_matrix
            * vec4(pos, 1.0)
    );
}
