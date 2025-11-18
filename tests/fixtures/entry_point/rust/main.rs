mod utils;

pub fn main_entry() {
    let data = utils::load_data();
    let result = process_data(data);
    output_result(result);
}

pub fn process_data(data: String) -> String {
    utils::transform_data(data)
}

pub fn output_result(result: String) {
    println!("Result: {}", result);
}

pub fn unused_function() {
    println!("This function is never called");
}
