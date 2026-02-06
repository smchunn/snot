//! Integration tests for the link graph:
//! build link networks -> traverse -> verify

use std::collections::HashSet;
use std::fs;

use snot::query::{self, QueryExecutor};
use snot::vault::Vault;

/// Helper: create a markdown file in the vault.
fn write_note(vault: &Vault, name: &str, content: &str) {
    let path = vault.path.join(format!("{}.md", name));
    fs::write(&path, content).unwrap();
}

/// Helper: ingest all .md files in the vault.
fn index_all(vault: &mut Vault) {
    let entries: Vec<_> = fs::read_dir(&vault.path)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.path())
        .collect();

    for path in entries {
        vault.ingest_file(&path, true).unwrap();
    }
    vault.save().unwrap();
}

/// Build a vault with a linked network:
///
///   meeting -> project-plan -> research
///   meeting -> research
///   journal (orphan, no links)
///
fn setup_linked_vault() -> (tempfile::TempDir, Vault) {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(
        &vault,
        "meeting",
        "---\ntags: [work]\n---\n\n# Meeting\n\nDiscussed [[project-plan]] and [[research]].\n",
    );
    write_note(
        &vault,
        "project-plan",
        "---\ntags: [work]\n---\n\n# Project Plan\n\nSee [[research]] for details.\n",
    );
    write_note(
        &vault,
        "research",
        "---\ntags: [work]\n---\n\n# Research\n\nPure research content.\n",
    );
    write_note(
        &vault,
        "journal",
        "---\ntags: [personal]\n---\n\n# Journal\n\nNo links here.\n",
    );

    index_all(&mut vault);
    (tmp, vault)
}

#[test]
fn test_forward_links() {
    let (_tmp, vault) = setup_linked_vault();

    let links = vault.db.graph().forward_links(&"meeting".to_string());
    assert!(links.contains("project-plan"));
    assert!(links.contains("research"));
    assert_eq!(links.len(), 2);

    let plan_links = vault.db.graph().forward_links(&"project-plan".to_string());
    assert!(plan_links.contains("research"));
    assert_eq!(plan_links.len(), 1);

    let research_links = vault.db.graph().forward_links(&"research".to_string());
    assert!(research_links.is_empty());
}

#[test]
fn test_backlinks() {
    let (_tmp, vault) = setup_linked_vault();

    // research is linked from meeting and project-plan
    let backlinks = vault.db.graph().backlinks(&"research".to_string());
    assert!(backlinks.contains("meeting"));
    assert!(backlinks.contains("project-plan"));
    assert_eq!(backlinks.len(), 2);

    // project-plan is linked from meeting
    let plan_backlinks = vault.db.graph().backlinks(&"project-plan".to_string());
    assert!(plan_backlinks.contains("meeting"));
    assert_eq!(plan_backlinks.len(), 1);

    // meeting has no backlinks
    let meeting_backlinks = vault.db.graph().backlinks(&"meeting".to_string());
    assert!(meeting_backlinks.is_empty());
}

#[test]
fn test_backlinks_via_db() {
    let (_tmp, vault) = setup_linked_vault();

    let backlinks = vault.db.get_backlinks(&"research".to_string());
    let ids: HashSet<_> = backlinks.iter().map(|n| n.id.as_str()).collect();
    assert!(ids.contains("meeting"));
    assert!(ids.contains("project-plan"));
}

#[test]
fn test_neighbors_depth_1() {
    let (_tmp, vault) = setup_linked_vault();

    // meeting -> project-plan, research (forward)
    // meeting has no backlinks, so depth 1 = {project-plan, research}
    let neighbors = vault.db.graph().neighbors(&"meeting".to_string(), 1);
    assert!(neighbors.contains("project-plan"));
    assert!(neighbors.contains("research"));
    assert!(!neighbors.contains("journal"));
    assert_eq!(neighbors.len(), 2);
}

#[test]
fn test_neighbors_depth_2() {
    let (_tmp, vault) = setup_linked_vault();

    // From project-plan at depth 2:
    //   depth 1: research (forward), meeting (backlink)
    //   depth 2: from meeting -> research (already visited), project-plan (start, excluded)
    //            from research -> meeting (already visited), project-plan (start, excluded)
    let neighbors = vault.db.graph().neighbors(&"project-plan".to_string(), 2);
    assert!(neighbors.contains("meeting"));
    assert!(neighbors.contains("research"));
    // journal is isolated, should never appear
    assert!(!neighbors.contains("journal"));
}

