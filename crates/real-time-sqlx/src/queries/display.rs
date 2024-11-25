//! Display unprepared Sqlite queries for debugging purposes

use std::fmt;

use crate::utils::format_list;

use super::serialize::{
    Condition, Constraint, ConstraintValue, FinalType, Operator, OrderBy, PaginateOptions,
    QueryTree,
};

impl fmt::Display for FinalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FinalType::Number(number) => {
                if number.is_f64() {
                    write!(f, "{}", number.as_f64().unwrap())
                } else {
                    write!(f, "{}", number.as_i64().unwrap())
                }
            }
            FinalType::String(string) => write!(f, "'{string}'"),
            FinalType::Bool(bool) => write!(f, "{}", if *bool { 1 } else { 0 }),
            FinalType::Null => write!(f, "NULL"),
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

impl fmt::Display for OrderBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderBy::Asc(column) => write!(f, "ORDER BY {} ASC", column),
            OrderBy::Desc(column) => write!(f, "ORDER BY {} DESC", column),
        }
    }
}

impl fmt::Display for PaginateOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(order) = &self.order_by {
            write!(f, "{} ", order)?;
        }
        write!(f, "LIMIT {} ", self.per_page)?;

        if let Some(offset) = self.offset {
            write!(f, "OFFSET {}", offset)?;
        }

        Ok(())
    }
}

impl fmt::Display for QueryTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SELECT * FROM {}", self.table)?;

        if let Some(condition) = &self.condition {
            write!(f, " WHERE {} ", condition)?;
        }

        if let Some(paginate) = &self.paginate {
            write!(f, "{}", paginate)?;
        }
        Ok(())
    }
}
