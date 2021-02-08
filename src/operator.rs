use std::fmt;
use num::{rational::Rational64, traits::CheckedDiv};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

pub const OPERATORS: [Operator; 4] = [
    Operator::Add,
    Operator::Sub,
    Operator::Mul,
    Operator::Div,
];

impl Operator {
    #[inline(always)]
    pub fn invoke(&self, x: Rational64, y: Rational64) -> Option<Rational64> {
        match self {
            &Operator::Add => Some(x + y),
            &Operator::Sub => Some(x - y),
            &Operator::Mul => Some(x * y),
            &Operator::Div => {
                x.checked_div(&y)
            },
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Operator::Add => write!(f, "+"),
            &Operator::Sub => write!(f, "-"),
            &Operator::Mul => write!(f, "*"),
            &Operator::Div => write!(f, "/"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn invoke() {
        assert_eq!(Operator::Add.invoke(
            Rational64::new(1, 3),
            Rational64::new(1, 2),
        ), Some(Rational64::new(5, 6)));
        assert_eq!(Operator::Sub.invoke(
            Rational64::new(1, 3),
            Rational64::new(1, 2),
        ), Some(Rational64::new(-1, 6)));
        assert_eq!(Operator::Mul.invoke(
            Rational64::new(1, 3),
            Rational64::new(1, 2),
        ), Some(Rational64::new(1, 6)));
        assert_eq!(Operator::Div.invoke(
            Rational64::new(1, 3),
            Rational64::new(1, 2),
        ), Some(Rational64::new(2, 3)));
        assert_eq!(Operator::Div.invoke(
            Rational64::new(1, 3),
            Rational64::new(0, 1),
        ), None);
    }

    #[test]
    fn fmt() {
        assert_eq!(Operator::Add.to_string(), "+".to_string());
        assert_eq!(Operator::Sub.to_string(), "-".to_string());
        assert_eq!(Operator::Mul.to_string(), "*".to_string());
        assert_eq!(Operator::Div.to_string(), "/".to_string());
    }
}