#[test]
fn test_shortest_path() {
    let (_tmp, vault) = setup_linked_vault();

    // meeting -> project-plan -> research (direct path exists)
    let path = vault
        .db
        .graph()
        .shortest_path(&"meeting".to_string(), &"research".to_string())
        .unwrap();

    // Could be meeting->research (direct) or meeting->project-plan->research
    // Since meeting links directly to research, shortest is length 2
    assert_eq!(path.first().unwrap(), "meeting");
    assert_eq!(path.last().unwrap(), "research");
    assert!(path.len() <= 3); // at most 3 hops
}

#[test]
fn test_shortest_path_same_node() {
    let (_tmp, vault) = setup_linked_vault();

    let path = vault
        .db
        .graph()
        .shortest_path(&"meeting".to_string(), &"meeting".to_string())
        .unwrap();
    assert_eq!(path, vec!["meeting"]);
}

#[test]
fn test_shortest_path_no_path() {
    let (_tmp, vault) = setup_linked_vault();

    // journal is an orphan, no path to it from meeting
    let path = vault
        .db
        .graph()
        .shortest_path(&"meeting".to_string(), &"journal".to_string());
    assert!(path.is_none());
}

#[test]
fn test_orphan_detection() {
    let (_tmp, vault) = setup_linked_vault();

    let all_ids = vault.db.all_note_ids();
    let linked = vault.db.graph().all_linked_notes();
    let orphans: HashSet<_> = all_ids.difference(&linked).cloned().collect();

    assert!(orphans.contains("journal"));
    assert_eq!(orphans.len(), 1);
}

#[test]
fn test_orphan_query_shorthand() {
    let (_tmp, vault) = setup_linked_vault();

    let parsed = query::parse("orphans").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();
    assert!(ids.contains("journal"));
    assert_eq!(ids.len(), 1);
}

#[test]
fn test_most_linked() {
    let (_tmp, vault) = setup_linked_vault();

    let top = vault.db.graph().most_linked(3);
    // research has 2 backlinks (meeting, project-plan) + 0 forward = links in 2 entries
    // meeting has 2 forward links + 0 backlinks
    // project-plan has 1 forward + 1 backlink
    assert!(!top.is_empty());

    // research should be high in the list because it appears in forward maps of
    // both meeting and project-plan (as target) and in reverse map (as key)
    let top_ids: Vec<_> = top.iter().map(|(id, _)| id.as_str()).collect();
    assert!(top_ids.contains(&"research"));
}

#[test]
fn test_connected_component() {
    let (_tmp, vault) = setup_linked_vault();

    let component = vault.db.graph().connected_component(&"meeting".to_string());
    assert!(component.contains("meeting"));
    assert!(component.contains("project-plan"));
    assert!(component.contains("research"));
    // journal is isolated
    assert!(!component.contains("journal"));
}

#[test]
fn test_links_to_query_shorthand() {
    let (_tmp, vault) = setup_linked_vault();

    // "links_to:research" = notes that link TO research (backlinks of research)
    let parsed = query::parse("links_to:research").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();

    assert!(ids.contains("meeting"));
    assert!(ids.contains("project-plan"));
    assert_eq!(ids.len(), 2);
}

#[test]
fn test_links_from_query_shorthand() {
    let (_tmp, vault) = setup_linked_vault();

    // "links_from:meeting" = forward links from meeting
    let parsed = query::parse("links_from:meeting").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();

    assert!(ids.contains("project-plan"));
    assert!(ids.contains("research"));
    assert_eq!(ids.len(), 2);
}

#[test]
fn test_links_to_query_sql() {
    let (_tmp, vault) = setup_linked_vault();

    let parsed = query::parse("links_to = 'research'").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();

    assert!(ids.contains("meeting"));
    assert!(ids.contains("project-plan"));
}

