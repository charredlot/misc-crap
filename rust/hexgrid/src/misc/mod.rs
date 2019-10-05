pub fn gcd32(a: i32, b: i32) -> i32 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
       let tmp = b;
       b = a % b;
       a = tmp;
    }
    a.abs()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd32() {
        assert_eq!(gcd32(0, 0), 0);
        assert_eq!(gcd32(0, 3), 3);
        assert_eq!(gcd32(5, 0), 5);
        assert_eq!(gcd32(-2, 4), 2);
        assert_eq!(gcd32(5, 15), 5);
        assert_eq!(gcd32(15, 6), 3);
        assert_eq!(gcd32(11, 17), 1);
        assert_eq!(gcd32(9, 28), 1);
    }
}
