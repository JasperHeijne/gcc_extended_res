use crate::basic_types::HashMap;
use crate::basic_types::HashSet;
use crate::basic_types::Inconsistency;
use crate::engine::propagation::LocalId;
use crate::engine::propagation::PropagationContextMut;
use crate::engine::propagation::Propagator;
use crate::engine::propagation::ReadDomains;
use crate::engine::DomainEvents;
use crate::predicate;
use crate::predicates::Predicate;
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

pub(crate) struct GccInequalitySets<Var> {
    variables: Box<[Var]>,
    equalities: HashMap<(usize, usize), Literal>,
}

impl<Var> GccInequalitySets<Var> {
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

    fn get_inequality_explanation(&self, vars: &[usize]) -> Vec<Predicate> {
        let mut reason = Vec::new();
        for i in 0..vars.len() {
            for j in (i + 1)..vars.len() {
                let literal = self.get_equality(vars[i], vars[j]);
                reason.push(predicate!(literal == 0));
            }
        }
        reason
    }
}

impl<Var: IntegerVariable + 'static> Propagator for GccInequalitySets<Var> {
    fn name(&self) -> &str {
        "GCC extended resolution inequality sets"
    }

    fn initialise_at_root(
        &mut self,
        context: &mut crate::engine::propagation::PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        for (i, var) in self.variables.iter().enumerate() {
            let _ = context.register(var.clone(), DomainEvents::ANY_INT, LocalId::from(i as u32));
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
        // Order variables using heuristic
        // - descendingly on the number of inequalities the variable is involved in
        // - break ties by ordering ascendingly on domain size
        // - lastly break ties using the original ordering
        let mut inequalities_count = vec![0; self.variables.len()];
        let mut domain_sizes = vec![0; self.variables.len()];

        for i in 0..self.variables.len() {
            for j in (i + 1)..self.variables.len() {
                let eq_literal = self.get_equality(i, j);
                if context.is_literal_false(&eq_literal) {
                    inequalities_count[i] += 1;
                    inequalities_count[j] += 1;
                }
            }
            domain_sizes[i] = self.variables[i].describe_domain(context.assignments).len();
        }

        let mut variable_indices: Vec<usize> = (0..self.variables.len()).collect();
        variable_indices.sort_by(|&a, &b| {
            inequalities_count[b]
                .cmp(&inequalities_count[a]) // descending on number of inequalities
                .then(domain_sizes[a].cmp(&domain_sizes[b])) // ascending on domain size
                .then(a.cmp(&b)) // ascending (original order)
        });

        let mut inequality_set: HashSet<usize> = HashSet::default();
        let mut found_valid_clique = false;

        for (i, &var_index) in variable_indices.iter().enumerate() {
            // For each variable, we take it and greedily try to find a set of variables that are
            // pairwise unequal
            inequality_set = HashSet::default();
            let _ = inequality_set.insert(var_index);

            for (j, &candidate_var) in variable_indices.iter().enumerate() {
                if j == i {
                    continue;
                }
                let mut all_unequal = true;
                for set_member in inequality_set.clone() {
                    let var = self.get_equality(candidate_var, set_member);
                    if !context.is_literal_false(&var) {
                        all_unequal = false;
                        break;
                    }
                }
                if all_unequal {
                    let _ = inequality_set.insert(candidate_var);
                }
            }

            if inequality_set.len() <= 2 {
                // GccExclusion already deals with two variables cases
                continue;
            }

            found_valid_clique = true;
            break;
        }

        if !found_valid_clique {
            return Ok(());
        }

        // We have now found a valid inequality clique with size k > 2

        let chosen_variables: Vec<usize> = inequality_set.iter().cloned().collect();
        let vars_len = chosen_variables.len();

        let mut values_to_ids: HashMap<i32, usize> = HashMap::default();
        let mut ids_to_values: HashMap<usize, i32> = HashMap::default();
        let mut next_value_id = vars_len + 1;

        let mut graph = Graph::new();

        // In the graph we keep track of nodes using usize which work as their IDs
        // 0 : source
        // 1 ..= vars_len : nodes for variables
        // vars_len+1 .. (however many needed) : nodes for values
        // (last) : sink

        let source = 0;

        // Connect source to each variable
        for i in 1..=vars_len {
            graph.add_edge(source, i);
        }

        // Connect each variable to its domain
        // Maintain matching `value -> node_id` and `node_id -> value`
        for (var_node_id, &var_ind) in chosen_variables.clone().iter().enumerate() {
            for x in self.variables[var_ind].lower_bound(context.assignments)
                ..=self.variables[var_ind].upper_bound(context.assignments)
            {
                if !self.variables[var_ind].contains(context.assignments, x) {
                    continue;
                }
                #[allow(clippy::map_entry, reason = "two inserts inside")]
                if !values_to_ids.contains_key(&x) {
                    let _ = values_to_ids.insert(x, next_value_id);
                    let _ = ids_to_values.insert(next_value_id, x);
                    next_value_id += 1;
                }
                graph.add_edge(
                    var_node_id + 1,
                    *values_to_ids
                        .get(&x)
                        .expect("All values should be in values_to_ids map"),
                );
            }
        }

        let sink = next_value_id;

        // Connect each value to sink
        for i in vars_len + 1..sink {
            graph.add_edge(i, sink);
        }

        // Maximize flow
        let max_flow = graph.ford_fulkerson(source, sink);

        if max_flow < vars_len as i32 {
            // Max flow is too small -> conflict

            // The explanation for the conflict includes the domain description
            // of all variables that are part of the flow + one other variable
            let mut included_vars: HashSet<usize> = HashSet::default();

            // Get variables that participate in the flow
            for ((u, v), cap) in graph.residual_capacities {
                if cap == 1 && u > 0 && v > 0 && v <= vars_len && !included_vars.contains(&(v - 1))
                {
                    let _ = included_vars.insert(v - 1);
                }
            }

            // Add one more variable
            for i in 0..vars_len {
                if !included_vars.contains(&i) {
                    let _ = included_vars.insert(i);
                    break;
                }
            }

            let mut reason = self.get_inequality_explanation(&chosen_variables);
            let all_diff_reason: Vec<Predicate> = included_vars
                .iter()
                .map(|&i| self.variables[chosen_variables[i]].clone())
                .flat_map(|var| var.describe_domain(context.assignments))
                .collect();
            reason.extend(all_diff_reason);
            return Err(Inconsistency::Conflict(PropositionalConjunction::new(
                reason,
            )));
        }

        // We have a valid flow, analyze connected components to prune values

        // Run Tarjan's algorithm and obtain mapping node -> its SCC root
        let (node_to_scc_root, sccs) = tarjans_algorithm(&graph);

        // Iterate over SCCs in reverse topological order
        for scc in sccs.iter().rev() {
            for &v in scc {
                if v == 0 || v > vars_len {
                    // Skip nodes that aren't variables
                    continue;
                }

                if let Some(neighbours) = &graph.adjacency_list.get(&v).cloned() {
                    for &u in neighbours {
                        // Skip `variable -> source edges`
                        if u == source {
                            continue;
                        }

                        // Check variable and value are in different SCCs
                        if node_to_scc_root[&v] == node_to_scc_root[&u] {
                            continue;
                        }

                        // Make sure this is a `variable -> value` edge in the residual graph
                        if graph.residual_capacities[&(v, u)] != 1 {
                            continue;
                        }

                        let mut reason = self.get_inequality_explanation(&chosen_variables);
                        let mut connected: HashSet<usize> = HashSet::default();

                        // Find all nodes connected to the value
                        let _ = graph.dfs(u, sink + 1, &mut connected, &mut Vec::new());

                        // Filter out the variable nodes and use them in explanation
                        for node in connected {
                            if node > 0 && node <= vars_len {
                                reason.extend(
                                    self.variables[chosen_variables[node - 1]]
                                        .describe_domain(context.assignments),
                                );
                            }
                        }

                        PropagationContextMut::remove(
                            &mut context,
                            &self.variables[chosen_variables[v - 1]],
                            ids_to_values[&u],
                            PropositionalConjunction::new(reason.clone()),
                        )?;

                        graph.remove_edge(v, u);
                    }
                }
            }
        }

        Ok(())
    }
}

