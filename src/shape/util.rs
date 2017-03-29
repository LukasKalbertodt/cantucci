macro_rules! impl_batch_methods {
    () => {
        fn batch_min_distance_from(&self, points: &[Point3<f32>]) -> Vec<f32> {
            points.iter().map(|&p| self.min_distance_from(p)).collect()
        }

        fn batch_max_distance_from(&self, points: &[Point3<f32>]) -> Vec<f32> {
            points.iter().map(|&p| self.max_distance_from(p).unwrap()).collect()
        }

        fn batch_bounded_distance_from(&self, points: &[Point3<f32>]) -> Vec<(f32, f32)> {
            points.iter().map(|&p| {
                let (min, max) = self.bounded_distance_from(p);
                (min, max.unwrap())
            }).collect()
        }
    }
}
