struct Calculator {
    value: i32,
}

impl Calculator {
    fn new() -> Self {
        Calculator { value: 0 }
    }

    fn add(&mut self, x: i32) {
        self.validate();
        self.value += x;
    }
    
    fn validate(&self) {
        // validation logic
    }
}

struct Logger;

impl Logger {
    fn new() -> Self {
        Logger
    }
}
