use crate::conjunction;
use crate::engine::propagation::LocalId;
use crate::engine::propagation::PropagationContextMut;
use crate::engine::propagation::Propagator;
use crate::engine::propagation::ReadDomains;
use crate::engine::DomainEvents;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

pub(crate) struct GccEquality<Var: IntegerVariable + 'static> {
    left: Var,
    right: Var,
    literal: Literal,
}

impl<Var: IntegerVariable> GccEquality<Var> {
    pub(crate) fn new(left: Var, right: Var, literal: Literal) -> Self {
        Self {
            left,
            right,
            literal,
        }
    }
}

impl<Var: IntegerVariable> Propagator for GccEquality<Var> {
    fn name(&self) -> &str {
        "x = v /\\ y = v => E_{x,y} = 1"
    }

    fn initialise_at_root(
        &mut self,
        context: &mut crate::engine::propagation::PropagatorInitialisationContext,
    ) -> Result<(), crate::predicates::PropositionalConjunction> {
        let _ = context.register(self.left.clone(), DomainEvents::ASSIGN, LocalId::from(0));
        let _ = context.register(self.right.clone(), DomainEvents::ASSIGN, LocalId::from(1));

        Ok(())
    }

    fn debug_propagate_from_scratch(
        &self,
        mut context: PropagationContextMut,
    ) -> crate::basic_types::PropagationStatusCP {
        if context.is_literal_true(&self.literal) {
            return Ok(());
        }

        if context.is_fixed(&self.left)
            && context.is_fixed(&self.right)
            && context.lower_bound(&self.left) == context.lower_bound(&self.right)
        {
            let value = context.lower_bound(&self.left);
            let reason = conjunction!([self.left == value] & [self.right == value]);

            context
                .solver_statistics
                .gcc_extended_statistics
                .equality_propagations += 1;

            PropagationContextMut::assign_literal(&mut context, &self.literal, true, reason)?;
        }

        Ok(())
    }

    fn priority(&self) -> u32 {
        0
    }
}
