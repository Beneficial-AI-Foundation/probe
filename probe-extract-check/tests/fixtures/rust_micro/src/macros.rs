macro_rules! make_converter {
    ($name:ident, $from:ty, $to:ty) => {
        pub fn $name(val: $from) -> $to {
            val as $to
        }
    };
}

make_converter!(to_u64, u32, u64);
make_converter!(to_i64, i32, i64);

pub fn convert_both(a: u32, b: i32) -> (u64, i64) {
    (to_u64(a), to_i64(b))
}
