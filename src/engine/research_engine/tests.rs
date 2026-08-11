use super::*;

fn tree() -> ResearchTree {
    ResearchTree::from_nodes(vec![
        ResearchNode::new("a", "A", "first", 10.0, vec![], (0.0, 0.0)),
        ResearchNode::new("b", "B", "needs a", 20.0, vec!["a"], (1.0, 0.0)),
        ResearchNode::new("c", "C", "needs a and b", 30.0, vec!["a", "b"], (2.0, 0.0)),
    ])
}

#[test]
fn can_research_requires_prerequisites_and_data() {
    let tree = tree();
    let unlocked = vec![];
    assert!(tree.can_research("a", &unlocked, 10.0));
    assert!(!tree.can_research("b", &unlocked, 100.0)); // prereq "a" missing
    assert!(!tree.can_research("a", &unlocked, 5.0)); // not enough data
}

#[test]
fn can_research_rejects_already_unlocked() {
    let tree = tree();
    let unlocked = vec!["a".to_string()];
    assert!(!tree.can_research("a", &unlocked, 100.0));
}

#[test]
fn can_select_ignores_available_data() {
    let tree = tree();
    let unlocked = vec!["a".to_string()];
    assert!(tree.can_select("b", &unlocked));
    assert!(!tree.can_select("c", &unlocked)); // still needs "b"
}

#[test]
fn available_research_filters_by_prereqs_and_data() {
    let tree = tree();
    let unlocked = vec!["a".to_string()];
    let available = tree.available_research(&unlocked, 20.0);
    assert_eq!(available.len(), 1);
    assert_eq!(available[0].id, "b");
}

#[test]
fn get_connections_pairs_each_node_with_its_prerequisites() {
    let tree = tree();
    let connections = tree.get_connections();
    // "a" has no prereqs, "b" contributes one edge (a->b), "c" contributes two (a->c, b->c).
    assert_eq!(connections.len(), 3);
    assert!(connections
        .iter()
        .any(|(from, to)| from.id == "a" && to.id == "b"));
    assert!(connections
        .iter()
        .any(|(from, to)| from.id == "b" && to.id == "c"));
}

#[test]
fn start_research_requires_selectable_tech() {
    let tree = tree();
    let mut state = ResearchState {
        unlocked: vec![],
        current_research: None,
        research_progress: 5.0,
    };

    assert!(!state.start_research("b", &tree, 100.0)); // prereq missing
    assert!(state.current_research.is_none());

    assert!(state.start_research("a", &tree, 100.0));
    assert_eq!(state.current_research, Some("a".to_string()));
    assert_eq!(state.research_progress, 0.0);
}

#[test]
fn complete_research_unlocks_tech_and_clears_progress() {
    let mut state = ResearchState {
        unlocked: vec![],
        current_research: Some("a".to_string()),
        research_progress: 10.0,
    };

    let completed = state.complete_research();
    assert_eq!(completed, Some("a".to_string()));
    assert!(state.is_unlocked("a"));
    assert_eq!(state.research_progress, 0.0);
    assert!(state.current_research.is_none());
}

#[test]
fn complete_research_is_noop_with_nothing_in_progress() {
    let mut state = ResearchState {
        unlocked: vec![],
        current_research: None,
        research_progress: 0.0,
    };
    assert_eq!(state.complete_research(), None);
}
