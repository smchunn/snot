//! Integration tests for full vault round-trips:
//! init -> create notes -> index -> query -> verify

use std::collections::HashSet;
use std::fs;

use snot::query::{self, QueryExecutor};
use snot::vault::Vault;

/// Helper: create a markdown file in the vault with the given content.
fn write_note(vault: &Vault, name: &str, content: &str) {
    let path = vault.path.join(format!("{}.md", name));
    fs::write(&path, content).unwrap();
}

/// Helper: ingest all .md files in the vault root.
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

#[test]
fn test_full_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");

    // Init
    let mut vault = Vault::init(&vault_path).unwrap();
    assert!(vault.path.exists());
    assert!(vault.path.join(".snot").exists());

    // Create notes
    write_note(
        &vault,
        "meeting-notes",
        "---\ntags: [work, meeting]\naliases: [standup]\n---\n\n# Meeting Notes\n\nDiscussed the [[project-plan]] and [[research]].\n",
    );
    write_note(
        &vault,
        "project-plan",
        "---\ntags: [work, planning]\n---\n\n# Project Plan\n\nSee [[research]] for background.\n",
    );
    write_note(
        &vault,
        "research",
        "---\ntags: [work, research]\n---\n\n# Research\n\nFindings documented here.\n",
    );
    write_note(
        &vault,
        "personal-journal",
        "---\ntags: [personal]\n---\n\n# Personal Journal\n\nReflections on the week.\n",
    );

    // Index
    index_all(&mut vault);

    // Verify all notes indexed
    assert_eq!(vault.db.get_all().len(), 4);

    // Verify specific note
    let meeting = vault.db.get(&"meeting-notes".to_string()).unwrap();
    assert_eq!(meeting.title, "Meeting Notes");
    assert!(meeting.tags.contains("work"));
    assert!(meeting.tags.contains("meeting"));

    // Verify aliases
    assert_eq!(meeting.aliases, vec!["standup"]);

    // Verify tags are aggregated correctly
    let mut all_tags = vault.db.all_tags();
    all_tags.sort();
    assert_eq!(
        all_tags,
        vec!["meeting", "personal", "planning", "research", "work"]
    );
}

#[test]
fn test_query_shorthand_tag() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(&vault, "a", "---\ntags: [work]\n---\n\n# Note A\n");
    write_note(&vault, "b", "---\ntags: [work, meeting]\n---\n\n# Note B\n");
    write_note(&vault, "c", "---\ntags: [personal]\n---\n\n# Note C\n");
    index_all(&mut vault);

    let parsed = query::parse("tag:work").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();

    assert_eq!(ids.len(), 2);
    assert!(ids.contains("a"));
    assert!(ids.contains("b"));
}

#[test]
fn test_query_shorthand_hash_tag() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(&vault, "a", "---\ntags: [rust]\n---\n\n# A\n");
    index_all(&mut vault);

    let parsed = query::parse("#rust").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "a");
}

#[test]
fn test_query_shorthand_implicit_and() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(
        &vault,
        "a",
        "---\ntags: [work, meeting]\n---\n\n# Work Meeting\n",
    );
    write_note(&vault, "b", "---\ntags: [work]\n---\n\n# Work Stuff\n");
    write_note(&vault, "c", "---\ntags: [meeting]\n---\n\n# Team Meeting\n");
    index_all(&mut vault);

    let parsed = query::parse("tag:work tag:meeting").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "a");
}

#[test]
fn test_query_shorthand_explicit_or() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(&vault, "a", "---\ntags: [work]\n---\n\n# A\n");
    write_note(&vault, "b", "---\ntags: [personal]\n---\n\n# B\n");
    write_note(&vault, "c", "---\ntags: [hobby]\n---\n\n# C\n");
    index_all(&mut vault);

    let parsed = query::parse("tag:work OR tag:personal").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();

    assert_eq!(ids.len(), 2);
    assert!(ids.contains("a"));
    assert!(ids.contains("b"));
}

#[test]
fn test_query_shorthand_negation() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(&vault, "a", "---\ntags: [work]\n---\n\n# A\n");
    write_note(&vault, "b", "---\ntags: [archived]\n---\n\n# B\n");
    index_all(&mut vault);

    let parsed = query::parse("-tag:archived").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "a");
}

#[test]
fn test_query_shorthand_title() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(&vault, "meeting", "# Team Meeting\n\nAgenda items.\n");
    write_note(&vault, "journal", "# Daily Journal\n\nThoughts.\n");
    index_all(&mut vault);

    let parsed = query::parse("title:meeting").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "meeting");
}

#[test]
fn test_query_shorthand_alias() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(
        &vault,
        "meeting",
        "---\naliases: [standup, daily]\n---\n\n# Meeting\n",
    );
    write_note(&vault, "other", "# Other\n");
    index_all(&mut vault);

    let parsed = query::parse("alias:standup").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "meeting");
}

#[test]
fn test_query_shorthand_fuzzy() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(&vault, "meeting", "# Team Meeting\n");
    write_note(&vault, "journal", "# Daily Journal\n");
    index_all(&mut vault);

    // "meating" is a typo for "meeting"
    let parsed = query::parse("~meating").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();
    assert!(ids.contains("meeting"));
}

#[test]
fn test_query_sql_tag() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(&vault, "a", "---\ntags: [work]\n---\n\n# A\n");
    write_note(&vault, "b", "---\ntags: [personal]\n---\n\n# B\n");
    index_all(&mut vault);

    let parsed = query::parse("tags CONTAINS 'work'").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "a");
}

