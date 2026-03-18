pub mod macros;
pub mod math;
pub mod shapes;
pub mod types_only;

pub fn greet(name: &str) -> String {
    format!("Hello, {}!", capitalize(name))
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
