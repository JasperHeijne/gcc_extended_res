use crate::basic_types::moving_averages::moving_average::MovingAverage;
use crate::basic_types::HashMap;
use crate::basic_types::HashSet;
use crate::engine::propagation::LocalId;
use crate::engine::propagation::PropagationContextMut;
use crate::engine::propagation::Propagator;
use crate::engine::propagation::ReadDomains;
use crate::engine::DomainEvents;
use crate::predicate;
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

pub(crate) struct GccLowerboundConflicts<Var: IntegerVariable + 'static> {
    variables: Box<[Var]>,
    equalities: HashMap<(usize, usize), Literal>,
    value: i32,
    min: usize,
}

impl<Var: IntegerVariable> GccLowerboundConflicts<Var> {
    pub(crate) fn new(
        variables: impl IntoIterator<Item = Var>,
        equalities: HashMap<(usize, usize), Literal>,
        value: i32,
        min: usize,
    ) -> Self {
        Self {
            variables: variables.into_iter().collect(),
            equalities,
            value,
            min,
        }
    }
}

impl<Var: IntegerVariable> Propagator for GccLowerboundConflicts<Var> {
    fn name(&self) -> &str {
        "GCC conflicts with extended resolution"
    }

    fn initialise_at_root(
        &mut self,
        context: &mut crate::engine::propagation::PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        for (i, var) in self.variables.iter().enumerate() {
            let _ = context.register(var.clone(), DomainEvents::ANY_INT, LocalId::from(i as u32));
        }

        for ((i, j), literal) in self.equalities.iter() {
            let id = (self.variables.len()) * (i + 1) + j;
            // UPPER_BOUND changes -> the equality variable is assigned to 0
            let _ = context.register(
                *literal,
                DomainEvents::UPPER_BOUND,
                LocalId::from(id as u32),
            );
        }

        Ok(())
    }

    fn debug_propagate_from_scratch(
        &self,
        context: PropagationContextMut,
    ) -> crate::basic_types::PropagationStatusCP {
        let irrelevant_variables = self
            .variables
            .iter()
            .filter(|var| !var.contains(context.assignments, self.value))
            .collect::<Vec<_>>();

        // If the number of variables with the domain is less than min, then the lower bound of the
        // value cannot be satisfied
        let relevant_variables_count = self.variables.len() - irrelevant_variables.len();
        if relevant_variables_count < self.min {
            // All other variables not having this value causes the conflict
            let reason = irrelevant_variables
                .into_iter()
                .map(|var| predicate!(var != self.value))
                .collect();

            context
                .solver_statistics
                .gcc_extended_statistics
                .extended_propagators_conflicts += 1;

            return Err(crate::basic_types::Inconsistency::Conflict(
                PropositionalConjunction::new(reason),
            ));
        }

        let relevant_variables_indices: HashSet<usize> = (0..self.variables.len())
            .filter(|&i| self.variables[i].contains(context.assignments, self.value))
            .collect();

        let mut edge_count = 0;
        let mut reason = Vec::new();

        for ((i, j), eq_literal) in self.equalities.clone() {
            if relevant_variables_indices.contains(&i)
                && relevant_variables_indices.contains(&j)
                && context.is_literal_false(&eq_literal)
            {
                edge_count += 1;
                reason.push(predicate!(eq_literal == 0));
            }
        }

        // I. Schiermeyer "Maximum independent sets near the upper bound"
        // https://www.sciencedirect.com/science/article/pii/S0166218X18304062
        // mentions the following formula as UB for size of maximum independent set (MIS)
        // with `n` nodes and `m` edges:
        // floor(1/2 + sqrt(1/4 + n^2 - n - 2m))

        let n = relevant_variables_indices.len() as f32;
        let mis_size_upper_bound =
            (0.5 + (0.25 + n * n - n - 2.0 * edge_count as f32).sqrt()).floor() as usize;

        if mis_size_upper_bound < self.min {
            let equality_reason_size = reason.len();
            for var in irrelevant_variables {
                reason.push(predicate!(var != self.value));
            }

            context
                .solver_statistics
                .gcc_extended_statistics
                .max_independent_set_conflicts += 1;

            context
                .solver_statistics
                .gcc_extended_statistics
                .extended_propagators_conflicts += 1;

            context
                .solver_statistics
                .gcc_extended_statistics
                .average_num_of_equality_vars_in_explanation
                .add_term(equality_reason_size as u64);

            context
                .solver_statistics
                .gcc_extended_statistics
                .average_size_of_extended_explanations
                .add_term(reason.len() as u64);

            return Err(crate::basic_types::Inconsistency::Conflict(
                PropositionalConjunction::new(reason),
            ));
        }

        Ok(())
    }

    fn priority(&self) -> u32 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::GccLowerboundConflicts;
    use crate::engine::test_solver::TestSolver;
    use crate::propagators::gcc_extended_resolution::generate_equalities;

    #[test]
    fn too_few_with_domain() {
        let mut solver = TestSolver::default();

        let x1 = solver.new_variable(1, 15);
        let x2 = solver.new_variable(1, 15);
        let x3 = solver.new_variable(1, 15);
        let x4 = solver.new_variable(1, 15);
        let x5 = solver.new_variable(1, 15);
        let variables = vec![x1, x2, x3, x4, x5];

        let equalities = generate_equalities(&mut solver, &variables);

        let value = 10;
        let min = 3;

        let propagator = GccLowerboundConflicts::new(variables, equalities, value, min);

        let propagator = solver.new_propagator(propagator).expect("no empty domains");
        solver
            .propagate_until_fixed_point(propagator)
            .expect("should not conflict");

        let _ = solver.set_bounds(x1, 1, 9);
        let _ = solver.set_bounds(x2, 1, 9);
        let _ = solver.set_bounds(x3, 1, 9);

        let _ = solver
            .propagate_until_fixed_point(propagator)
            .expect_err("at most 2 variables can be assigned, but min = 3");
    }

    #[test]
    fn too_err_because_of_inequality() {
        let mut solver = TestSolver::default();

        let x1 = solver.new_variable(1, 15);
        let x2 = solver.new_variable(1, 15);
        let x3 = solver.new_variable(1, 15);
        let x4 = solver.new_variable(1, 15);
        let x5 = solver.new_variable(1, 15);
        let variables = vec![x1, x2, x3, x4, x5];

        let equalities = generate_equalities(&mut solver, &variables);

        let value = 10;
        let min = 3;

        let propagator = GccLowerboundConflicts::new(variables, equalities.clone(), value, min);

        let propagator = solver.new_propagator(propagator).expect("no empty domains");
        solver
            .propagate_until_fixed_point(propagator)
            .expect("should not conflict");

        let _ = solver.set_bounds(x1, 1, 9);
        let _ = solver.set_bounds(x2, 1, 9);

        let _ = solver.set_literal(equalities[&(2, 3)], false); // x3 != x4

        let _ = solver.propagate_until_fixed_point(propagator).expect_err(
            "min = 3, at most 3 variables can be assigned, but two of them are unequal",
        );
    }
}
