use crate::engine::propagation::LocalId;
use crate::engine::propagation::PropagationContextMut;
use crate::engine::propagation::Propagator;
use crate::engine::propagation::ReadDomains;
use crate::engine::DomainEvents;
use crate::predicate;
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;

pub(crate) struct GccConflicts<Var: IntegerVariable + 'static> {
    variables: Box<[Var]>,
    value: i32,
    min: usize,
    max: usize,
}

impl<Var: IntegerVariable> GccConflicts<Var> {
    pub(crate) fn new(
        variables: impl IntoIterator<Item = Var>,
        value: i32,
        min: usize,
        max: usize,
    ) -> Self {
        Self {
            variables: variables.into_iter().collect(),
            value,
            min,
            max,
        }
    }
}

impl<Var: IntegerVariable> Propagator for GccConflicts<Var> {
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

        Ok(())
    }

    fn debug_propagate_from_scratch(
        &self,
        context: PropagationContextMut,
    ) -> crate::basic_types::PropagationStatusCP {
        let relevant_variables = self
            .variables
            .iter()
            .filter(|var| var.contains(context.assignments, self.value))
            .collect::<Vec<_>>();

        // If the number of variables with the domain is less than min, then the lower bound of the
        // value cannot be satisfied
        if relevant_variables.len() < self.min {
            // All other variables not having this value causes the conflict
            let reason = self
                .variables
                .iter()
                .filter(|var| !var.contains(context.assignments, self.value))
                .map(|var| predicate!(var != self.value))
                .collect();

            return Err(crate::basic_types::Inconsistency::Conflict(
                PropositionalConjunction::new(reason),
            ));
        }

        let assigned_variables = self
            .variables
            .iter()
            .filter(|var| context.is_fixed(*var) && var.contains(context.assignments, self.value))
            .collect::<Vec<_>>();

        // Number of assigned variables should not violate the upperbound
        if assigned_variables.len() > self.max {
            let reason = assigned_variables
                .into_iter()
                .map(|var| predicate!(var == self.value))
                .collect();

            return Err(crate::basic_types::Inconsistency::Conflict(
                PropositionalConjunction::new(reason),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::GccConflicts;
    use crate::engine::test_solver::TestSolver;

    #[test]
    fn assigned() {
        let mut solver = TestSolver::default();

        let x1 = solver.new_variable(1, 15);
        let x2 = solver.new_variable(1, 15);
        let x3 = solver.new_variable(1, 15);
        let x4 = solver.new_variable(1, 15);
        let x5 = solver.new_variable(1, 15);
        let variables = vec![x1, x2, x3, x4, x5];

        let value = 10;
        let min = 3;
        let max = 5;

        let propagator = GccConflicts::new(variables, value, min, max);

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
}
