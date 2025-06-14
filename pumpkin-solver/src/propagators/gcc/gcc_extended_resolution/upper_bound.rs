use reunion::UnionFind;
use reunion::UnionFindTrait;

use crate::basic_types::HashMap;
use crate::basic_types::HashSet;
use crate::engine::predicates::predicate::Predicate;
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

    fn get_set_reason(
        &self,
        graph: &HashMap<usize, Vec<usize>>,
        start_node: usize,
    ) -> Vec<Predicate> {
        let mut reason = Vec::new();
        let mut stack = Vec::new();
        let mut visited: HashSet<usize> = HashSet::default();

        stack.push(start_node);
        let _ = visited.insert(start_node);

        while !stack.is_empty() {
            let cur = stack
                .pop()
                .expect("There is an element if the stack isn't empty");
            if let Some(neighbours) = graph.get(&cur) {
                for &n in neighbours {
                    if visited.contains(&n) {
                        continue;
                    }
                    stack.push(n);
                    let _ = visited.insert(n);
                    let eq_literal = self.get_equality(cur, n);
                    reason.push(predicate!(eq_literal == 1));
                }
            }
        }

        reason
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

        let mut equality_graph: HashMap<usize, Vec<usize>> = HashMap::default();

        for ((i, j), literal) in &self.equalities {
            if context.is_literal_true(literal) {
                uf.union(*i, *j);
                equality_graph.entry(*i).or_default().push(*j);
                equality_graph.entry(*j).or_default().push(*i);
            }
        }

        // Now recalculated per set, to include only variables that aren't in the set
        // See `assigned_not_in_set`
        // let mut assigned: HashMap<i32, Vec<&Var>> = HashMap::default();
        // for var in &self.variables {
        //     if context.is_fixed(var) {
        //         let value = context.lower_bound(var);
        //         assigned.entry(value).or_default().push(var);
        //     }
        // }

        for set in uf.subsets() {
            if set.len() < 2 {
                // Ignore sets with only one variable
                continue;
            }

            let set_reason = self.get_set_reason(
                &equality_graph,
                *set.iter().next().expect("set has size of at least 2"),
            );

            let mut assigned_not_in_set: HashMap<i32, Vec<&Var>> = HashMap::default();
            for (i, var) in self.variables.iter().enumerate() {
                if set.contains(&i) {
                    continue;
                }
                if context.is_fixed(var) {
                    let value = context.lower_bound(var);
                    assigned_not_in_set.entry(value).or_default().push(var);
                }
            }

            let domain: Vec<_> = self.variables
                [*set.iter().next().expect("set has size of at least 2")]
            .iterate_domain(context.assignments)
            .collect();

            let k = set.len();

            for value in domain {
                let upper_bound = if let Some(upper_bound) = self.values.get(&value) {
                    upper_bound.1
                } else {
                    continue;
                };

                let assigned_vars = assigned_not_in_set.entry(value).or_default();

                if k + assigned_vars.len() <= upper_bound {
                    // the upperbound is not exceeded even if all k variables from set
                    // are assigned the value
                    continue;
                }

                // we exceed the upper bound
                // let mut reason = Vec::new();
                // Arbitrary chain/tree for now
                // todo: improve somehow

                // The following might not work properly:
                // For example, in uf (without path compression) we have
                // x -> y -> z because E_xy and E_yz
                // If the transitive propagator wasn't called then E_xz is still unassigned
                // But because of path compression parent[x] = z
                // So we try to add [E_xz == 1] to the reason, which isn't true yet
                // For now this gets solved using `set_reason` above
                // TODO: Fix/improve

                // for var_index in set.iter() {
                //     let parent_index = uf.find(*var_index);
                //     if *var_index == parent_index {
                //         continue;
                //     }
                //     let literal = self.get_equality(*var_index, parent_index);
                //     reason.push(predicate!(literal == 1));
                // }

                let mut reason = set_reason.clone();

                let required_assigned_number = upper_bound - k.min(upper_bound);

                for &assigned_var in assigned_vars.iter().take(required_assigned_number) {
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

    fn priority(&self) -> u32 {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::GccUpperBound;
    use super::*;
    use crate::basic_types::HashMap;
    use crate::conjunction;
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
        // solver.assert_bounds(x3, 1, 3);
    }

    #[test]
    fn too_many_assigned() {
        let mut solver = TestSolver::default();

        let x1 = solver.new_variable(1, 3);
        let x2 = solver.new_variable(1, 3);
        let x3 = solver.new_variable(1, 3);

        let values: HashMap<i32, (usize, usize)> =
            HashMap::from_iter([(1, (0, 2)), (2, (0, 2)), (3, (0, 2))]);

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
        solver.set_literal(equalities[&(1, 2)], true).unwrap(); // x2 = x3

        let _ = solver
            .propagate_until_fixed_point(propagator)
            .expect_err("no assignment is possible");
    }

    #[test]
    fn test_not_all_assigned_vars_in_explanation() {
        let mut solver = TestSolver::default();

        let x1 = solver.new_variable(1, 2);
        let x2 = solver.new_variable(1, 2);
        let x3 = solver.new_variable(2, 2);

        let values: HashMap<i32, (usize, usize)> = HashMap::from_iter([(1, (0, 10)), (2, (0, 1))]);

        let equalities = generate_equalities(&mut solver, &[x1, x2, x3]);

        solver.set_literal(equalities[&(0, 1)], true).unwrap(); // x1 = x2

        let propagator = GccUpperBound {
            variables: Box::new([x1, x2, x3]),
            values,
            equalities: equalities.clone(),
        };

        let _ = solver
            .new_propagator(propagator)
            .expect("Expected no errors");
        solver.assert_bounds(x1, 1, 1);
        solver.assert_bounds(x2, 1, 1);

        let reason = solver.get_reason_int(predicate!(x1 != 2));
        assert_eq!(conjunction!([equalities[&(0, 1)] == 1]), reason);
    }
}
