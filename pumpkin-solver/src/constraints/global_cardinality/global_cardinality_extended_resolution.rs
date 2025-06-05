use std::num::NonZero;

use super::global_cardinality_lower_upper::Values;
use super::Constraint;
use crate::basic_types::HashMap;
use crate::propagators::gcc_extended_resolution::equality::GccEquality;
use crate::propagators::gcc_extended_resolution::exclusion::GccExclusion;
use crate::propagators::gcc_extended_resolution::inequality::GccInequality;
use crate::propagators::gcc_extended_resolution::intersection::GccIntersection;
use crate::propagators::gcc_extended_resolution::transitive::GccTransitive;
use crate::propagators::gcc_extended_resolution::upper_bound::GccUpperBound;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

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
            let x = self.variables[*i].clone();
            let y = self.variables[*j].clone();

            GccIntersection::new(*extended_literal, x.clone(), y.clone()).post(solver, tag)?;
            GccIntersection::new(*extended_literal, y, x).post(solver, tag)?;
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

        // x = v and y = v => E_{x,y} = 1
        for ((i, j), extended_literal) in &self.equalities {
            let left = self.variables[*i].clone();
            let right = self.variables[*j].clone();

            let equality = GccEquality::new(left, right, *extended_literal);
            equality.post(solver, tag)?;
        }

        // E_{x,y} = 0 and x = v => y != v
        for ((i, j), e_xy) in &self.equalities {
            let x = self.variables[*i].clone();
            let y = self.variables[*j].clone();

            GccExclusion::new(*e_xy, x.clone(), y.clone()).post(solver, tag)?;
            GccExclusion::new(*e_xy, y, x).post(solver, tag)?;
        }

        // D(x) ∩ D(y) = {} => E_{x, y} = 0
        for ((i, j), e_xy) in &self.equalities {
            let x = self.variables[*i].clone();
            let y = self.variables[*j].clone();

            GccInequality::new(x, y, *e_xy).post(solver, tag)?;
        }

        // GCC upper-bounds
        GccUpperBound::new(
            self.variables.clone(),
            self.values.clone(),
            self.equalities.clone(),
        )
        .post(solver, tag)?;

        Ok(())
    }

    fn implied_by(
        self,
        _solver: &mut crate::Solver,
        _reification_literal: Literal,
        _tag: Option<NonZero<u32>>,
    ) -> Result<(), crate::ConstraintOperationError> {
        todo!("half-reification not implemented")
    }
}

pub fn gcc_extended_resolution<Var: IntegerVariable + 'static>(
    variables: impl Into<Box<[Var]>>,
    values: impl Into<Box<[Values]>>,
    equalities: HashMap<(usize, usize), Literal>,
) -> impl Constraint {
    GccExtendedResolution::new(variables.into(), values.into(), equalities)
}
