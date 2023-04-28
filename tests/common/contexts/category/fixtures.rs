use rand::Rng;

pub fn software_predefined_category_name() -> String {
    "software".to_string()
}

pub fn software_predefined_category_id() -> i64 {
    5
}

pub fn random_category_name() -> String {
    format!("category name {}", random_id())
}

fn random_id() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..1_000_000)
}
