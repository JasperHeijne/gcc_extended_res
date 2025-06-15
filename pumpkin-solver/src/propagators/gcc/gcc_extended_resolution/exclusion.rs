use crate::conjunction;
use crate::engine::propagation::LocalId;
use crate::engine::propagation::PropagationContextMut;
use crate::engine::propagation::Propagator;
use crate::engine::propagation::ReadDomains;
use crate::engine::DomainEvents;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

pub(crate) struct GccExclusion<Var: IntegerVariable + 'static> {
    literal: Literal,
    left: Var,
    right: Var,
}

impl<Var: IntegerVariable> GccExclusion<Var> {
    pub(crate) fn new(literal: Literal, left: Var, right: Var) -> Self {
        Self {
            literal,
            left,
            right,
        }
    }
}

impl<Var: IntegerVariable> Propagator for GccExclusion<Var> {
    fn name(&self) -> &str {
        "E_{x,y} = 0 /\\ x = v => y != v"
    }

    fn initialise_at_root(
        &mut self,
        context: &mut crate::engine::propagation::PropagatorInitialisationContext,
    ) -> Result<(), crate::predicates::PropositionalConjunction> {
        let _ = context.register(self.literal, DomainEvents::UPPER_BOUND, LocalId::from(0));
        let _ = context.register(self.left.clone(), DomainEvents::ASSIGN, LocalId::from(1));

        Ok(())
    }

    fn debug_propagate_from_scratch(
        &self,
        mut context: PropagationContextMut,
    ) -> crate::basic_types::PropagationStatusCP {
        if context.is_literal_false(&self.literal) && context.is_fixed(&self.left) {
            let value = context.lower_bound(&self.left);
            let reason = conjunction!([self.literal == 0] & [self.left == value]);

            PropagationContextMut::remove(&mut context, &self.right, value, reason)?;
        }

        Ok(())
    }

    fn priority(&self) -> u32 {
        0
    }
}
