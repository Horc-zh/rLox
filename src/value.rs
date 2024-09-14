use crate::{loxcallable::LoxCallable, loxfunction::LoxFunction};
use std::cmp::Ordering;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Nil,
    LoxFunction(LoxFunction),
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::Nil, Value::Nil) => Some(Ordering::Equal),
            // Different types cannot be compared, return None
            (Value::Number(_), _)
            | (Value::Boolean(_), _)
            | (Value::String(_), _)
            | (Value::Nil, _) => None,
            (Value::LoxFunction(_), _) => None,
        }
    }
    // add code here
}

impl Value {
    pub fn is_true(&self) -> bool {
        if let &Value::Boolean(b) = self {
            b
        } else {
            false
        }
    }
}

impl LoxCallable for Value {
    fn call(
        &self,
        interpreter: &mut crate::interpreter::Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, crate::runtime_error::RuntimeError> {
        match self {
            //WARNING: error may occur
            Value::LoxFunction(func) => func.call(interpreter, arguments),
            _ => unreachable!(),
        }
    }

    fn arity(&self) -> usize {
        match self {
            Value::LoxFunction(func) => func.arity(),
            _ => unreachable!(),
        }
    }
}
impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Nil => write!(f, "nil"),
            Value::LoxFunction(func) => write!(f, "{}", func),
        }
    }
}

impl std::ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(n) => Value::Number(-n),
            _ => panic!("Unary negation is only defined for numbers"),
        }
    }
}

impl std::ops::Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l + r),
            (Value::String(mut l), Value::String(r)) => {
                l.push_str(&r);
                Value::String(l)
            }
            _ => panic!("Addition is only defined for two numbers or two strings"),
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l - r),
            _ => panic!("Subtraction is only defined for two numbers"),
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l * r),
            _ => panic!("Multiplication is only defined for two numbers"),
        }
    }
}

impl std::ops::Div for Value {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Value::Number(l / r),
            _ => panic!("Division is only defined for two numbers"),
        }
    }
}

impl std::ops::Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        let b = match self {
            Value::Boolean(b) => b,
            Value::Nil => false,
            _ => true,
        };
        Value::Boolean(!b)
    }
}
