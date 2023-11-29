mod error;

use std::ops::{Add, Div, Mul, Sub};

pub use error::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FieldElement {
    num: u64,
    prime: u64,
}

impl FieldElement {
    pub fn new(num: u64, prime: u64) -> Result<FieldElement, Error> {
        if num > prime {
            return Err(Error::ValueError { num, prime });
        }

        Ok(Self { num, prime })
    }

    pub fn pow(self, exponent: u32) -> FieldElement {
        let prime = self.prime;
        let num = self.num.pow(exponent) % prime;
        FieldElement { num, prime }
    }
}

impl Add for FieldElement {
    type Output = Self;

    fn add(self, other: FieldElement) -> Self::Output {
        let prime1 = self.prime;
        let prime2 = other.prime;

        if self.prime != other.prime {
            panic!("Can't add with two different FieldElment {prime1} and {prime2}")
        }
        FieldElement::new((self.num + other.num) % prime1, prime1).unwrap()
    }
}

impl Sub for FieldElement {
    type Output = Self;

    fn sub(self, other: FieldElement) -> Self::Output {
        let prime1 = self.prime;
        let prime2 = other.prime;

        if self.prime != other.prime {
            panic!("Can't sub with two different FieldElment {prime1} and {prime2}")
        }
        FieldElement::new((self.num - other.num) % self.prime, self.prime).unwrap()
    }
}

impl Mul for FieldElement {
    type Output = Self;

    fn mul(self, other: FieldElement) -> Self::Output {
        let prime1 = self.prime;
        let prime2 = other.prime;

        if self.prime != other.prime {
            panic!("Can't multiply with two different FieldElment {prime1} and {prime2}")
        }
        FieldElement::new((self.num * other.num) % self.prime, self.prime).unwrap()
    }
}

impl Div for FieldElement {
    type Output = Self;

    fn div(self, other: FieldElement) -> Self::Output {
        let prime1 = self.prime;
        let prime2 = other.prime;

        if self.prime != other.prime {
            panic!("Can't divide with two different FieldElment {prime1} and {prime2}")
        }
        FieldElement::new((self.num / other.num) % self.prime, self.prime).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_element_should_works() {
        let a = FieldElement::new(7, 13).unwrap();
        let b = FieldElement::new(6, 13).unwrap();

        assert_ne!(a, b);
        assert_eq!(a, a);
        assert_eq!(b, b);

        assert_eq!(
            FieldElement::new(95 * 45 * 31 % 97, 97).unwrap(),
            FieldElement::new(23, 97).unwrap()
        );
        assert_eq!(
            FieldElement::new(17 * 13 * 19 * 44 % 97, 97).unwrap(),
            FieldElement::new(68, 97).unwrap()
        );
        // assert_eq!(FieldElement::new(((17_isize.pow(7) * 77_isize.pow(49)) % 97) as i64, 97).unwrap(), FieldElement::new(63, 97).unwrap()); // overflow
    }

    #[test]
    fn file_element_value_error_should_fail() {
        let a = FieldElement::new(17, 13);

        assert_eq!(Error::ValueError { num: 17, prime: 13 }, a.unwrap_err());
        
    }

    #[test]
    fn field_element_add_should_works() {
        let a = FieldElement::new(7, 13).unwrap();
        let b = FieldElement::new(12, 13).unwrap();
        let c = FieldElement::new(6, 13).unwrap();

        let res = a + b;
        assert_eq!(res, c);
    }

    #[test]
    fn field_element_sub_should_works() {
        let a = FieldElement::new(2, 19).unwrap();
        let b = FieldElement::new(11, 19).unwrap();
        let c = FieldElement::new(9, 19).unwrap();

        let res = b - a;
        assert_eq!(res, c);
    }

    #[test]
    fn field_element_mul_should_works() {
        let a = FieldElement::new(3, 13).unwrap();
        let b = FieldElement::new(12, 13).unwrap();
        let c = FieldElement::new(10, 13).unwrap();

        let res = a * b;
        assert_eq!(res, c);

        let a = FieldElement::new(24, 31).unwrap();
        let b = FieldElement::new(19, 31).unwrap();
        let c = FieldElement::new(22, 31).unwrap();

        let res = a * b;
        assert_eq!(res, c);
    }

    #[test]
    fn field_element_div_should_works() {
        let a = FieldElement::new(12, 13).unwrap();
        let b = FieldElement::new(7, 13).unwrap();
        let c = FieldElement::new(1, 13).unwrap();

        let res = a / b;
        assert_eq!(res, c);
    }

    #[test]
    fn field_element_pow_should_works() {
        let a = FieldElement::new(3, 13).unwrap();
        let c = FieldElement::new(1, 13).unwrap();

        let res = a.pow(3);
        assert_eq!(res, c);

        let c = FieldElement::new(17, 31).unwrap();
        let d = FieldElement::new(15, 31).unwrap();
        assert_eq!(c.pow(3), d);
    }
}
