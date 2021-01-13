use cgmath::Point3;
use test::{Bencher, black_box};

use super::{Mandelbulb, Shape};

// Some points close to the surface of the mandelbulb.
const POINTS: [Point3<f32>; 20] = [
    Point3::new(-0.73772496, -0.002343091, -0.7382717),
    Point3::new(-0.7484558, -0.8255949, -0.0026540023),
    Point3::new(-1.0951594, -0.0014639703, -0.0027306266),
    Point3::new(-0.60622436, -0.16786861, 0.7227598),
    Point3::new(-0.6000897, -0.5997089, 0.028461732),
    Point3::new(-0.6077231, -0.8336551, -0.004541016),
    Point3::new(-0.05153041, -0.5906257, -0.7647207),
    Point3::new(-0.73772484, -0.0030531297, -0.7382715),
    Point3::new(-1.09658, -0.032518614, 0.026089936),
    Point3::new(-0.74845594, -0.8255949, -0.0033077204),
    Point3::new(-0.0031473506, 0.59545904, 0.7711717),
    Point3::new(0.59178185, -0.009300065, 0.70574695),
    Point3::new(0.5934337, -0.0065053166, -0.8548532),
    Point3::new(0.5906368, 0.5906708, 0.0002929632),
    Point3::new(0.5909915, 0.6001409, -0.4285654),
    Point3::new(-0.004541016, 0.5956404, 0.36293367),
    Point3::new(-0.00073693885, 0.5916996, -0.8447121),
    Point3::new(0.59545904, -0.004541016, 0.35817686),
    Point3::new(0.59545904, -0.004541016, -0.3581769),
    Point3::new(0.60028464, -0.36826742, 0.6579103),

];

// #[bench]
// fn mandel_single_i8_b5(b: &mut Bencher) {
//     let m = Mandelbulb::classic(8, 5.0);
//     let p = black_box(POINTS[0]);
//     b.iter(|| m.min_distance_from(p));
// }

#[bench]
fn mandel_10points_i8_b5(b: &mut Bencher) {
    let m = Mandelbulb::classic(8, 5.0);
    b.iter(|| {
        POINTS.iter()
            .map(|&p| m.min_distance_from(black_box(p)))
            .sum::<f32>()
    });
}
