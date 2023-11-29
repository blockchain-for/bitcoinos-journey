
use crate::chapter01::*;

pub fn is_curve_point(x: u64, y: u64, prime: u64) -> bool {
    let left = FieldElement::new(y.pow(2) % prime, prime).unwrap();
    let right = FieldElement::new((x.pow(3) + 7) % prime, prime).unwrap();

    left == right
}

#[cfg(test)]
mod tests {

    use super::*;


    #[test]
    fn test_point_should_works() {
        assert!(is_curve_point(192, 105, 223));
        assert!(is_curve_point(17, 56, 223));

        assert!(is_curve_point(1, 193, 223));
        
    }

    #[test]
    #[should_panic]
    fn test_point_should_panics() {
        assert!(is_curve_point(200, 119, 223));

        assert!(is_curve_point(42, 99, 223));
    }
}