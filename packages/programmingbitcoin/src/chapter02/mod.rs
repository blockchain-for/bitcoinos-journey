
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Point {
    x: i64,
    y: i64,
    a: i64, 
    b: i64, 
}

impl Point {
    pub fn new(x: i64, y: i64, a: i64, b: i64) -> Self {
        if y.pow(2) != x.pow(3) + a * x + b {
            panic!("{}, {} is not one the curve", x, y)
        }

        Self { x, y, a, b }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn point_new_should_works() {
        let p = Point::new(-1, -1, 5, 7);
        assert_eq!(p.a, 5);

        let _p = Point::new(18, 77, 5, 7);
    }

    #[test]
    #[should_panic]
    fn point_new_should_panic() {
        let _p = Point::new(-1, -2, 5, 7);
    }

    #[test]
    #[should_panic]
    fn point2_new_should_panic() {
        let _p = Point::new(2, 4, 5, 7);
    }

    #[test]
    #[should_panic]
    fn point3_new_should_panic() {
        let _p = Point::new(5, 7, 5, 7);
    }

    #[test] 
    #[should_panic]
    fn curve1_should_panic() {
        let _p = Point::new(192, 105, 0, 7);
    }

    #[test] 
    #[should_panic]
    fn curve_should_panic() {
        let _p = Point::new(17, 56, 0, 7);
    }

    #[test] 
    #[should_panic]
    fn curve3_should_panic() {
        let _p = Point::new(200, 119, 0, 7);
    }

    #[test] 
    #[should_panic]
    fn curve4_should_panic() {
        let _p = Point::new(1, 193, 0, 7);
    }

    #[test] 
    #[should_panic]
    fn curve5_should_panic() {
        let _p = Point::new(42, 99, 0, 7);
    }
}
