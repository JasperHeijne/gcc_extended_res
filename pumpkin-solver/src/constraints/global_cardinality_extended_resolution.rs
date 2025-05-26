use std::num::NonZero;

use super::global_cardinality_lower_upper::Values;
use super::Constraint;
use crate::basic_types::HashMap;
use crate::propagators::gcc_extended_resolution::intersection::GccIntersection;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

#[allow(dead_code, reason = "still being implemented")]
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
        tag: Option<NonZero<u32>>,
    ) -> Result<(), crate::ConstraintOperationError> {
        for ((i, j), extended_literal) in &self.equalities {
            let left = self.variables[*i].clone();
            let right = self.variables[*j].clone();

            let intersection = GccIntersection::new(*extended_literal, left, right);
            intersection.post(solver, tag)?;
        }

        todo!("full constraint not implemented yet")
    }

    fn implied_by(
        self,
        _solver: &mut crate::Solver,
        _reification_literal: Literal,
        _tag: Option<NonZero<u32>>,
    ) -> Result<(), crate::ConstraintOperationError> {
        todo!("half-reinfication not implemented")
    }
}

pub fn gcc_extended_resolution<Var: IntegerVariable + 'static>(
    variables: impl Into<Box<[Var]>>,
    values: impl Into<Box<[Values]>>,
    equalities: HashMap<(usize, usize), Literal>,
) -> impl Constraint {
    GccExtendedResolution::new(variables.into(), values.into(), equalities)
}
