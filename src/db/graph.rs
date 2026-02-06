use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::note::NoteId;

/// Bidirectional link graph stored separately from notes.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LinkGraph {
    /// Outgoing links: note -> set of notes it links to.
    forward: HashMap<NoteId, HashSet<NoteId>>,
    /// Incoming links (backlinks): note -> set of notes that link to it.
    reverse: HashMap<NoteId, HashSet<NoteId>>,
}

impl LinkGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the forward links for a note, updating reverse links accordingly.
    pub fn set_links(&mut self, note_id: &NoteId, links: HashSet<NoteId>) {
        // Remove old reverse entries
        if let Some(old_links) = self.forward.get(note_id) {
            for target in old_links.clone() {
                if let Some(rev) = self.reverse.get_mut(&target) {
                    rev.remove(note_id);
                    if rev.is_empty() {
                        self.reverse.remove(&target);
                    }
                }
            }
        }

        // Add new reverse entries
        for target in &links {
            self.reverse
                .entry(target.clone())
                .or_default()
                .insert(note_id.clone());
        }

        if links.is_empty() {
            self.forward.remove(note_id);
        } else {
            self.forward.insert(note_id.clone(), links);
        }
    }

    /// Remove a note from the graph entirely (both forward and reverse).
    pub fn remove_note(&mut self, note_id: &NoteId) {
        // Remove outgoing links and their reverse entries
        if let Some(old_links) = self.forward.remove(note_id) {
            for target in old_links {
                if let Some(rev) = self.reverse.get_mut(&target) {
                    rev.remove(note_id);
                    if rev.is_empty() {
                        self.reverse.remove(&target);
                    }
                }
            }
        }

        // Remove incoming links and their forward entries
        if let Some(backers) = self.reverse.remove(note_id) {
            for backer in backers {
                if let Some(fwd) = self.forward.get_mut(&backer) {
                    fwd.remove(note_id);
                    if fwd.is_empty() {
                        self.forward.remove(&backer);
                    }
                }
            }
        }
    }

    /// Get all notes that `note_id` links to (outgoing).
    pub fn forward_links(&self, note_id: &NoteId) -> HashSet<NoteId> {
        self.forward.get(note_id).cloned().unwrap_or_default()
    }

    /// Get all notes that link to `note_id` (incoming / backlinks).
    pub fn backlinks(&self, note_id: &NoteId) -> HashSet<NoteId> {
        self.reverse.get(note_id).cloned().unwrap_or_default()
    }

    /// BFS to find all notes within `depth` hops of `start`.
    /// Returns the set of reachable note IDs (excluding `start` itself).
    pub fn neighbors(&self, start: &NoteId, depth: usize) -> HashSet<NoteId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(start.clone());
        queue.push_back((start.clone(), 0));

        while let Some((current, dist)) = queue.pop_front() {
            if dist >= depth {
                continue;
            }

            // Traverse both directions
            let outgoing = self.forward.get(&current).cloned().unwrap_or_default();
            let incoming = self.reverse.get(&current).cloned().unwrap_or_default();

            for neighbor in outgoing.into_iter().chain(incoming) {
                if visited.insert(neighbor.clone()) {
                    queue.push_back((neighbor, dist + 1));
                }
            }
        }

        visited.remove(start);
        visited
    }

    /// BFS shortest path from `from` to `to`.
    /// Returns None if no path exists, otherwise the path as a list of note IDs.
    pub fn shortest_path(&self, from: &NoteId, to: &NoteId) -> Option<Vec<NoteId>> {
        if from == to {
            return Some(vec![from.clone()]);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<NoteId, NoteId> = HashMap::new();

        visited.insert(from.clone());
        queue.push_back(from.clone());

        while let Some(current) = queue.pop_front() {
            let outgoing = self.forward.get(&current).cloned().unwrap_or_default();
            let incoming = self.reverse.get(&current).cloned().unwrap_or_default();

            for neighbor in outgoing.into_iter().chain(incoming) {
                if visited.insert(neighbor.clone()) {
                    parent.insert(neighbor.clone(), current.clone());

                    if neighbor == *to {
                        // Reconstruct path
                        let mut path = vec![to.clone()];
                        let mut cur = to.clone();
                        while let Some(p) = parent.get(&cur) {
                            path.push(p.clone());
                            cur = p.clone();
                        }
                        path.reverse();
                        return Some(path);
                    }

                    queue.push_back(neighbor);
                }
            }
        }

        None
    }

    /// Find the connected component containing `start`.
    #[allow(dead_code)]
    pub fn connected_component(&self, start: &NoteId) -> HashSet<NoteId> {
        let mut component = HashSet::new();
        let mut queue = VecDeque::new();

        component.insert(start.clone());
        queue.push_back(start.clone());

        while let Some(current) = queue.pop_front() {
            let outgoing = self.forward.get(&current).cloned().unwrap_or_default();
            let incoming = self.reverse.get(&current).cloned().unwrap_or_default();

            for neighbor in outgoing.into_iter().chain(incoming) {
                if component.insert(neighbor.clone()) {
                    queue.push_back(neighbor);
                }
            }
        }

        component
    }

    /// Find all note IDs that appear in the graph (have any links at all).
    pub fn all_linked_notes(&self) -> HashSet<NoteId> {
        let mut result = HashSet::new();
        for (k, v) in &self.forward {
            result.insert(k.clone());
            result.extend(v.iter().cloned());
        }
        for (k, v) in &self.reverse {
            result.insert(k.clone());
            result.extend(v.iter().cloned());
        }
        result
    }

    /// Get the N most-linked notes (by total forward + reverse link count).
    pub fn most_linked(&self, n: usize) -> Vec<(NoteId, usize)> {
        let mut counts: HashMap<NoteId, usize> = HashMap::new();

        for (id, links) in &self.forward {
            *counts.entry(id.clone()).or_default() += links.len();
        }
        for (id, links) in &self.reverse {
            *counts.entry(id.clone()).or_default() += links.len();
        }

        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(n);
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_links(pairs: &[(&str, &[&str])]) -> LinkGraph {
        let mut g = LinkGraph::new();
        for (from, tos) in pairs {
            let links: HashSet<NoteId> = tos.iter().map(|s| s.to_string()).collect();
            g.set_links(&from.to_string(), links);
        }
        g
    }

    #[test]
    fn test_forward_and_reverse() {
        let g = make_links(&[("a", &["b", "c"]), ("b", &["c"])]);
        assert_eq!(
            g.forward_links(&"a".into()),
            ["b", "c"].iter().map(|s| s.to_string()).collect()
        );
        assert_eq!(
            g.backlinks(&"c".into()),
            ["a", "b"].iter().map(|s| s.to_string()).collect()
        );
        assert_eq!(
            g.backlinks(&"b".into()),
            ["a"].iter().map(|s| s.to_string()).collect()
        );
    }

    #[test]
    fn test_set_links_updates_reverse() {
        let mut g = make_links(&[("a", &["b", "c"])]);
        // Change a's links from [b,c] to [c,d]
        g.set_links(
            &"a".into(),
            ["c", "d"].iter().map(|s| s.to_string()).collect(),
        );

        assert!(g.backlinks(&"b".into()).is_empty()); // b no longer linked from a
        assert!(g.backlinks(&"c".into()).contains("a"));
        assert!(g.backlinks(&"d".into()).contains("a"));
    }

    #[test]
    fn test_remove_note() {
        let mut g = make_links(&[("a", &["b"]), ("b", &["c"]), ("c", &["a"])]);
        g.remove_note(&"b".into());

        assert!(g.forward_links(&"b".into()).is_empty());
        assert!(g.backlinks(&"b".into()).is_empty());
        // a's forward links should no longer contain b
        assert!(!g.forward_links(&"a".into()).contains("b"));
        // c's backlinks should no longer contain b
        assert!(!g.backlinks(&"c".into()).contains("b"));
    }

    #[test]
    fn test_neighbors() {
        // a -> b -> c -> d
        let g = make_links(&[("a", &["b"]), ("b", &["c"]), ("c", &["d"])]);
        let n1 = g.neighbors(&"a".into(), 1);
        assert!(n1.contains("b"));
        assert!(!n1.contains("c"));

        let n2 = g.neighbors(&"a".into(), 2);
        assert!(n2.contains("b"));
        assert!(n2.contains("c"));
        assert!(!n2.contains("d"));

        let n3 = g.neighbors(&"b".into(), 1);
        assert!(n3.contains("a")); // reverse link
        assert!(n3.contains("c")); // forward link
    }

    #[test]
    fn test_shortest_path() {
        let g = make_links(&[("a", &["b"]), ("b", &["c"]), ("c", &["d"])]);
        let path = g.shortest_path(&"a".into(), &"d".into()).unwrap();
        assert_eq!(path, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_shortest_path_no_path() {
        let g = make_links(&[("a", &["b"]), ("c", &["d"])]);
        assert!(g.shortest_path(&"a".into(), &"d".into()).is_none());
    }

    #[test]
    fn test_shortest_path_same_node() {
        let g = make_links(&[("a", &["b"])]);
        let path = g.shortest_path(&"a".into(), &"a".into()).unwrap();
        assert_eq!(path, vec!["a"]);
    }

    #[test]
    fn test_connected_component() {
        let g = make_links(&[("a", &["b"]), ("b", &["c"]), ("d", &["e"])]);
        let comp = g.connected_component(&"a".into());
        assert!(comp.contains("a"));
        assert!(comp.contains("b"));
        assert!(comp.contains("c"));
        assert!(!comp.contains("d"));
    }

    #[test]
    fn test_most_linked() {
        let g = make_links(&[("a", &["b", "c"]), ("b", &["c"]), ("d", &["c"])]);
        let top = g.most_linked(2);
        // c has 3 backlinks from a, b, d
        assert_eq!(top[0].0, "c");
    }
}
