use std::collections::HashSet;

use crate::basic_types::HashMap;
use crate::engine::propagation::LocalId;
use crate::engine::propagation::PropagationContextMut;
use crate::engine::propagation::Propagator;
use crate::engine::propagation::ReadDomains;
use crate::engine::DomainEvents;
use crate::predicate;
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

use rand::seq::SliceRandom;
use rand::thread_rng;

pub(crate) struct GccInequalitySets<Var: IntegerVariable + 'static> {
    variables: Box<[Var]>,
    equalities: HashMap<(usize, usize), Literal>,
}

impl<Var: IntegerVariable> GccInequalitySets<Var> {
    pub(crate) fn new(
        variables: impl IntoIterator<Item = Var>,
        equalities: HashMap<(usize, usize), Literal>,
    ) -> Self {
        Self {
            variables: variables.into_iter().collect(),
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

impl<Var: IntegerVariable> Propagator for GccInequalitySets<Var> {
    fn name(&self) -> &str {
        "GCC extended resolution inequality sets"
    }

    fn initialise_at_root(
        &mut self,
        context: &mut crate::engine::propagation::PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        // In theory, we could run this propagator for every change in domains of variables
        // but this would be rather wasteful, right?
        for (i, var) in self.variables.iter().enumerate() {
            let _ = context.register(var.clone(), DomainEvents::ASSIGN, LocalId::from(i as u32));
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
        mut context: PropagationContextMut,
    ) -> crate::basic_types::PropagationStatusCP {
        // Consider variables in random order
        let mut variable_indices: Vec<usize> = (0..self.variables.len()).collect();
        variable_indices.shuffle(&mut thread_rng());

        let mut inequality_set;
        let mut found_valid_clique = false;

        for (i, &var_index) in variable_indices.iter().enumerate() {
            inequality_set = HashSet::new();
            let _ = inequality_set.insert(var_index);

            for j in (i + 1)..self.variables.len() {
                let mut all_unequal = true;
                for set_member in inequality_set.clone() {
                    let var = self.get_equality(variable_indices[j], set_member);
                    if !context.is_literal_false(&var) {
                        all_unequal = false;
                        break;
                    }
                }
                if all_unequal {
                    let _ = inequality_set.insert(variable_indices[j]);
                }
            }

            if inequality_set.len() <= 2 {
                // GccExclusion already deals with two variables cases
                continue;
            }

            // We would also want our clique variables to have some common domain values
            // As otherwise we won't do any pruning
            let mut common_domain: HashSet<_> = self.variables[var_index]
                .iterate_domain(&context.assignments)
                .collect();
            for ind in inequality_set {
                if ind == var_index {
                    continue;
                }
                common_domain = common_domain
                    .intersection(
                        &self.variables[ind]
                            .iterate_domain(&context.assignments)
                            .collect(),
                    )
                    .cloned()
                    .collect();
            }
            if common_domain.len() == 0 {
                continue;
            }

            found_valid_clique = true;
            break;
        }

        if !found_valid_clique {
            return Ok(());
        }

        // We have now found a valid inequality clique with size k > 2

        todo!()
    }
}
