#version 140

// INCLUDE(DE)

uniform vec3 light_dir;

in float dist;
in float z;
in vec3 world_pos;
in vec3 world_normal;

out vec4 color;


bool ray_march(vec3 to, vec3 dir, out vec3 stop, out int steps) {
    #define ESCAPE_RADIUS 2.0
    #define EPSILON 0.03
    #define MAX_ITER 10

    float error = shape_de(world_pos);
    dir = normalize(dir);
    vec3 p = to + ESCAPE_RADIUS * 2 * dir;
    // start += EPSILON * dir * 400; // FIXME

    for (int i = 0; i < MAX_ITER; i++) {
        float dist = shape_de(p);
        p -= dir * dist;

        if (length(p - to) < 8 * EPSILON - min(error, 0)) {
            break;
        }

        if (dist < EPSILON) {
            stop = p;
            steps = i;
            return true;
        }
    }
    steps = MAX_ITER;
    return false;
}

float col() {
    return pow(length(world_pos) * 0.85, 8.0) * 0.7;
}

void main() {
    vec3 light_color = vec3(0.7, 0.6, 0.3);
    float light_strength = 10.0;
    vec3 base_color = vec3(col());

    // 1 means: fully lit, 0 to -1 means: not lit
    float angle = dot(world_normal, -light_dir);

    // The end result
    vec3 out_color = base_color;

    if (angle > 0) {
        vec3 hit;
        int steps;
        bool shadowed = ray_march(world_pos, -light_dir, hit, steps);
        if (!shadowed) {
            out_color += base_color * light_strength * light_color * angle;
        }
    }

    color = vec4(out_color, 1);
    // FARBE AENDERN
    color = vec4(world_normal/2 + 0.4, 1);
    color = vec4(world_normal/3 + 0.66, 1);
    color = vec4(vec3(abs(dist)), 1.0);




    // vec3 col = vec3(col()) * 0.5;
    // if (!shadowed) {
    //     vec3 new_col = col * vec3(7, 6, 3.0);
    //     col = mix(col, new_col, clamp();
    //     // col = new_col;
    // } else {
    //     // col = hit.bbb + 1.0;
    //     // col = vec3(length(hit - world_pos));
    //     // col = vec3(float(steps)/2);
    //     // col = vec3(shape_de(hit));
    //     // col = vec3(shape_de(world_pos) * 500);
    // }
    // // col = vec3(dot(world_normal, -light_dir));
    // color = vec4(col, 1.0);
    // // color = vec4(world_normal / 2 + 0.5, 1.0);
    // // color = vec4(world_normal, 1.0);
    // // color = vec4(vec3(shape_de(world_pos)*100), 1.0);
}
