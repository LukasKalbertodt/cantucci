#version 140

// INCLUDE(DE)

in float dist;
in float z;
in vec3 world_pos;

out vec4 color;

float col() {
    return pow(length(world_pos) * 0.85, 8.0) * 0.7;
}

void main() {
    color = vec4(vec3(col()), 1.0);
}
