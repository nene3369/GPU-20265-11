use std::collections::{HashMap, HashSet, VecDeque};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Unique identifier for a render pass node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PassId(pub u32);

/// Resource usage declaration for a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceAccess {
    Read,
    Write,
    ReadWrite,
}

/// A named GPU resource (texture or buffer) tracked by the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(pub String);

impl ResourceId {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

/// Resource declaration for a pass.
#[derive(Debug, Clone)]
pub struct ResourceBinding {
    pub resource: ResourceId,
    pub access: ResourceAccess,
}

/// Context passed to each pass during execution.
#[derive(Debug)]
pub struct PassContext {
    pub pass_id: PassId,
    pub pass_name: String,
    pub frame_index: u64,
}

/// A render pass node in the graph.
pub struct RenderPassNode {
    pub id: PassId,
    pub name: String,
    pub reads: Vec<ResourceId>,
    pub writes: Vec<ResourceId>,
    /// User-provided execution function -- called during graph execution.
    /// In a real engine this would encode wgpu commands.  For now it is a
    /// callback.
    #[allow(clippy::type_complexity)]
    execute_fn: Option<Box<dyn FnMut(&PassContext) + Send>>,
}

// Manual Debug so the closure field does not block derive.
impl std::fmt::Debug for RenderPassNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderPassNode")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("reads", &self.reads)
            .field("writes", &self.writes)
            .field("has_execute_fn", &self.execute_fn.is_some())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum GraphError {
    CycleDetected,
    UnknownPass(PassId),
    NotCompiled,
    ResourceConflict {
        resource: String,
        pass_a: PassId,
        pass_b: PassId,
    },
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::CycleDetected => write!(f, "Render graph contains a cycle"),
            GraphError::UnknownPass(id) => write!(f, "Unknown pass: {:?}", id),
            GraphError::NotCompiled => write!(f, "Graph not compiled"),
            GraphError::ResourceConflict {
                resource,
                pass_a,
                pass_b,
            } => {
                write!(
                    f,
                    "Resource conflict on '{}' between {:?} and {:?}",
                    resource, pass_a, pass_b
                )
            }
        }
    }
}

impl std::error::Error for GraphError {}

// ---------------------------------------------------------------------------
// RenderGraph
// ---------------------------------------------------------------------------

/// The render graph -- a DAG of passes with resource dependencies.
///
/// Inspired by TQFT cobordism axioms: each pass is a cobordism (state
/// transition) between resource states, and resource sharing between passes
/// corresponds to the gluing axiom.
pub struct RenderGraph {
    passes: HashMap<PassId, RenderPassNode>,
    /// Explicit edges: from -> to (meaning "from" must execute before "to").
    edges: Vec<(PassId, PassId)>,
    next_id: u32,
    /// Compiled execution order (topological sort result).
    execution_order: Option<Vec<PassId>>,
}

impl RenderGraph {
    // -- Construction -------------------------------------------------------

    pub fn new() -> Self {
        Self {
            passes: HashMap::new(),
            edges: Vec::new(),
            next_id: 0,
            execution_order: None,
        }
    }

    /// Add a pass node, returns its unique [`PassId`].
    pub fn add_pass(&mut self, name: &str) -> PassId {
        let id = PassId(self.next_id);
        self.next_id += 1;
        self.passes.insert(
            id,
            RenderPassNode {
                id,
                name: name.to_string(),
                reads: Vec::new(),
                writes: Vec::new(),
                execute_fn: None,
            },
        );
        // Invalidate any previous compilation.
        self.execution_order = None;
        id
    }

    /// Declare which resources a pass reads.
    pub fn set_pass_reads(&mut self, pass: PassId, resources: &[&str]) {
        if let Some(node) = self.passes.get_mut(&pass) {
            node.reads = resources.iter().map(|r| ResourceId::new(r)).collect();
            self.execution_order = None;
        }
    }

    /// Declare which resources a pass writes.
    pub fn set_pass_writes(&mut self, pass: PassId, resources: &[&str]) {
        if let Some(node) = self.passes.get_mut(&pass) {
            node.writes = resources.iter().map(|r| ResourceId::new(r)).collect();
            self.execution_order = None;
        }
    }

    /// Set the execution callback for a pass.
    pub fn set_pass_execute<F>(&mut self, pass: PassId, f: F)
    where
        F: FnMut(&PassContext) + Send + 'static,
    {
        if let Some(node) = self.passes.get_mut(&pass) {
            node.execute_fn = Some(Box::new(f));
        }
    }

