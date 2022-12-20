pub fn div_with_remainder(dividend: f32, divisor: f32) -> (f32, f32) {
    let quotient = dividend / divisor;
    let remainder = dividend % divisor;
    (quotient, remainder)
}