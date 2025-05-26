use std::num::NonZero;

use super::global_cardinality_lower_upper::Values;
use super::Constraint;
use crate::basic_types::HashMap;
use crate::propagators::gcc_extended_resolution::intersection::GccIntersection;
use crate::propagators::gcc_extended_resolution::transitive::GccTransitive;
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
        // If E_{x,y} = 1, then D'(x) = D'(y) = D(x) ∩ D(y)
        for ((i, j), extended_literal) in &self.equalities {
            let left = self.variables[*i].clone();
            let right = self.variables[*j].clone();

            let intersection = GccIntersection::new(*extended_literal, left, right);
            intersection.post(solver, tag)?;
        }

        // E_{x,y} = 1 and E_{y, z} = 1 => E_{x,z} = 1
        // Naive O(n^3) initialization
        for i in 0..self.variables.len() {
            for j in 0..self.variables.len() {
                for k in 0..self.variables.len() {
                    if self.equalities.contains_key(&(i, j))
                        && self.equalities.contains_key(&(j, k))
                        && self.equalities.contains_key(&(i, k))
                    {
                        let xy = self.equalities[&(i, j)];
                        let yz = self.equalities[&(j, k)];
                        let xz = self.equalities[&(i, k)];

                        let transitive = GccTransitive::new(xy, yz, xz);
                        transitive.post(solver, tag)?;
                    }
                }
            }
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