    /// Add an explicit dependency edge (`from` must execute before `to`).
    pub fn add_edge(&mut self, from: PassId, to: PassId) {
        self.edges.push((from, to));
        self.execution_order = None;
    }

    // -- Compilation --------------------------------------------------------

    /// Build the execution order via Kahn's algorithm.
    ///
    /// 1. Collect explicit edges.
    /// 2. Infer implicit edges from resource dependencies (write -> read).
    /// 3. Detect write-write conflicts on the same resource when there is no
    ///    ordering between the two writers (neither explicit edge nor
    ///    transitive path).
    /// 4. Topological sort -- error on cycle.
    /// 5. Store result in `execution_order`.
    pub fn compile(&mut self) -> Result<(), GraphError> {
        // Collect all pass ids.
        let all_ids: Vec<PassId> = self.passes.keys().copied().collect();

        // ---- 1.  Start with explicit edges --------------------------------
        let mut edge_set: HashSet<(PassId, PassId)> = HashSet::new();
        for &(from, to) in &self.edges {
            if !self.passes.contains_key(&from) {
                return Err(GraphError::UnknownPass(from));
            }
            if !self.passes.contains_key(&to) {
                return Err(GraphError::UnknownPass(to));
            }
            edge_set.insert((from, to));
        }

        // ---- 2.  Infer implicit edges from resources ----------------------
        // Build resource -> writers / readers maps.
        let mut writers: HashMap<String, Vec<PassId>> = HashMap::new();
        let mut readers: HashMap<String, Vec<PassId>> = HashMap::new();

        for (&pid, node) in &self.passes {
            for w in &node.writes {
                writers.entry(w.0.clone()).or_default().push(pid);
            }
            for r in &node.reads {
                readers.entry(r.0.clone()).or_default().push(pid);
            }
        }

        // A write to resource R before a read of R implies an edge
        // writer -> reader (for every writer-reader pair on the same
        // resource, *unless* they are the same pass).
        for (res, w_list) in &writers {
            if let Some(r_list) = readers.get(res) {
                for &w in w_list {
                    for &r in r_list {
                        if w != r {
                            edge_set.insert((w, r));
                        }
                    }
                }
            }
        }

        // ---- 3.  Detect write-write conflicts -----------------------------
        // Two passes writing the same resource with no ordering between them
        // is a conflict.  We build a quick reachability set from the current
        // edge_set to test ordering.
        for (res, w_list) in &writers {
            if w_list.len() < 2 {
                continue;
            }
            // Build adjacency for reachability among writers.
            let adj = build_adjacency(&edge_set, &all_ids);
            for i in 0..w_list.len() {
                for j in (i + 1)..w_list.len() {
                    let a = w_list[i];
                    let b = w_list[j];
                    if !is_reachable(&adj, a, b) && !is_reachable(&adj, b, a) {
                        return Err(GraphError::ResourceConflict {
                            resource: res.clone(),
                            pass_a: a,
                            pass_b: b,
                        });
                    }
                }
            }
        }

        // ---- 4.  Kahn's topological sort ----------------------------------
        let adj = build_adjacency(&edge_set, &all_ids);
        let mut in_degree: HashMap<PassId, usize> = HashMap::new();
        for &id in &all_ids {
            in_degree.insert(id, 0);
        }
        for &(_, to) in edge_set.iter() {
            *in_degree.entry(to).or_insert(0) += 1;
        }

        let mut queue: VecDeque<PassId> = VecDeque::new();
        // Seed with zero-indegree nodes, sorted by id for deterministic order.
        let mut zero_in: Vec<PassId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();
        zero_in.sort_by_key(|p| p.0);
        for id in zero_in {
            queue.push_back(id);
        }

        let mut order: Vec<PassId> = Vec::with_capacity(all_ids.len());

        while let Some(node) = queue.pop_front() {
            order.push(node);
            if let Some(neighbours) = adj.get(&node) {
                // Sort neighbours for determinism.
                let mut sorted: Vec<PassId> = neighbours.to_vec();
                sorted.sort_by_key(|p| p.0);
                for next in sorted {
                    let deg = in_degree.get_mut(&next).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(next);
                    }
                }
            }
        }

        if order.len() != all_ids.len() {
            return Err(GraphError::CycleDetected);
        }