/// Graph struct which we use to run Ford-Fulkerson
/// Edges are stored using adjacency list (in both directions, as if the edges were undirected)
/// The direction can be recovered from residual_capacities, since those are always equal to 0 or 1
struct Graph {
    adjacency_list: HashMap<usize, Vec<usize>>,
    residual_capacities: HashMap<(usize, usize), i32>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            adjacency_list: HashMap::default(),
            residual_capacities: HashMap::default(),
        }
    }

    // Adds both forward and reverse (residual) edges
    fn add_edge(&mut self, u: usize, v: usize) {
        self.adjacency_list.entry(u).or_default().push(v);
        self.adjacency_list.entry(v).or_default().push(u);

        // Residual capacities: forward is 1, reverse is 0
        let _ = self.residual_capacities.insert((u, v), 1);
        let _ = self.residual_capacities.insert((v, u), 0);
    }

    fn remove_edge(&mut self, u: usize, v: usize) {
        if let Some(neighbors) = self.adjacency_list.get_mut(&u) {
            if let Some(pos) = neighbors.iter().position(|&x| x == v) {
                let _ = neighbors.remove(pos);
            }
        }

        if let Some(neighbors) = self.adjacency_list.get_mut(&v) {
            if let Some(pos) = neighbors.iter().position(|&x| x == u) {
                let _ = neighbors.remove(pos);
            }
        }

        let _ = self.residual_capacities.remove(&(u, v));
        let _ = self.residual_capacities.remove(&(v, u));
    }

    // Depth-First Search to find an augmenting path
    fn dfs(
        &self,
        cur: usize,
        goal: usize,
        visited: &mut HashSet<usize>,
        path: &mut Vec<usize>,
    ) -> bool {
        if cur == goal {
            return true;
        }

        let _ = visited.insert(cur);

        // Go through neighbours and recursively call on edges that have residual_capacity == 1
        let neighbours_option = self.adjacency_list.get(&cur);
        match neighbours_option {
            None => (),
            Some(neighbours) => {
                for &neighbour in neighbours {
                    // Check if neighbour wasn't visited and if the edge has residual capacity == 1
                    // in that direction
                    if !visited.contains(&neighbour)
                        && *self
                            .residual_capacities
                            .get(&(cur, neighbour))
                            .unwrap_or(&0)
                            > 0
                    {
                        path.push(neighbour);
                        if self.dfs(neighbour, goal, visited, path) {
                            return true;
                        }
                        let _ = path.pop();
                    }
                }
            }
        }

        false
    }

    // Runs Ford-Fulkerson for a given source and sink
    // Returns the found max flow and final residual_capacities
    fn ford_fulkerson(&mut self, source: usize, sink: usize) -> i32 {
        let mut max_flow = 0;

        loop {
            let mut visited = HashSet::default();
            let mut path = vec![source];

            if !self.dfs(source, sink, &mut visited, &mut path) {
                break;
            }

            // Augment flow along the path, flipping the edges in the residual graph
            for i in 0..(path.len() - 1) {
                let u = path[i];
                let v = path[i + 1];

                *self.residual_capacities.get_mut(&(u, v)).unwrap() -= 1;
                *self.residual_capacities.get_mut(&(v, u)).unwrap() += 1;
            }

            max_flow += 1;
        }

        max_flow
    }
}

