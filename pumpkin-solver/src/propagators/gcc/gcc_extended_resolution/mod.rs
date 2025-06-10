pub(crate) mod equality;
pub(crate) mod exclusion;
pub(crate) mod inequality;
pub(crate) mod intersection;
pub(crate) mod transitive;
pub(crate) mod upper_bound;

#[cfg(test)]
fn generate_equalities(
    solver: &mut crate::engine::test_solver::TestSolver,
    vars: &[crate::variables::DomainId],
) -> crate::basic_types::HashMap<(usize, usize), crate::variables::Literal> {
    use crate::basic_types::HashMap;
    use crate::variables::Literal;

    let mut local_map: HashMap<(usize, usize), Literal> = HashMap::default();

    // This loop wastes half of its iterations, but it is still O(n^2)

    for (i, a) in vars.iter().enumerate() {
        for (j, b) in vars.iter().enumerate() {
            // Ensure a has lower id than b
            if a.id >= b.id {
                continue;
            }
            let literal = solver.new_literal();

            let _ = local_map.insert((i, j), literal);
        }
    }

    local_map
}
