use crate::conjunction;
use crate::engine::propagation::LocalId;
use crate::engine::propagation::PropagationContextMut;
use crate::engine::propagation::Propagator;
use crate::engine::propagation::ReadDomains;
use crate::engine::DomainEvents;
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
        "GCC transitive"
    }

    fn initialise_at_root(
        &mut self,
        context: &mut crate::engine::propagation::PropagatorInitialisationContext,
    ) -> Result<(), crate::predicates::PropositionalConjunction> {
        let _ = context.register(self.xy, DomainEvents::ASSIGN, LocalId::from(0));
        let _ = context.register(self.yz, DomainEvents::ASSIGN, LocalId::from(1));

        Ok(())
    }

    fn debug_propagate_from_scratch(
        &self,
        mut context: PropagationContextMut,
    ) -> crate::basic_types::PropagationStatusCP {
        if context.is_literal_true(&self.xy) {
            if context.is_literal_true(&self.yz) {
                // E_{x,y} = 1 /\\ E_{y,z} = 1 => E_{x,z} = 1
                let reason = conjunction!([self.xy == 1] & [self.yz == 1]);
                context.assign_literal(&self.xz, true, reason)?;
            } else if context.is_literal_false(&self.yz) {
                // E_{x,y} = 1 /\\ E_{y,z} = 0 => E_{x,z} = 0
                let reason = conjunction!([self.xy == 1] & [self.yz == 0]);
                context.assign_literal(&self.xz, false, reason)?;
            }
        } else if context.is_literal_false(&self.xy) && context.is_literal_true(&self.yz) {
            // E_{x,y} = 0 /\\ E_{y,z} = 1 => E_{x,z} = 0
            let reason = conjunction!([self.xy == 0] & [self.yz == 1]);
            context.assign_literal(&self.xz, false, reason)?;
        }

        Ok(())
    }

    fn priority(&self) -> u32 {
        0
    }
}
