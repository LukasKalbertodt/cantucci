#version 400

// uniform mat4 world_matrix;
uniform mat4 view_matrix;
uniform mat4 proj_matrix;
uniform vec3 cube_start;
uniform vec3 cube_end;

in vec3 pos;
// in vec2 unit_pos;


void main() {
    vec3 world_pos = cube_start + pos * (cube_end - cube_start);
    gl_Position = vec4(
        proj_matrix
            * view_matrix
            // * world_matrix
            * vec4(world_pos, 1.0)
    );
}
