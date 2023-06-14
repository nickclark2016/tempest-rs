use std::{
    collections::{HashMap, VecDeque, HashSet},
    hash::Hash,
};

pub struct Node<'a, T> {
    data: &'a T,
    adjacencies: Vec<&'a Node<'a, T>>,
}

impl<'a, T: PartialEq> PartialEq for Node<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<'a, T: Eq> Eq for Node<'a, T> {}

impl<'a, T: std::hash::Hash> std::hash::Hash for Node<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<'a, T> Node<'a, T> {
    pub fn add_edge(&'a mut self, neighbor: &'a mut Node<'a, T>) -> &'a mut Self {
        self.adjacencies.push(neighbor);
        self
    }
}

impl<'a, T: PartialEq> Node<'a, T> {
    pub fn remove_edge(&mut self, neighbor: &Node<'a, T>) {
        if let Some(index) = self.adjacencies.iter().position(|n| *n == neighbor) {
            self.adjacencies.remove(index);
        }
    }
}

pub struct Graph<'a, T> {
    nodes: Vec<Node<'a, T>>,
}

impl<'a, T> Graph<'a, T> {
    pub fn new() -> Self {
        Graph { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, data: &'a T) -> &mut Node<'a, T> {
        let node = Node {
            data,
            adjacencies: Vec::new(),
        };
        self.nodes.push(node);
        self.nodes.last_mut().unwrap()
    }
}

impl<'a, T: std::fmt::Display> Graph<'a, T> {
    pub fn pretty_print(&self) {
        for node in &self.nodes {
            let adjacencies = node
                .adjacencies
                .iter()
                .map(|n| n.data.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            println!("{}: [{}]", node.data, adjacencies);
        }
    }
}

impl<'a, T: Eq + Hash> Graph<'a, T> {
    pub fn topo_sort(&'a self) -> Vec<&'a Node<'a, T>> {
        #[cfg(debug_assertions)] {
            if self.has_cycle() {
                panic!("Topologic sort is not defined on graphs with cycles.");
            }
        }

        let mut result = Vec::new();
        let mut in_degree = HashMap::new();

        // Calculate the in-degree of each node
        for node in self.nodes.iter() {
            in_degree.entry(node).or_insert(0);
            for neighbor in node.adjacencies.iter() {
                *in_degree.entry(neighbor).or_insert(0) += 1;
            }
        }

        // Perform topological sort using Kahn's algorithm
        let mut queue = VecDeque::new();

        for node in self.nodes.iter() {
            if in_degree.get(node) == Some(&0) {
                queue.push_back(node);
            }
        }

        while let Some(node) = queue.pop_front() {
            result.push(node);
            for neighbor in node.adjacencies.iter() {
                let neighbor_in_degree = in_degree.get_mut(neighbor).unwrap();
                *neighbor_in_degree -= 1;
                if *neighbor_in_degree == 0 {
                    queue.push_back(neighbor);
                }
            }
        }

        result
    }

    pub fn has_cycle(&self) -> bool {
        let mut stack: Vec<&Node<T>> = Vec::new();
        let mut visited: HashSet<&Node<T>> = HashSet::new();
    
        for node in self.nodes.iter() {
            if visited.contains(node) {
                // If we've already visited this node, we can safely ignore it
                continue;
            }
    
            stack.push(node);
    
            while let Some(current) = stack.pop() {
                visited.insert(current);
    
                for neighbor in current.adjacencies.iter() {
                    if !visited.contains(neighbor) {
                        stack.push(neighbor);
                    } else if stack.contains(&neighbor) {
                        return true;
                    }
                }
            }
        }
    
        false
    }
}
