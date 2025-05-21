use crate::variables::IntegerVariable;

use super::global_cardinality_lower_upper::Values;

struct GccExtendedResolution<Variable: IntegerVariable + 'static> {
    variables: Box<[Variable]>,
    values: Box<[Values]>,
}