#[test]
fn test_neighbors_query_shorthand() {
    let (_tmp, vault) = setup_linked_vault();

    let parsed = query::parse("neighbors:project-plan:1").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();

    assert!(ids.contains("meeting"));
    assert!(ids.contains("research"));
}

#[test]
fn test_neighbors_query_sql() {
    let (_tmp, vault) = setup_linked_vault();

    let parsed = query::parse("neighbors('project-plan', 1)").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();

    assert!(ids.contains("meeting"));
    assert!(ids.contains("research"));
}

#[test]
fn test_link_update_on_note_change() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    // Create note A linking to B
    write_note(&vault, "a", "# A\n\nLinks to [[b]].\n");
    write_note(&vault, "b", "# B\n");
    index_all(&mut vault);

    let backlinks = vault.db.graph().backlinks(&"b".to_string());
    assert!(backlinks.contains("a"));

    // Update A to link to C instead of B
    let note_a_path = vault.path.join("a.md");
    fs::write(&note_a_path, "# A Updated\n\nNow links to [[c]].\n").unwrap();
    vault.ingest_file(&note_a_path, true).unwrap();

    // B should no longer have A as backlink
    let backlinks_b = vault.db.graph().backlinks(&"b".to_string());
    assert!(!backlinks_b.contains("a"));

    // C should have A as backlink
    let backlinks_c = vault.db.graph().backlinks(&"c".to_string());
    assert!(backlinks_c.contains("a"));
}

#[test]
fn test_link_cleanup_on_delete() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(&vault, "a", "# A\n\nLinks to [[b]].\n");
    write_note(&vault, "b", "# B\n\nLinks to [[a]].\n");
    index_all(&mut vault);

    // Verify bidirectional links
    assert!(vault
        .db
        .graph()
        .forward_links(&"a".to_string())
        .contains("b"));
    assert!(vault.db.graph().backlinks(&"a".to_string()).contains("b"));

    // Delete A
    let path_a = vault.path.join("a.md");
    vault.delete_file(&path_a).unwrap();

    // B's forward links to A should still exist (B still links to A textually),
    // but A's backlinks entry should be cleaned up
    assert!(
        vault.db.graph().backlinks(&"a".to_string()).is_empty()
            || !vault.db.graph().backlinks(&"a".to_string()).contains("a")
    );
    assert!(!vault
        .db
        .graph()
        .forward_links(&"a".to_string())
        .contains("b"));
}

#[test]
fn test_graph_persistence() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");

    // Create and index
    {
        let mut vault = Vault::init(&vault_path).unwrap();
        write_note(&vault, "a", "# A\n\nLinks to [[b]] and [[c]].\n");
        write_note(&vault, "b", "# B\n\nLinks to [[c]].\n");
        write_note(&vault, "c", "# C\n");
        index_all(&mut vault);
    }

    // Reopen and verify graph is preserved
    {
        let vault = Vault::open(&vault_path).unwrap();

        let a_links = vault.db.graph().forward_links(&"a".to_string());
        assert!(a_links.contains("b"));
        assert!(a_links.contains("c"));

        let c_backlinks = vault.db.graph().backlinks(&"c".to_string());
        assert!(c_backlinks.contains("a"));
        assert!(c_backlinks.contains("b"));

        let path = vault
            .db
            .graph()
            .shortest_path(&"a".to_string(), &"c".to_string());
        assert!(path.is_some());
    }
}

#[test]
fn test_complex_graph_query() {
    let (_tmp, vault) = setup_linked_vault();

    // Find notes tagged "work" that are also neighbors of research
    let parsed = query::parse("tag:work neighbors:research:1").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();

    // research's neighbors at depth 1: meeting (backlink), project-plan (backlink)
    // Both are tagged "work"
    assert!(ids.contains("meeting"));
    assert!(ids.contains("project-plan"));
}

#[test]
fn test_orphans_combined_with_tag() {
    let (_tmp, vault) = setup_linked_vault();

    // orphans AND tag:personal -> should find journal
    let parsed = query::parse("orphans tag:personal").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "journal");

    // orphans AND tag:work -> should find nothing (all work notes are linked)
    let parsed2 = query::parse("orphans tag:work").unwrap();
    let results2 = executor.execute(&parsed2);
    assert_eq!(results2.len(), 0);
}
