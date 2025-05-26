use crate::basic_types::HashSet;
use crate::conjunction;
use crate::engine::propagation::LocalId;
use crate::engine::propagation::PropagationContextMut;
use crate::engine::propagation::Propagator;
use crate::engine::DomainEvents;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

pub(crate) struct GccIntersection<Var: IntegerVariable + 'static> {
    extended_literal: Literal,
    left: Var,
    right: Var,
}

impl<Var: IntegerVariable> GccIntersection<Var> {
    pub(crate) fn new(extended_literal: Literal, left: Var, right: Var) -> Self {
        Self {
            extended_literal,
            left,
            right,
        }
    }
}

impl<Var: IntegerVariable> Propagator for GccIntersection<Var> {
    fn name(&self) -> &str {
        "if E_{x,y} = 1, then D'(x) = D'(y) = D(x) ∩ D(y)"
    }

    fn initialise_at_root(
        &mut self,
        context: &mut crate::engine::propagation::PropagatorInitialisationContext,
    ) -> Result<(), crate::predicates::PropositionalConjunction> {
        let _ = context.register(
            self.extended_literal,
            DomainEvents::LOWER_BOUND,
            LocalId::from(0),
        );
        let _ = context.register(self.left.clone(), DomainEvents::ANY_INT, LocalId::from(1));
        let _ = context.register(self.right.clone(), DomainEvents::ANY_INT, LocalId::from(2));

        Ok(())
    }

    fn debug_propagate_from_scratch(
        &self,
        mut context: PropagationContextMut,
    ) -> crate::basic_types::PropagationStatusCP {
        if self.extended_literal.lower_bound(context.assignments) != 1 {
            // Early return if E_{x,y} is not set to 1
            return Ok(());
        }

        let left: HashSet<_> = self.left.iterate_domain(context.assignments).collect();
        let right: HashSet<_> = self.right.iterate_domain(context.assignments).collect();

        for value in &left {
            if !right.contains(value) {
                let reason = conjunction!([self.extended_literal == 1] & [self.right != *value]);
                PropagationContextMut::remove(&mut context, &self.left, *value, reason)?;
            }
        }

        for value in &right {
            if !left.contains(value) {
                let reason = conjunction!([self.extended_literal == 1] & [self.left != *value]);
                PropagationContextMut::remove(&mut context, &self.right, *value, reason)?;
            }
        }

        Ok(())
    }
}
