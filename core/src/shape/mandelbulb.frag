float shape_de(vec3 point) {
    vec3 z = point;
    float dr = 1.0;
    float r = 0.0;

    for (int i = 0; i < {MAX_ITERS}; i++) {
        r = length(z);
        if (r > {BAILOUT} /*|| (1.0 / r).is_infinite()*/) {
            break;
        }

        // convert to polar coordinates
        float theta = acos(z.z / r);
        float phi = atan(z.y, z.x);
        dr = pow(r, {POWER} - 1.0) * {POWER} * dr + 1.0;

        // scale and rotate the point
        float zr = pow(r, {POWER});
        theta = theta * {POWER};
        phi = phi * {POWER};

        // convert back to cartesian coordinates
        z = zr * vec3(
            sin(theta) * cos(phi),
            sin(phi) * sin(theta),
            cos(theta)
        );
        z = z + point;
    }

    // let ln_r = if r.ln().is_infinite() { 0.0 } else { r.ln() * r };
    float ln_r = log(r) * r;
    // if (ln_r == 1.0/0.0) {
    //     ln_r = 0;
    // }
    float lower = 0.5 * ln_r / dr;

    return lower;
}
