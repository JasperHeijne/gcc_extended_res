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

pub(crate) struct GccInequality<Var: IntegerVariable + 'static> {
    left: Var,
    right: Var,
    literal: Literal,
}

impl<Var: IntegerVariable> GccInequality<Var> {
    pub(crate) fn new(left: Var, right: Var, literal: Literal) -> Self {
        Self {
            left,
            right,
            literal,
        }
    }
}

impl<Var: IntegerVariable> Propagator for GccInequality<Var> {
    fn name(&self) -> &str {
        "If D(x) ∩ D(y) = {}, then E_{x,y} = 0"
    }

    fn initialise_at_root(
        &mut self,
        context: &mut crate::engine::propagation::PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        let _ = context.register(self.left.clone(), DomainEvents::ANY_INT, LocalId::from(0));
        let _ = context.register(self.right.clone(), DomainEvents::ANY_INT, LocalId::from(1));

        Ok(())
    }

    fn debug_propagate_from_scratch(
        &self,
        mut context: PropagationContextMut,
    ) -> crate::basic_types::PropagationStatusCP {
        if context.is_literal_false(&self.literal) {
            return Ok(());
        }

        let left: HashSet<_> = self.left.iterate_domain(context.assignments).collect();
        let right: HashSet<_> = self.right.iterate_domain(context.assignments).collect();

        if left.intersection(&right).next().is_none() {
            let mut reason = Vec::new();
            domain_description(&mut reason, &self.left, context.assignments);
            domain_description(&mut reason, &self.right, context.assignments);

            context
                .solver_statistics
                .gcc_extended_statistics
                .equality_propagations += 1;

            PropagationContextMut::assign_literal(
                &mut context,
                &self.literal,
                false,
                PropositionalConjunction::new(reason),
            )?;
        }

        Ok(())
    }

    fn priority(&self) -> u32 {
        1
    }
}

fn domain_description<Var: IntegerVariable>(
    description: &mut Vec<crate::predicates::Predicate>,
    var: &Var,
    assignment: &crate::engine::Assignments,
) {
    let lower_bound = var.lower_bound(assignment);
    let upper_bound = var.upper_bound(assignment);
    description.push(predicate!(var >= lower_bound));
    description.push(predicate!(var <= upper_bound));

    for value in lower_bound..=upper_bound {
        if !var.contains(assignment, value) {
            description.push(predicate!(var != value));
        }
    }
}
