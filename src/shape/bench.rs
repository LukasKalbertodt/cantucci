use test::{Bencher, black_box};

use super::{BENCH_POINTS, Mandelbulb, Shape};


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
        BENCH_POINTS.iter()
            .map(|&p| m.min_distance_from(black_box(p.into())))
            .sum::<f32>()
    });
}