// Runs Tarjan's algorithm finding strongly connected components in the graph
// Implemented based on https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm
fn tarjans_algorithm(graph: &Graph) -> (HashMap<usize, usize>, Vec<Vec<usize>>) {
    let mut index = 0;
    let mut stack = Vec::new();
    let mut indices = HashMap::default();
    let mut lowlink = HashMap::default();
    let mut on_stack = HashSet::default();

    let mut scc_count = 0;
    let mut node_to_scc_root = HashMap::default();
    let mut sccs = Vec::new();

    #[allow(clippy::too_many_arguments, reason = "todo: refactor in future")]
    fn strongconnect(
        node: usize,
        index: &mut i32,
        stack: &mut Vec<usize>,
        indices: &mut HashMap<usize, i32>,
        lowlink: &mut HashMap<usize, i32>,
        on_stack: &mut HashSet<usize>,
        scc_count: &mut i32,
        graph: &Graph,
        node_to_scc_root: &mut HashMap<usize, usize>,
        sccs: &mut Vec<Vec<usize>>,
    ) {
        // Set the depth index for the node to the smallest unused index
        let _ = indices.insert(node, *index);
        let _ = lowlink.insert(node, *index);
        *index += 1;
        stack.push(node);
        let _ = on_stack.insert(node);

        let neighbours_option = graph.adjacency_list.get(&node);
        match neighbours_option {
            None => (),
            Some(neighbours) => {
                for &neighbour in neighbours {
                    if *graph
                        .residual_capacities
                        .get(&(node, neighbour))
                        .unwrap_or(&0)
                        > 0
                    {
                        if !indices.contains_key(&neighbour) {
                            // Successor of the node (the neighbour) has not yet been visited;
                            // recurse on it
                            strongconnect(
                                neighbour,
                                index,
                                stack,
                                indices,
                                lowlink,
                                on_stack,
                                scc_count,
                                graph,
                                node_to_scc_root,
                                sccs,
                            );
                            *lowlink.get_mut(&node).unwrap() =
                                lowlink[&node].min(lowlink[&neighbour]);
                        } else if on_stack.contains(&neighbour) {
                            // Neighbour is in stack and hence in the current SCC
                            // If neighbour is not on stack, then (node, neighbour) is an edge
                            // pointing to an SCC already found and must
                            // be ignored
                            *lowlink.get_mut(&node).unwrap() =
                                lowlink[&node].min(indices[&neighbour]);
                        }
                    }
                }
            }
        }

        // If node is a root node, pop the stack and generate an SCC, mapping all of its nodes in
        // the `node_to_scc_root` map
        if lowlink[&node] == indices[&node] {
            // Create a new strongly connected component
            let mut new_scc = Vec::new();
            while let Some(w) = stack.pop() {
                let _ = on_stack.remove(&w);
                let _ = node_to_scc_root.insert(w, *scc_count as usize);
                new_scc.push(w);
                if w == node {
                    break;
                }
            }
            *scc_count += 1;
            sccs.push(new_scc);
        }
    }

    // Call strongconnect on each unvisited node
    for &node in graph.adjacency_list.keys() {
        if !indices.contains_key(&node) {
            strongconnect(
                node,
                &mut index,
                &mut stack,
                &mut indices,
                &mut lowlink,
                &mut on_stack,
                &mut scc_count,
                graph,
                &mut node_to_scc_root,
                &mut sccs,
            );
        }
    }

    (node_to_scc_root, sccs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::test_solver::TestSolver;

    #[test]
    fn test_trivial() {
        let mut solver = TestSolver::default();

        let x1 = solver.new_variable(1, 1);
        let x2 = solver.new_variable(1, 2);
        let x3 = solver.new_variable(5, 5);

        let mut equalities: HashMap<(usize, usize), Literal> = HashMap::default();

        let lit_false = solver.new_literal();
        let _ = solver.set_literal(lit_false, false);
        let _ = equalities.insert((0, 1), lit_false);
        let _ = equalities.insert((0, 2), lit_false);
        let _ = equalities.insert((1, 2), lit_false);

        let _ = solver
            .new_propagator(GccInequalitySets::new([x1, x2, x3], equalities))
            .expect("Expected no error");

        solver.assert_bounds(x2, 2, 2);
    }

    #[test]
    fn test_unsat() {
        let mut solver = TestSolver::default();

        let x1 = solver.new_variable(0, 1);
        let x2 = solver.new_variable(0, 1);
        let x3 = solver.new_variable(0, 1);

        let mut equalities: HashMap<(usize, usize), Literal> = HashMap::default();

        let lit_false = solver.new_literal();
        let _ = solver.set_literal(lit_false, false);
        let _ = equalities.insert((0, 1), lit_false);
        let _ = equalities.insert((0, 2), lit_false);
        let _ = equalities.insert((1, 2), lit_false);

        let _ = solver
            .new_propagator(GccInequalitySets::new([x1, x2, x3], equalities))
            .expect_err("Expected error");
    }

    #[test]
    fn test_bounds_propagation() {
        let mut solver = TestSolver::default();

        let x1 = solver.new_variable(0, 3);
        let x2 = solver.new_variable(0, 3);
        let x3 = solver.new_variable(0, 3);
        let x4 = solver.new_variable(1, 2);
        let x5 = solver.new_variable(-2, 6);
        let x6 = solver.new_variable(1, 6);

        let variables = [x1, x2, x3, x4, x5, x6];

        let lit_false = solver.new_literal();
        let _ = solver.set_literal(lit_false, false);

        let mut equalities: HashMap<(usize, usize), Literal> = HashMap::default();
        for i in 0..variables.len() {
            for j in (i + 1)..variables.len() {
                let _ = equalities.insert((i, j), lit_false);
            }
        }

        let _ = solver
            .new_propagator(GccInequalitySets::new(variables, equalities))
            .expect("Expected no error");

        solver.assert_bounds(x5, -2, 6);
        for i in 0..=3 {
            assert!(!solver.contains(x5, i));
        }
        solver.assert_bounds(x6, 4, 6);
    }

    #[test]
    fn test_equal_variables_not_affected() {
        let mut solver = TestSolver::default();

        let x1 = solver.new_variable(0, 3);
        let x2 = solver.new_variable(0, 3);
        let x3 = solver.new_variable(0, 3);
        let x4 = solver.new_variable(1, 2);
        let x5 = solver.new_variable(-2, 6);
        let x6 = solver.new_variable(1, 6);
        let x7 = solver.new_variable(0, 3);

        let variables = [x1, x2, x3, x4, x5, x6, x7];

        let lit_false = solver.new_literal();
        let lit_true = solver.new_literal();
        let lit_unassigned = solver.new_literal();
        let _ = solver.set_literal(lit_false, false);
        let _ = solver.set_literal(lit_true, true);

        let mut equalities: HashMap<(usize, usize), Literal> = HashMap::default();
        for i in 0..6 {
            for j in (i + 1)..6 {
                let _ = equalities.insert((i, j), lit_false);
            }
        }

        let _ = equalities.insert((0, 6), lit_true);

        for i in 1..6 {
            let _ = equalities.insert((i, 6), lit_unassigned);
        }

        let _ = solver
            .new_propagator(GccInequalitySets::new(variables, equalities))
            .expect("Expected no error");

        solver.assert_bounds(x5, -2, 6);
        for i in 0..=3 {
            assert!(!solver.contains(x5, i));
        }
        solver.assert_bounds(x6, 4, 6);

        solver.assert_bounds(x7, 0, 3);
    }
}
