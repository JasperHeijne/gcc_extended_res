use reunion::UnionFind;
use reunion::UnionFindTrait;

use crate::basic_types::HashMap;
use crate::engine::propagation::LocalId;
use crate::engine::propagation::PropagationContextMut;
use crate::engine::propagation::Propagator;
use crate::engine::propagation::ReadDomains;
use crate::engine::DomainEvents;
use crate::predicate;
use crate::predicates::PropositionalConjunction;
use crate::propagators::gcc_david::Values;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

pub(crate) struct GccUpperBound<Var: IntegerVariable + 'static> {
    variables: Box<[Var]>,
    values: HashMap<i32, (usize, usize)>,
    equalities: HashMap<(usize, usize), Literal>,
}

impl<Var: IntegerVariable> GccUpperBound<Var> {
    fn values_to_bounds(values: impl IntoIterator<Item = Values>) -> HashMap<i32, (usize, usize)> {
        values
            .into_iter()
            .map(|values| (values.value, (values.omin as usize, values.omax as usize)))
            .collect()
    }

    pub(crate) fn new(
        variables: impl IntoIterator<Item = Var>,
        values: impl IntoIterator<Item = Values>,
        equalities: HashMap<(usize, usize), Literal>,
    ) -> Self {
        Self {
            variables: variables.into_iter().collect(),
            values: Self::values_to_bounds(values),
            equalities,
        }
    }

    fn get_equality(&self, x: usize, y: usize) -> Literal {
        assert!(x < self.variables.len());
        assert!(y < self.variables.len());

        *self
            .equalities
            .get(&(x, y))
            .or(self.equalities.get(&(y, x)))
            .expect("E_{x,y} or E_{y,x} must be defined")
    }
}

impl<Var: IntegerVariable> Propagator for GccUpperBound<Var> {
    fn name(&self) -> &str {
        "GCC upper-bound with extended resolution"
    }

    fn initialise_at_root(
        &mut self,
        context: &mut crate::engine::propagation::PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        for (i, var) in self.variables.iter().enumerate() {
            let _ = context.register(var.clone(), DomainEvents::ASSIGN, LocalId::from(i as u32));
        }

        for ((i, j), literal) in self.equalities.iter() {
            let id = (self.variables.len()) * (i + 1) + j;
            let _ = context.register(
                *literal,
                DomainEvents::LOWER_BOUND,
                LocalId::from(id as u32),
            );
        }

        Ok(())
    }

    fn debug_propagate_from_scratch(
        &self,
        mut context: PropagationContextMut,
    ) -> crate::basic_types::PropagationStatusCP {
        let mut uf: UnionFind<usize> = UnionFind::with_capacity(self.variables.len());

        for ((i, j), literal) in &self.equalities {
            if context.is_literal_true(literal) {
                uf.union(*i, *j);
            }
        }

        dbg!(&uf);

        let mut assigned: HashMap<i32, Vec<&Var>> = HashMap::default();
        for var in &self.variables {
            if context.is_fixed(var) {
                let value = context.lower_bound(var);
                assigned.entry(value).or_default().push(var);
            }
        }

        for set in uf.subsets() {
            if set.len() < 2 {
                // Ignore sets with only one variable
                continue;
            }

            let domain: Vec<_> = self.variables
                [*set.iter().next().expect("set has size of at least 2")]
            .iterate_domain(context.assignments)
            .collect();

            let k = domain.len();

            for value in domain {
                let upper_bound = if let Some(upper_bound) = self.values.get(&value) {
                    upper_bound.1
                } else {
                    continue;
                };
                let assigned_vars = assigned.entry(value).or_default();
                if k + assigned_vars.len() <= upper_bound {
                    // the upperbound is not exceeded even if all k variables
                    // are assigned the value
                    continue;
                }

                // we exceed the upper bound
                let mut reason = Vec::new();
                // Arbitrary chain/tree for now
                // todo: improve somehow
                for var_index in set.iter() {
                    let parent_index = uf.find(*var_index);
                    if parent_index == *var_index {
                        continue;
                    }
                    let literal = self.get_equality(*var_index, parent_index);
                    reason.push(predicate!(literal == 1));
                }

                for assigned_var in assigned_vars {
                    reason.push(predicate!(assigned_var == value))
                }

                for var_index in set.iter() {
                    let var = &self.variables[*var_index];

                    PropagationContextMut::remove(
                        &mut context,
                        var,
                        value,
                        PropositionalConjunction::new(reason.clone()),
                    )?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::GccUpperBound;
    use crate::basic_types::HashMap;
    use crate::engine::test_solver::TestSolver;
    use crate::propagators::gcc_extended_resolution::generate_equalities;

    #[test]
    fn test_eliminate_from_set() {
        let mut solver = TestSolver::default();

        let x1 = solver.new_variable(1, 3);
        let x2 = solver.new_variable(1, 3);
        let x3 = solver.new_variable(1, 3);

        let values: HashMap<i32, (usize, usize)> =
            HashMap::from_iter([(1, (0, 1)), (2, (0, 1)), (3, (0, 2))]);

        let equalities = generate_equalities(&mut solver, &[x1, x2, x3]);

        let propagator = GccUpperBound {
            variables: Box::new([x1, x2, x3]),
            values,
            equalities: equalities.clone(),
        };

        let propagator = solver.new_propagator(propagator).expect("no empty domains");
        solver
            .propagate_until_fixed_point(propagator)
            .expect("should not conflict");

        solver.set_literal(equalities[&(0, 1)], true).unwrap(); // x1 = x2

        solver
            .propagate_until_fixed_point(propagator)
            .expect("should not conflict");

        solver.assert_bounds(x1, 3, 3);
        solver.assert_bounds(x2, 3, 3);
        solver.assert_bounds(x2, 1, 2);
    }
}
