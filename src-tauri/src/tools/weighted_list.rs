use rand::Rng;

pub fn pick_by_weight<T>(list: &[(f32, T)]) -> usize {
    let total_weight = {
        let mut t = 0.0;
        for e in list {
            t += e.0;
        }

        t
    };

    let mut rng = rand::thread_rng();
    let mut weight = rng.gen_range(0.0..total_weight);
    let mut i = 0;
    while weight > list[i].0 {
        i += 1;
        weight -= list[i].0;
    }

    i
}