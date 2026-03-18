pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn double(x: i32) -> i32 {
    add(x, x)
}

pub fn is_even(n: i32) -> bool {
    if n == 0 {
        true
    } else {
        !is_odd(n - 1)
    }
}

pub fn is_odd(n: i32) -> bool {
    if n == 0 {
        false
    } else {
        !is_even(n - 1)
    }
}

pub fn standalone() -> bool {
    true
}
