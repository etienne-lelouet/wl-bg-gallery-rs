#[derive(Debug)]
pub struct Output {
    pub make: String,
    pub model: String,
    pub mode_height: i32,
    pub mode_width: i32,
    pub description: String,
}

impl Output {
    pub fn new() -> Self {
	Self{ make: String::from(""), model: String::from(""), mode_height: 0, mode_width: 0, description: String::from("") }
    }
}
