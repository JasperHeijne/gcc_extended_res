use super::global_cardinality_lower_upper::Values;
use super::Constraint;
use crate::basic_types::HashMap;
use crate::propagators::gcc_extended_resolution::conflicts::GccConflicts;
use crate::propagators::gcc_extended_resolution::equality::GccEquality;
use crate::propagators::gcc_extended_resolution::exclusion::GccExclusion;
use crate::propagators::gcc_extended_resolution::inequality::GccInequality;
use crate::propagators::gcc_extended_resolution::inequality_sets::GccInequalitySets;
use crate::propagators::gcc_extended_resolution::intersection::GccIntersection;
use crate::propagators::gcc_extended_resolution::transitive::GccTransitive;
use crate::propagators::gcc_extended_resolution::upper_bound::GccUpperBound;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

struct GccExtendedResolution<Var: IntegerVariable + 'static> {
    intersections: Vec<GccIntersection<Var>>,
    transitives: Vec<GccTransitive>,
    equality_constraints: Vec<GccEquality<Var>>,
    exclusions: Vec<GccExclusion<Var>>,
    inequalities: Vec<GccInequality<Var>>,
    inequality_sets: GccInequalitySets<Var>,
    conflicts: Vec<GccConflicts<Var>>,
    upper_bound: Option<GccUpperBound<Var>>,
}

impl<Var: IntegerVariable + 'static> GccExtendedResolution<Var> {
    fn new(
        variables: impl IntoIterator<Item = Var>,
        values: impl IntoIterator<Item = Values>,
        equalities: HashMap<(usize, usize), Literal>,
    ) -> Self {
        let variables: Vec<Var> = variables.into_iter().collect();
        let values: Vec<Values> = values.into_iter().collect();

        // If E_{x,y} = 1, then D'(x) = D'(y) = D(x) ∩ D(y)
        let mut intersections: Vec<GccIntersection<Var>> = Vec::new();
        for ((i, j), extended_literal) in &equalities {
            let x = variables[*i].clone();
            let y = variables[*j].clone();

            let intersection_xy = GccIntersection::new(*extended_literal, x.clone(), y.clone());
            let intersection_yx = GccIntersection::new(*extended_literal, y, x);

            intersections.push(intersection_xy);
            intersections.push(intersection_yx);
        }

        // E_{x,y} = 1 and E_{y, z} = 1 => E_{x,z} = 1
        // Naive O(n^3) initialization
        let mut transitives: Vec<GccTransitive> = Vec::new();
        for i in 0..variables.len() {
            for j in 0..variables.len() {
                for k in 0..variables.len() {
                    if equalities.contains_key(&(i, j))
                        && equalities.contains_key(&(j, k))
                        && equalities.contains_key(&(i, k))
                    {
                        let xy = equalities[&(i, j)];
                        let yz = equalities[&(j, k)];
                        let xz = equalities[&(i, k)];

                        let transitive = GccTransitive::new(xy, yz, xz);
                        transitives.push(transitive);
                    }
                }
            }
        }

        // x = v and y = v => E_{x,y} = 1
        let mut equality_constraints: Vec<GccEquality<Var>> = Vec::new();
        for ((i, j), extended_literal) in &equalities {
            let left = variables[*i].clone();
            let right = variables[*j].clone();

            let equality = GccEquality::new(left, right, *extended_literal);
            equality_constraints.push(equality);
        }

        // E_{x,y} = 0 and x = v => y != v
        let mut exclusions: Vec<GccExclusion<Var>> = Vec::new();
        for ((i, j), e_xy) in &equalities {
            let x = variables[*i].clone();
            let y = variables[*j].clone();

            let exclusion_xy = GccExclusion::new(*e_xy, x.clone(), y.clone());
            let exclusion_yx = GccExclusion::new(*e_xy, y, x);
            exclusions.push(exclusion_xy);
            exclusions.push(exclusion_yx);
        }

        // D(x) ∩ D(y) = {} => E_{x, y} = 0
        let mut inequalities: Vec<GccInequality<Var>> = Vec::new();
        for ((i, j), e_xy) in &equalities {
            let x = variables[*i].clone();
            let y = variables[*j].clone();

            let inequality = GccInequality::new(x, y, *e_xy);

            inequalities.push(inequality);
        }

        // GCC inequality sets (greedy cliques)
        let inequality_sets: GccInequalitySets<Var> =
            GccInequalitySets::new(variables.clone(), equalities.clone());

        // GCC upper-bounds
        let upper_bound: GccUpperBound<Var> =
            GccUpperBound::new(variables.clone(), values.clone(), equalities.clone());

        let upper_bound = Some(upper_bound);

        let conflicts: Vec<GccConflicts<Var>> = values
            .iter()
            .map(|value| {
                GccConflicts::new(
                    variables.clone(),
                    value.value,
                    value.omin as usize,
                    value.omax as usize,
                )
            })
            .collect();

        Self {
            intersections,
            transitives,
            equality_constraints,
            exclusions,
            inequalities,
            inequality_sets,
            conflicts,
            upper_bound,
        }
    }
}

pub fn gcc_extended_resolution<Var: IntegerVariable + 'static>(
    variables: impl Into<Box<[Var]>>,
    values: impl Into<Box<[Values]>>,
    equalities: HashMap<(usize, usize), Literal>,
) -> impl Constraint {
    GccExtendedResolution::new(variables.into(), values.into(), equalities)
}

impl<Var: IntegerVariable + 'static> Constraint for GccExtendedResolution<Var> {
    fn post(
        self,
        solver: &mut crate::Solver,
        tag: Option<std::num::NonZero<u32>>,
    ) -> Result<(), crate::ConstraintOperationError> {
        self.intersections
            .into_iter()
            .try_for_each(|c| c.post(solver, tag))?;
        self.transitives
            .into_iter()
            .try_for_each(|c| c.post(solver, tag))?;
        self.equality_constraints
            .into_iter()
            .try_for_each(|c| c.post(solver, tag))?;
        self.exclusions
            .into_iter()
            .try_for_each(|c| c.post(solver, tag))?;
        self.inequalities
            .into_iter()
            .try_for_each(|c| c.post(solver, tag))?;
        self.inequality_sets.post(solver, tag)?;
        self.conflicts
            .into_iter()
            .try_for_each(|c| c.post(solver, tag))?;
        self.upper_bound
            .into_iter()
            .try_for_each(|c| c.post(solver, tag))
    }

    fn implied_by(
        self,
        solver: &mut crate::Solver,
        reif: Literal,
        tag: Option<std::num::NonZero<u32>>,
    ) -> Result<(), crate::ConstraintOperationError> {
        self.intersections
            .into_iter()
            .try_for_each(|c| c.implied_by(solver, reif, tag))?;
        self.transitives
            .into_iter()
            .try_for_each(|c| c.implied_by(solver, reif, tag))?;
        self.equality_constraints
            .into_iter()
            .try_for_each(|c| c.implied_by(solver, reif, tag))?;
        self.exclusions
            .into_iter()
            .try_for_each(|c| c.implied_by(solver, reif, tag))?;
        self.inequalities
            .into_iter()
            .try_for_each(|c| c.implied_by(solver, reif, tag))?;
        self.inequality_sets.implied_by(solver, reif, tag)?;
        self.conflicts
            .into_iter()
            .try_for_each(|c| c.implied_by(solver, reif, tag))?;
        self.upper_bound
            .into_iter()
            .try_for_each(|c| c.implied_by(solver, reif, tag))
    }
}
