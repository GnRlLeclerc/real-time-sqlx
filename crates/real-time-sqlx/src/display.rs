//! Display unprepared Sqlite queries for debugging purposes

use std::fmt;

use crate::{
    serialize::{
        Condition, Constraint, ConstraintValue, FinalConstraintValue, Operator, QueryTree,
    },
    utils::format_list,
};

impl fmt::Display for FinalConstraintValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FinalConstraintValue::Int(int) => write!(f, "{}", int),
            FinalConstraintValue::String(string) => write!(f, "'{}'", string),
            FinalConstraintValue::Bool(bool) => write!(f, "{}", if *bool { 1 } else { 0 }),
            FinalConstraintValue::Null => write!(f, "NULL"),
        }
    }
}

impl fmt::Display for ConstraintValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstraintValue::Final(value) => write!(f, "{}", value),
            ConstraintValue::List(list) => {
                write!(f, "{}", format_list(&list, ", "))
            }
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Equal => write!(f, "="),
            Operator::LessThan => write!(f, "<"),
            Operator::GreaterThan => write!(f, ">"),
            Operator::LessThanOrEqual => write!(f, "<="),
            Operator::GreaterThanOrEqual => write!(f, ">="),
            Operator::NotEqual => write!(f, "!="),
            Operator::In => write!(f, "in"),
            Operator::Like => write!(f, "like"),
            Operator::ILike => write!(f, "ilike"),
        }
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\" {} {}", self.column, self.operator, self.value)
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Single { constraint } => write!(f, "{}", constraint),
            Condition::Or { conditions } => {
                write!(f, "({})", format_list(&conditions, " OR "))
            }
            Condition::And { conditions } => {
                write!(f, "({})", format_list(&conditions, " AND "))
            }
        }
    }
}

impl fmt::Display for QueryTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SELECT * FROM {}", self.table)?;

        if let Some(condition) = &self.condition {
            write!(f, " WHERE {}", condition)?;
        }
        Ok(())
    }
}