        self.execution_order = Some(order);
        Ok(())
    }

    // -- Execution ----------------------------------------------------------

    /// Run all passes in compiled order.
    pub fn execute(&mut self, frame_index: u64) {
        let order = match &self.execution_order {
            Some(o) => o.clone(),
            None => return,
        };

        for pid in order {
            // Build context.
            let ctx = {
                let node = self.passes.get(&pid).unwrap();
                PassContext {
                    pass_id: pid,
                    pass_name: node.name.clone(),
                    frame_index,
                }
            };
            // Call the callback (need mutable borrow of the node).
            if let Some(node) = self.passes.get_mut(&pid) {
                if let Some(f) = node.execute_fn.as_mut() {
                    f(&ctx);
                }
            }
        }
    }

    /// Get the compiled execution order (if compiled).
    pub fn execution_order(&self) -> Option<&[PassId]> {
        self.execution_order.as_deref()
    }

    // -- Resource analysis --------------------------------------------------

    /// Return `(first_writer, last_reader)` for a resource.  This tells the
    /// engine when memory can be allocated and freed -- the TQFT "cobordism
    /// boundary" of the resource's lifetime.
    pub fn resource_lifetime(&self, resource: &str) -> Option<(PassId, PassId)> {
        let order = self.execution_order.as_ref()?;

        let rid = ResourceId::new(resource);
        let mut first_writer: Option<PassId> = None;
        let mut last_reader: Option<PassId> = None;

        for &pid in order {
            let node = self.passes.get(&pid)?;
            if node.writes.contains(&rid) && first_writer.is_none() {
                first_writer = Some(pid);
            }
            if node.reads.contains(&rid) {
                last_reader = Some(pid);
            }
        }

        match (first_writer, last_reader) {
            (Some(w), Some(r)) => Some((w, r)),
            _ => None,
        }
    }

    /// Number of pass nodes.
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// Number of explicit edges (does *not* include implicit resource edges).
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helpers (module-private)
// ---------------------------------------------------------------------------

/// Build an adjacency list from a set of directed edges.
fn build_adjacency(
    edges: &HashSet<(PassId, PassId)>,
    _all_ids: &[PassId],
) -> HashMap<PassId, Vec<PassId>> {
    let mut adj: HashMap<PassId, Vec<PassId>> = HashMap::new();
    for &(from, to) in edges {
        adj.entry(from).or_default().push(to);
    }
    adj
}

