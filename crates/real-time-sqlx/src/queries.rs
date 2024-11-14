//! Query system for real-time SQLX

use serialize::{Condition, Constraint, ConstraintValue, FinalType, Operator, QueryTree};

use crate::{
    operations::serialize::JsonObject,
    utils::{sql_ilike, sql_like},
};

pub mod display;
pub mod serialize;

// ************************************************************************* //
//                        QUERY SYSTEM IMPLEMENTATION                        //
// ************************************************************************* //

/// Comparing 2 final types
impl FinalType {
    /// Compare self (left side) with another final type (right side) using an operator
    pub fn compare(&self, other: &FinalType, operator: &Operator) -> bool {
        match operator {
            Operator::Equal => self.equals(other),
            Operator::LessThan => self.less_than(other),
            Operator::GreaterThan => self.greater_than(other),
            Operator::LessThanOrEqual => self.less_than(other) || self.equals(other),
            Operator::GreaterThanOrEqual => self.greater_than(other) || self.equals(other),
            Operator::NotEqual => !self.equals(other),
            Operator::Like => match (self, other) {
                (FinalType::String(s), FinalType::String(t)) => sql_like(t, s),
                _ => false,
            },
            Operator::ILike => match (self, other) {
                (FinalType::String(s), FinalType::String(t)) => sql_ilike(t, s),
                _ => false,
            },
            _ => panic!("Invalid operator {} for comparison", operator),
        }
    }

    // Particularized implementations

    /// &self == other
    pub fn equals(&self, other: &FinalType) -> bool {
        match (self, other) {
            (FinalType::Number(n), FinalType::Number(m)) => {
                if n.is_f64() && m.is_f64() {
                    n.as_f64().unwrap() == m.as_f64().unwrap()
                } else if n.is_i64() && m.is_i64() {
                    n.as_i64().unwrap() == m.as_i64().unwrap()
                } else {
                    false
                }
            }
            (FinalType::String(s), FinalType::String(t)) => s == t,
            (FinalType::Bool(b), FinalType::Bool(c)) => b == c,
            (FinalType::Null, FinalType::Null) => true,
            _ => false,
        }
    }

    /// &self < other
    pub fn less_than(&self, other: &FinalType) -> bool {
        match (self, other) {
            (FinalType::Number(n), FinalType::Number(m)) => {
                if n.is_f64() && m.is_f64() {
                    n.as_f64().unwrap() < m.as_f64().unwrap()
                } else if n.is_i64() && m.is_i64() {
                    n.as_i64().unwrap() < m.as_i64().unwrap()
                } else {
                    false
                }
            }
            (FinalType::String(s), FinalType::String(t)) => s < t,
            (FinalType::Bool(b), FinalType::Bool(c)) => b < c,
            _ => false,
        }
    }

    /// &self > other
    pub fn greater_than(&self, other: &FinalType) -> bool {
        match (self, other) {
            (FinalType::Number(n), FinalType::Number(m)) => {
                if n.is_f64() && m.is_f64() {
                    n.as_f64().unwrap() > m.as_f64().unwrap()
                } else if n.is_i64() && m.is_i64() {
                    n.as_i64().unwrap() > m.as_i64().unwrap()
                } else {
                    false
                }
            }
            (FinalType::String(s), FinalType::String(t)) => s > t,
            (FinalType::Bool(b), FinalType::Bool(c)) => b > c,
            _ => false,
        }
    }

    /// &self <= other
    pub fn less_than_or_equal(&self, other: &FinalType) -> bool {
        self.less_than(other) || self.equals(other)
    }

    /// &self >= other
    pub fn greater_than_or_equal(&self, other: &FinalType) -> bool {
        self.greater_than(other) || self.equals(other)
    }
}

impl ConstraintValue {
    /// Compare a constraint value with a final type (a constraint value can be a list of final types)
    /// NOTE : assume that the ConstraintValue is always on the right side of the comparison
    /// (for instance with the operator IN)
    pub fn compare(&self, other: &FinalType, operator: &Operator) -> bool {
        match self {
            ConstraintValue::Final(final_type) => final_type.compare(other, operator),
            ConstraintValue::List(list) => match operator {
                Operator::In => {
                    for value in list {
                        if value.compare(other, &Operator::Equal) {
                            return true;
                        }
                    }
                    false
                }
                _ => panic!("Invalid operator {} for list comparison", operator),
            },
        }
    }
}

// ************************************************************************* //
//                       CHECKS AGAINST JSON OBJECT                          //
// ************************************************************************* //

pub trait Checkable {
    fn check(&self, object: &JsonObject) -> bool;
}

impl Checkable for Constraint {
    /// Check if a constraint is satisfied by a JSON object
    fn check(&self, object: &JsonObject) -> bool {
        let value = object
            .get(&self.column)
            .expect("Column not found in JSON object");

        let final_type = FinalType::try_from(value.clone())
            .expect(format!("Incompatible value for column: {value}").as_str());

        self.value.compare(&final_type, &self.operator)
    }
}

impl Checkable for Condition {
    /// Check if a condition is satisfied by a JSON object
    fn check(&self, object: &JsonObject) -> bool {
        match self {
            Condition::Single { constraint } => constraint.check(object),
            Condition::And { conditions } => {
                for condition in conditions {
                    if !condition.check(object) {
                        return false;
                    }
                }
                true
            }
            Condition::Or { conditions } => {
                for condition in conditions {
                    if condition.check(object) {
                        return true;
                    }
                }
                false
            }
        }
    }
}

impl Checkable for QueryTree {
    /// Check if a query is satisfied by a JSON object
    fn check(&self, object: &JsonObject) -> bool {
        if let Some(condition) = &self.condition {
            condition.check(object)
        } else {
            true
        }
    }
}
