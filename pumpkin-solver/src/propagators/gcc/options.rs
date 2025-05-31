use clap::ValueEnum;

#[derive(Debug, Copy, Clone, Default)]
pub struct GccOptions {
    pub propagation_method: GccPropagatorMethod,
}

impl GccOptions {
    pub fn new(propagation_method: GccPropagatorMethod) -> Self {
        Self { propagation_method }
    }
}

#[derive(Debug, Copy, Clone, Default, ValueEnum)]
pub enum GccPropagatorMethod {
    Bruteforce,
    BasicFilter,
    ReginArcConsistent,
    ExtendedResolution,
    #[default]
    ExtendedResolutionWithRegin,
}