/// BFS reachability: can we reach `to` from `from`?
fn is_reachable(adj: &HashMap<PassId, Vec<PassId>>, from: PassId, to: PassId) -> bool {
    let mut visited: HashSet<PassId> = HashSet::new();
    let mut queue: VecDeque<PassId> = VecDeque::new();
    queue.push_back(from);
    visited.insert(from);

    while let Some(cur) = queue.pop_front() {
        if cur == to {
            return true;
        }
        if let Some(neighbours) = adj.get(&cur) {
            for &n in neighbours {
                if visited.insert(n) {
                    queue.push_back(n);
                }
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    // 1. Empty graph compiles successfully.
    #[test]
    fn empty_graph_compiles() {
        let mut g = RenderGraph::new();
        assert!(g.compile().is_ok());
        assert_eq!(g.execution_order(), Some(&[][..]));
    }

    // 2. One pass compiles and executes.
    #[test]
    fn single_pass_executes() {
        let mut g = RenderGraph::new();
        let p = g.add_pass("only");
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        g.set_pass_execute(p, move |_ctx| {
            c.fetch_add(1, Ordering::SeqCst);
        });
        g.compile().unwrap();
        g.execute(0);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    // 3. Linear chain A -> B -> C executes in order.
    #[test]
    fn linear_chain() {
        let mut g = RenderGraph::new();
        let a = g.add_pass("A");
        let b = g.add_pass("B");
        let c = g.add_pass("C");
        g.add_edge(a, b);
        g.add_edge(b, c);

        let log = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

        for (pass, name) in [(a, "A"), (b, "B"), (c, "C")] {
            let l = log.clone();
            g.set_pass_execute(pass, move |_ctx| {
                l.lock().unwrap().push(name.to_string());
            });
        }

        g.compile().unwrap();
        g.execute(0);

        let result = log.lock().unwrap().clone();
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    // 4. Diamond dependency: A->B, A->C, B->D, C->D.
    #[test]
    fn diamond_dependency() {
        let mut g = RenderGraph::new();
        let a = g.add_pass("A");
        let b = g.add_pass("B");
        let c = g.add_pass("C");
        let d = g.add_pass("D");

        g.add_edge(a, b);
        g.add_edge(a, c);
        g.add_edge(b, d);
        g.add_edge(c, d);

        g.compile().unwrap();
        let order = g.execution_order().unwrap();

        // A must come first, D must come last; B and C in the middle.
        let pos = |id: PassId| order.iter().position(|&x| x == id).unwrap();
        assert!(pos(a) < pos(b));
        assert!(pos(a) < pos(c));
        assert!(pos(b) < pos(d));
        assert!(pos(c) < pos(d));
    }

    // 5. Cycle detection: A->B->C->A returns CycleDetected.
    #[test]
    fn cycle_detection() {
        let mut g = RenderGraph::new();
        let a = g.add_pass("A");
        let b = g.add_pass("B");
        let c = g.add_pass("C");
        g.add_edge(a, b);
        g.add_edge(b, c);
        g.add_edge(c, a);

        let err = g.compile().unwrap_err();
        assert!(matches!(err, GraphError::CycleDetected));
    }

    // 6. Implicit dependency from resources: write "gbuffer" before read.
    #[test]
    fn implicit_dependency_from_resources() {
        let mut g = RenderGraph::new();
        let writer = g.add_pass("GBufferWrite");
        let reader = g.add_pass("LightingRead");

        g.set_pass_writes(writer, &["gbuffer"]);
        g.set_pass_reads(reader, &["gbuffer"]);

        // No explicit edge -- the graph should infer writer -> reader.
        let log = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

        {
            let l = log.clone();
            g.set_pass_execute(writer, move |_ctx| {
                l.lock().unwrap().push("write".into());
            });
        }
        {
            let l = log.clone();
            g.set_pass_execute(reader, move |_ctx| {
                l.lock().unwrap().push("read".into());
            });
        }

        g.compile().unwrap();
        g.execute(0);

        let result = log.lock().unwrap().clone();
        assert_eq!(result, vec!["write", "read"]);
    }

    // 7. Resource lifetime tracking: first writer to last reader.
    #[test]
    fn resource_lifetime_tracking() {
        let mut g = RenderGraph::new();
        let a = g.add_pass("A");
        let b = g.add_pass("B");
        let c = g.add_pass("C");

        g.set_pass_writes(a, &["tex"]);
        g.set_pass_reads(b, &["tex"]);
        g.set_pass_reads(c, &["tex"]);

        g.add_edge(a, b);
        g.add_edge(b, c);
        g.compile().unwrap();

        let (first, last) = g.resource_lifetime("tex").unwrap();
        assert_eq!(first, a);
        assert_eq!(last, c);
    }

    // 8. Execution callbacks fire in order with correct frame_index.
    #[test]
    fn execution_callbacks_fire() {
        let mut g = RenderGraph::new();
        let a = g.add_pass("A");
        let b = g.add_pass("B");
        let c = g.add_pass("C");

        g.add_edge(a, b);
        g.add_edge(b, c);

        let counter = Arc::new(AtomicU32::new(0));

        // Each callback adds its 1-based position to the counter.
        for (pass, value) in [(a, 1u32), (b, 2), (c, 3)] {
            let ctr = counter.clone();
            g.set_pass_execute(pass, move |ctx| {
                assert_eq!(ctx.frame_index, 42);
                // Accumulate values in execution order.
                let prev = ctr.fetch_add(value, Ordering::SeqCst);
                // A(1) runs first (prev==0), B(2) second (prev==1), C(3) third (prev==3).
                match value {
                    1 => assert_eq!(prev, 0),
                    2 => assert_eq!(prev, 1),
                    3 => assert_eq!(prev, 3),
                    _ => unreachable!(),
                }
            });
        }

        g.compile().unwrap();
        g.execute(42);
        // 1 + 2 + 3 = 6
        assert_eq!(counter.load(Ordering::SeqCst), 6);
    }

    // 9. Two passes writing the same resource without ordering -> conflict.
    #[test]
    fn multiple_writers_conflict() {
        let mut g = RenderGraph::new();
        let a = g.add_pass("A");
        let b = g.add_pass("B");

        g.set_pass_writes(a, &["shared"]);
        g.set_pass_writes(b, &["shared"]);

        // No edge between a and b -> conflict.
        let err = g.compile().unwrap_err();
        assert!(matches!(err, GraphError::ResourceConflict { .. }));
    }

    // 10. Basic bookkeeping: pass_count and edge_count.
    #[test]
    fn pass_count_and_edge_count() {
        let mut g = RenderGraph::new();
        assert_eq!(g.pass_count(), 0);
        assert_eq!(g.edge_count(), 0);

        let a = g.add_pass("A");
        let b = g.add_pass("B");
        let c = g.add_pass("C");

        assert_eq!(g.pass_count(), 3);

        g.add_edge(a, b);
        g.add_edge(b, c);

        assert_eq!(g.edge_count(), 2);
    }
}
