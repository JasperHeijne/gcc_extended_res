use crate::{
    basic_types::HashMap,
    variables::{IntegerVariable, Literal},
};

use super::{global_cardinality_lower_upper::Values, Constraint};

struct GccExtendedResolution<Var: IntegerVariable + 'static> {
    variables: Box<[Var]>,
    values: Box<[Values]>,
    equalities: HashMap<(usize, usize), Literal>,
}

impl<Var: IntegerVariable> GccExtendedResolution<Var> {
    fn new(
        variables: impl IntoIterator<Item = Var>,
        values: impl IntoIterator<Item = Values>,
        equalities: HashMap<(usize, usize), Literal>,
    ) -> Self {
        Self {
            variables: variables.into_iter().collect(),
            values: values.into_iter().collect(),
            equalities,
        }
    }
}

impl<Var: IntegerVariable> Constraint for GccExtendedResolution<Var> {
    fn post(
        self,
        solver: &mut crate::Solver,
        tag: Option<std::num::NonZero<u32>>,
    ) -> Result<(), crate::ConstraintOperationError> {
        todo!()
    }

    fn implied_by(
        self,
        solver: &mut crate::Solver,
        reification_literal: Literal,
        tag: Option<std::num::NonZero<u32>>,
    ) -> Result<(), crate::ConstraintOperationError> {
        todo!()
    }
}

pub fn gcc_extended_resolution<Var: IntegerVariable + 'static>(
    variables: impl Into<Box<[Var]>>,
    values: impl Into<Box<[Values]>>,
    equalities: HashMap<(usize, usize), Literal>,
) -> impl Constraint {
    GccExtendedResolution::new(variables.into(), values.into(), equalities)
}
