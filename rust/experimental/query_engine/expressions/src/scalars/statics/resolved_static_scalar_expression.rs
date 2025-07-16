use crate::{StaticScalarExpression, Value, ValueType};

#[derive(Debug)]
pub enum ResolvedStaticScalarExpression<'a> {
    Reference(&'a StaticScalarExpression),
    Value(StaticScalarExpression),
}

impl ResolvedStaticScalarExpression<'_> {
    pub fn get_value_type(&self) -> ValueType {
        match self {
            ResolvedStaticScalarExpression::Reference(s) => s.get_value_type(),
            ResolvedStaticScalarExpression::Value(s) => s.get_value_type(),
        }
    }

    pub fn to_value(&self) -> Value {
        match self {
            ResolvedStaticScalarExpression::Reference(s) => s.to_value(),
            ResolvedStaticScalarExpression::Value(s) => s.to_value(),
        }
    }
}

#[cfg(test)]
impl AsRef<StaticScalarExpression> for ResolvedStaticScalarExpression<'_> {
    fn as_ref(&self) -> &StaticScalarExpression {
        match self {
            ResolvedStaticScalarExpression::Reference(s) => s,
            ResolvedStaticScalarExpression::Value(s) => s,
        }
    }
}
