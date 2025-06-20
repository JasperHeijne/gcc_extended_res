use super::Constraint;
use crate::propagators::gcc_david::gcc_lower_upper::GCCLowerUpper;
use crate::propagators::gcc_david::gcc_lower_upper_2::GCCLowerUpper2;
use crate::propagators::gcc_david::simple_gcc_lower_upper::SimpleGCCLowerUpper;
pub use crate::propagators::gcc_david::Values;
use crate::variables::IntegerVariable;

#[derive(Clone, Copy, Debug, Default)]
pub enum GccMethod {
    Bruteforce,
    BasicFilter,
    #[default]
    ReginArcConsistent,
}

#[derive(Debug)]
#[allow(
    clippy::large_enum_variant,
    reason = "enum used rarely enough (once per constraint)"
)]
enum GccConstraint<Variable: IntegerVariable + 'static> {
    Bruteforce(SimpleGCCLowerUpper<Variable>),
    BasicFilter(GCCLowerUpper2<Variable>),
    ReginArcConsistent(GCCLowerUpper<Variable>),
}

impl<Variable: IntegerVariable + 'static> Constraint for GccConstraint<Variable> {
    fn post(
        self,
        solver: &mut crate::Solver,
        tag: Option<std::num::NonZero<u32>>,
    ) -> Result<(), crate::ConstraintOperationError> {
        match self {
            GccConstraint::Bruteforce(simple_gcclower_upper) => {
                simple_gcclower_upper.post(solver, tag)
            }
            GccConstraint::BasicFilter(gcclower_upper2) => gcclower_upper2.post(solver, tag),
            GccConstraint::ReginArcConsistent(gcclower_upper) => gcclower_upper.post(solver, tag),
        }
    }

    fn implied_by(
        self,
        solver: &mut crate::Solver,
        reification_literal: crate::variables::Literal,
        tag: Option<std::num::NonZero<u32>>,
    ) -> Result<(), crate::ConstraintOperationError> {
        match self {
            GccConstraint::Bruteforce(simple_gcclower_upper) => {
                simple_gcclower_upper.implied_by(solver, reification_literal, tag)
            }
            GccConstraint::BasicFilter(gcclower_upper2) => {
                gcclower_upper2.implied_by(solver, reification_literal, tag)
            }
            GccConstraint::ReginArcConsistent(gcclower_upper) => {
                gcclower_upper.implied_by(solver, reification_literal, tag)
            }
        }
    }
}

pub fn global_cardinality_lower_upper<Variable: IntegerVariable + 'static>(
    variables: impl IntoIterator<Item = Variable>,
    values: impl IntoIterator<Item = Values>,
    method: GccMethod,
) -> impl Constraint {
    match method {
        GccMethod::Bruteforce => GccConstraint::Bruteforce(SimpleGCCLowerUpper::new(
            variables.into_iter().collect(),
            values.into_iter().collect(),
        )),
        GccMethod::BasicFilter => GccConstraint::BasicFilter(GCCLowerUpper2::new(
            variables.into_iter().collect(),
            values.into_iter().collect(),
        )),
        GccMethod::ReginArcConsistent => GccConstraint::ReginArcConsistent(GCCLowerUpper::new(
            variables.into_iter().collect(),
            values.into_iter().collect(),
        )),
    }
}
