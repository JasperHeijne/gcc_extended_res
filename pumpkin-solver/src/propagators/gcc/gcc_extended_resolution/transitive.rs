use crate::conjunction;
use crate::engine::propagation::LocalId;
use crate::engine::propagation::PropagationContextMut;
use crate::engine::propagation::Propagator;
use crate::engine::DomainEvents;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

pub(crate) struct GccTransitive {
    xy: Literal,
    yz: Literal,
    xz: Literal,
}

impl GccTransitive {
    pub(crate) fn new(xy: Literal, yz: Literal, xz: Literal) -> Self {
        Self { xy, yz, xz }
    }
}

impl Propagator for GccTransitive {
    fn name(&self) -> &str {
        "E_{x,y} = 1 /\\ E_{y,z} = 1 => E_{x,z} = 1"
    }

    fn initialise_at_root(
        &mut self,
        context: &mut crate::engine::propagation::PropagatorInitialisationContext,
    ) -> Result<(), crate::predicates::PropositionalConjunction> {
        // We only care when it is set to 1, i.e. lowerbound increases from 0 to 1.
        let _ = context.register(self.xy, DomainEvents::LOWER_BOUND, LocalId::from(0));
        let _ = context.register(self.yz, DomainEvents::LOWER_BOUND, LocalId::from(0));

        Ok(())
    }

    fn debug_propagate_from_scratch(
        &self,
        mut context: PropagationContextMut,
    ) -> crate::basic_types::PropagationStatusCP {
        if self.xy.lower_bound(context.assignments) == 1
            && self.yz.lower_bound(context.assignments) == 1
        {
            let reason = conjunction!([self.xy == 1] & [self.yz == 1]);
            PropagationContextMut::assign_literal(&mut context, &self.xz, true, reason)?;
        }

        Ok(())
    }
}