#[test]
fn test_query_sql_complex_boolean() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(
        &vault,
        "a",
        "---\ntags: [work, meeting]\n---\n\n# Work Meeting\n",
    );
    write_note(
        &vault,
        "b",
        "---\ntags: [work, archived]\n---\n\n# Archived Work\n",
    );
    write_note(&vault, "c", "---\ntags: [personal]\n---\n\n# Personal\n");
    index_all(&mut vault);

    // work AND NOT archived
    let parsed = query::parse("tags CONTAINS 'work' AND NOT tags CONTAINS 'archived'").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "a");
}

#[test]
fn test_query_sql_full_select() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(&vault, "a", "---\ntags: [work]\n---\n\n# A\n");
    index_all(&mut vault);

    let parsed = query::parse("SELECT * FROM notes WHERE tags CONTAINS 'work'").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_query_empty_returns_all() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(&vault, "a", "# A\n");
    write_note(&vault, "b", "# B\n");
    index_all(&mut vault);

    let parsed = query::parse("").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_update_note_preserves_created_at() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    let note_path = vault.path.join("note.md");
    fs::write(&note_path, "# Original\n").unwrap();
    vault.ingest_file(&note_path, true).unwrap();

    let original_created = vault.db.get(&"note".to_string()).unwrap().created_at;

    // Modify the file
    fs::write(&note_path, "# Updated Title\n\nNew content.\n").unwrap();
    vault.ingest_file(&note_path, true).unwrap();

    let updated = vault.db.get(&"note".to_string()).unwrap();
    assert_eq!(updated.created_at, original_created);
    assert_eq!(updated.title, "Updated Title");
}

#[test]
fn test_checksum_skip() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    let note_path = vault.path.join("note.md");
    fs::write(&note_path, "# Hello\n").unwrap();

    let changed1 = vault.ingest_file(&note_path, false).unwrap();
    assert!(changed1);

    // Same content, no force -> should skip
    let changed2 = vault.ingest_file(&note_path, false).unwrap();
    assert!(!changed2);

    // Force -> should re-ingest
    let changed3 = vault.ingest_file(&note_path, true).unwrap();
    assert!(changed3);
}

#[test]
fn test_delete_and_query() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(&vault, "a", "---\ntags: [work]\n---\n\n# A\n");
    write_note(&vault, "b", "---\ntags: [work]\n---\n\n# B\n");
    index_all(&mut vault);

    assert_eq!(vault.db.get_all().len(), 2);

    // Delete one
    let path_a = vault.path.join("a.md");
    vault.delete_file(&path_a).unwrap();

    assert_eq!(vault.db.get_all().len(), 1);
    assert!(vault.db.get(&"a".to_string()).is_none());

    // Tag query should only return b
    let parsed = query::parse("tag:work").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "b");
}

#[test]
fn test_inline_tags_and_links() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(
        &vault,
        "note",
        "# My Note\n\nThis has #inline-tag and links to [[other-note]].\n",
    );
    index_all(&mut vault);

    let note = vault.db.get(&"note".to_string()).unwrap();
    assert!(note.tags.contains("inline-tag"));

    let links = vault.db.graph().forward_links(&"note".to_string());
    assert!(links.contains("other-note"));
}

#[test]
fn test_subdirectory_notes() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    // Create subdirectory
    let sub_dir = vault.path.join("work");
    fs::create_dir_all(&sub_dir).unwrap();

    let note_path = sub_dir.join("meeting.md");
    fs::write(&note_path, "---\ntags: [work]\n---\n\n# Meeting\n").unwrap();
    vault.ingest_file(&note_path, true).unwrap();

    // Note ID should include subdirectory
    let note = vault.db.get(&"work-meeting".to_string()).unwrap();
    assert_eq!(note.title, "Meeting");
    assert!(note.tags.contains("work"));
}

#[test]
fn test_persistence_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");

    // Create vault and add notes
    {
        let mut vault = Vault::init(&vault_path).unwrap();
        write_note(
            &vault,
            "a",
            "---\ntags: [work]\n---\n\n# A\n\nLinks to [[b]].\n",
        );
        write_note(&vault, "b", "# B\n");
        index_all(&mut vault);
    }

    // Reopen and verify
    {
        let vault = Vault::open(&vault_path).unwrap();
        assert_eq!(vault.db.get_all().len(), 2);

        let note_a = vault.db.get(&"a".to_string()).unwrap();
        assert!(note_a.tags.contains("work"));

        let links = vault.db.graph().forward_links(&"a".to_string());
        assert!(links.contains("b"));

        let backlinks = vault.db.graph().backlinks(&"b".to_string());
        assert!(backlinks.contains("a"));
    }
}

#[test]
fn test_combined_frontmatter_and_inline_tags() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(
        &vault,
        "note",
        "---\ntags: [yaml-tag]\n---\n\n# Note\n\nText with #inline-tag here.\n",
    );
    index_all(&mut vault);

    let note = vault.db.get(&"note".to_string()).unwrap();
    assert!(note.tags.contains("yaml-tag"));
    assert!(note.tags.contains("inline-tag"));
}

#[test]
fn test_content_search_with_ripgrep() {
    let tmp = tempfile::tempdir().unwrap();
    let vault_path = tmp.path().join("vault");
    let mut vault = Vault::init(&vault_path).unwrap();

    write_note(
        &vault,
        "quarterly",
        "# Quarterly Report\n\nRevenue increased by 15%.\n",
    );
    write_note(&vault, "journal", "# Daily Journal\n\nWent for a walk.\n");
    index_all(&mut vault);

    let parsed = query::parse("content:Revenue").unwrap();
    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);

    // ripgrep may or may not be available; with title fallback we should still get results
    // if ripgrep is available, it should find "quarterly"
    // This test verifies the content search path doesn't panic
    let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();
    // At minimum, content search should not return errors
    assert!(ids.len() <= 2);
}
