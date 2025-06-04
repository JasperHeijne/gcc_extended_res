use crate::basic_types::{HashMap, HashSet, Inconsistency};
use crate::engine::propagation::LocalId;
use crate::engine::propagation::PropagationContextMut;
use crate::engine::propagation::Propagator;
use crate::engine::propagation::ReadDomains;
use crate::engine::DomainEvents;
use crate::predicate;
use crate::predicates::PropositionalConjunction;
use crate::variables::IntegerVariable;
use crate::variables::Literal;

use rand::seq::SliceRandom;
use rand::thread_rng;

pub(crate) struct GccInequalitySets<Var: IntegerVariable + 'static> {
    variables: Box<[Var]>,
    equalities: HashMap<(usize, usize), Literal>,
}

impl<Var: IntegerVariable> GccInequalitySets<Var> {
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
}

impl<Var: IntegerVariable> Propagator for GccInequalitySets<Var> {
    fn name(&self) -> &str {
        "GCC extended resolution inequality sets"
    }

    fn initialise_at_root(
        &mut self,
        context: &mut crate::engine::propagation::PropagatorInitialisationContext,
    ) -> Result<(), PropositionalConjunction> {
        // In theory, we could run this propagator for every change in domains of variables
        // but this would be rather wasteful, right?
        for (i, var) in self.variables.iter().enumerate() {
            let _ = context.register(var.clone(), DomainEvents::ASSIGN, LocalId::from(i as u32));
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
        // Consider variables in random order
        let mut variable_indices: Vec<usize> = (0..self.variables.len()).collect();
        variable_indices.shuffle(&mut thread_rng());

        let mut inequality_set: HashSet<usize> = HashSet::default();
        let mut found_valid_clique = false;

        for (i, &var_index) in variable_indices.iter().enumerate() {
            inequality_set = HashSet::default();
            let _ = inequality_set.insert(var_index);

            for j in (i + 1)..self.variables.len() {
                let mut all_unequal = true;
                for set_member in inequality_set.clone() {
                    let var = self.get_equality(variable_indices[j], set_member);
                    if !context.is_literal_false(&var) {
                        all_unequal = false;
                        break;
                    }
                }
                if all_unequal {
                    let _ = inequality_set.insert(variable_indices[j]);
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

        let source = 0;

        for i in 1..=vars_len {
            graph.add_edge(source, i);
        }

        for (var_node_id, &var_ind) in chosen_variables.clone().iter().enumerate() {
            for x in self.variables[var_ind].lower_bound(context.assignments)
                ..=self.variables[var_ind].upper_bound(context.assignments)
            {
                if !self.variables[var_ind].contains(&context.assignments, x) {
                    continue;
                }
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

        for i in vars_len + 1..sink {
            graph.add_edge(i, sink);
        }

        let max_flow = graph.ford_fulkerson(source, sink);

        if max_flow < vars_len as i32 {
            // Max flow is too small -> conflict

            // The explanation for the conflict includes the domain description
            // of all variables that are part of the flow + one other variable
            let mut included_vars: HashSet<usize> = HashSet::default();

            // Get variables that perticipate in the flow
            for ((u, v), cap) in graph.residual_capacities {
                if !included_vars.contains(&(v - 1)) && cap == 1 && u > 0 && v > 0 && v <= vars_len
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

            return Err(Inconsistency::Conflict(
                included_vars
                    .iter()
                    .map(|&i| self.variables[chosen_variables[i]].clone())
                    .into_iter()
                    .flat_map(|var| var.describe_domain(context.assignments))
                    .collect(),
            ));
        }

        // We have a valid flow, analyze connected components to prune values

        todo!()
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
        // (those are the ones available in the residual graph)
        let neighbours_option = self.adjacency_list.get(&cur);
        match neighbours_option {
            None => (),
            Some(neighbours) => {
                for &neighbour in neighbours {
                    // Check if neighbour wasn't visited and if the edge has residual capacity == 1 in that direction
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

        return false;
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

        return max_flow;
    }

    fn find_connected_components(&self) -> HashMap<usize, Vec<usize>> {
        // eprintln!("residual capacities: {:?}", self.residual_capacities);
        let mut ccs = HashMap::default();
        for &node in self.adjacency_list.keys() {
            let mut reachable = Vec::new();
            let mut stack = vec![node];
            let mut visited: HashSet<usize> = HashSet::default();

            while let Some(current) = stack.pop() {
                if visited.insert(current) {
                    reachable.push(current);
                    if let Some(neighbors) = self.adjacency_list.get(&current) {
                        for &neighbor in neighbors {
                            if !visited.contains(&neighbor) {
                                if let Some(&capacity) =
                                    self.residual_capacities.get(&(current, neighbor))
                                {
                                    if capacity > 0 {
                                        stack.push(neighbor);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let _ = ccs.insert(node, reachable);
        }
        ccs
    }
}

// Runs Tarjan's algorithm finding strongly connected components in the graph
// Implemented based on https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm
// However, in our case we don't need all the components, just the mapping node -> its component root,
// as just have to check if two nodes are in the same strongly connected component, or not
fn tarjans_algorithm(graph: &Graph) -> HashMap<usize, usize> {
    let mut index = 0;
    let mut stack = Vec::new();
    let mut indices = HashMap::default();
    let mut lowlink = HashMap::default();
    let mut on_stack = HashSet::default();

    let mut scc_count = 0;
    let mut node_to_scc_root = HashMap::default();

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
                            // Successor of the node (the neighbour) has not yet been visited; recurse on it
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
                            );
                            *lowlink.get_mut(&node).unwrap() =
                                lowlink[&node].min(lowlink[&neighbour]);
                        } else if on_stack.contains(&neighbour) {
                            // Neighbour is in stack and hence in the current SCC
                            // If neighbour is not on stack, then (node, neighbour) is an edge pointing to
                            // an SCC already found and must be ignored
                            *lowlink.get_mut(&node).unwrap() =
                                lowlink[&node].min(indices[&neighbour]);
                        }
                    }
                }
            }
        }

        // If node is a root node, pop the stack and generate an SCC, mapping all of its nodes in the `node_to_scc_root` map
        if lowlink[&node] == indices[&node] {
            // create a new strongly connected component
            while let Some(w) = stack.pop() {
                let _ = on_stack.remove(&w);
                let _ = node_to_scc_root.insert(w, *scc_count as usize);
                if w == node {
                    break;
                }
            }
            *scc_count += 1;
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
            );
        }
    }

    return node_to_scc_root;
}
