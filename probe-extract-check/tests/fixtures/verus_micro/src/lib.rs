use vstd::prelude::*;

verus! {

spec fn is_positive(x: int) -> bool {
    x > 0
}

proof fn positive_sum(a: int, b: int)
    requires
        is_positive(a),
        is_positive(b),
    ensures
        is_positive(a + b),
{
}

exec fn checked_add(a: u32, b: u32) -> (result: u32)
    requires
        is_positive(a as int),
        a as int + b as int < u32::MAX as int,
    ensures
        result == a + b,
{
    a + b
}

exec fn double_checked(x: u32) -> (result: u32)
    requires
        is_positive(x as int),
        2 * (x as int) < u32::MAX as int,
    ensures
        result == 2 * x,
{
    checked_add(x, x)
}

} // verus!
