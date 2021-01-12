#version 450

layout(location = 0) in float in_distance_from_surface;
layout(location = 1) in vec3 in_position;
layout(location = 2) in vec3 in_normal;

layout(location = 0) out vec4 out_color;



float col() {
    return pow(length(in_position) * 0.85, 8.0) * 0.7;
}

void main() {
    vec3 light_dir = vec3(1, 0, 0);
    vec3 light_color = vec3(0.7, 0.6, 0.3);
    float light_strength = 10.0;
    vec3 base_color = vec3(col());

    // 1 means: fully lit, 0 to -1 means: not lit
    float angle = dot(in_normal, -light_dir);

    // The end result
    vec3 color = base_color;

    if (angle > 0) {
        color += base_color * light_strength * light_color * angle;
    }

    out_color = vec4(color, 1);
}
