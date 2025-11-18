pub fn load_data() -> String {
    fetch_from_database()
}

pub fn transform_data(data: String) -> String {
    let cleaned = clean_data(data);
    validate_data(cleaned)
}

fn fetch_from_database() -> String {
    "raw data from db".to_string()
}

fn clean_data(data: String) -> String {
    data.trim().to_string()
}

fn validate_data(data: String) -> String {
    format!("validated: {}", data)
}

pub fn another_unused() {
    println!("Also never called");
}
