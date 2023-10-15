use rand::Rng;

pub fn random_tag_name() -> String {
    format!("tag name {}", random_id())
}

fn random_id() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..1_000_000)
}
