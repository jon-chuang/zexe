mod g1 {
    use algebra::{
        sw6::{Fr, G1Affine, G1Projective as G1},
        ProjectiveCurve, UniformRand,
    };
    use core::ops::AddAssign;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

    #[bench]
    fn bench_g1_rand(b: &mut ::test::Bencher) {
        let mut rng = XorShiftRng::seed_from_u64(1231275789u64);
        b.iter(|| G1::rand(&mut rng));
    }

    #[bench]
    fn bench_g1_mul_assign(b: &mut ::test::Bencher) {
        const SAMPLES: usize = 1000;

        let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

        let v: Vec<(G1, Fr)> = (0..SAMPLES)
            .map(|_| (G1::rand(&mut rng), Fr::rand(&mut rng)))
            .collect();

        let mut count = 0;
        b.iter(|| {
            let mut tmp = v[count].0;
            tmp *= v[count].1;
            count = (count + 1) % SAMPLES;
            tmp
        });
    }

    #[bench]
    fn bench_g1_add_assign(b: &mut ::test::Bencher) {
        const SAMPLES: usize = 1000;

        let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

        let v: Vec<(G1, G1)> = (0..SAMPLES)
            .map(|_| (G1::rand(&mut rng), G1::rand(&mut rng)))
            .collect();

        let mut count = 0;
        b.iter(|| {
            let mut tmp = v[count].0;
            for i in 0..1000 { tmp.add_assign(&v[i].1); }
            count = (count + 1) % SAMPLES;
            tmp
        });
    }

    #[bench]
    fn bench_g1_add_assign_mixed(b: &mut ::test::Bencher) {
        const SAMPLES: usize = 1000;

        let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

        let v: Vec<(G1, G1Affine)> = (0..SAMPLES)
            .map(|_| (G1::rand(&mut rng), G1::rand(&mut rng).into()))
            .collect();

        let mut count = 0;
        b.iter(|| {
            let mut tmp = v[count].0;
            for i in 0..1000 { tmp.add_assign_mixed(&v[i].1); }
            count = (count + 1) % SAMPLES;
            tmp
        });
    }

    #[bench]
    fn bench_g1_double(b: &mut ::test::Bencher) {
        const SAMPLES: usize = 1000;

        let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

        let v: Vec<(G1, G1)> = (0..SAMPLES)
            .map(|_| (G1::rand(&mut rng), G1::rand(&mut rng)))
            .collect();

        let mut count = 0;
        b.iter(|| {
            let mut tmp = v[count].0;
            for i in 0..1000 { tmp.double_in_place(); }
            count = (count + 1) % SAMPLES;
            tmp
        });
    }
}

mod g2 {
    use algebra::{
        bls12_377::{Fr, G2Affine, G2Projective as G2},
        ProjectiveCurve, UniformRand,
    };
    use core::ops::AddAssign;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

    #[bench]
    fn bench_g2_rand(b: &mut ::test::Bencher) {
        let mut rng = XorShiftRng::seed_from_u64(1231275789u64);
        b.iter(|| G2::rand(&mut rng));
    }

    #[bench]
    fn bench_g2_mul_assign(b: &mut ::test::Bencher) {
        const SAMPLES: usize = 1000;

        let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

        let v: Vec<(G2, Fr)> = (0..SAMPLES)
            .map(|_| (G2::rand(&mut rng), Fr::rand(&mut rng)))
            .collect();

        let mut count = 0;
        b.iter(|| {
            let mut tmp = v[count].0;
            tmp *= v[count].1;
            count = (count + 1) % SAMPLES;
            tmp
        });
    }

    #[bench]
    fn bench_g2_add_assign(b: &mut ::test::Bencher) {
        const SAMPLES: usize = 1000;

        let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

        let v: Vec<(G2, G2)> = (0..SAMPLES)
            .map(|_| (G2::rand(&mut rng), G2::rand(&mut rng)))
            .collect();

        let mut count = 0;
        b.iter(|| {
            let mut tmp = v[count].0;
            tmp.add_assign(&v[count].1);
            count = (count + 1) % SAMPLES;
            tmp
        });
    }

    #[bench]
    fn bench_g2_add_assign_mixed(b: &mut ::test::Bencher) {
        const SAMPLES: usize = 1000;

        let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

        let v: Vec<(G2, G2Affine)> = (0..SAMPLES)
            .map(|_| (G2::rand(&mut rng), G2::rand(&mut rng).into()))
            .collect();

        let mut count = 0;
        b.iter(|| {
            let mut tmp = v[count].0;
            tmp.add_assign_mixed(&v[count].1);
            count = (count + 1) % SAMPLES;
            tmp
        });
    }

    #[bench]
    fn bench_g2_double(b: &mut ::test::Bencher) {
        const SAMPLES: usize = 1000;

        let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

        let v: Vec<(G2, G2)> = (0..SAMPLES)
            .map(|_| (G2::rand(&mut rng), G2::rand(&mut rng)))
            .collect();

        let mut count = 0;
        b.iter(|| {
            let mut tmp = v[count].0;
            tmp.double_in_place();
            count = (count + 1) % SAMPLES;
            tmp
        });
    }
}
