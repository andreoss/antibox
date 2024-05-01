use super::*;

fn tree() -> Vec<MenuNode<u32>> {
    vec![
        MenuNode::group_expanded(
            "Network",
            vec![
                MenuNode::leaf("Firefox", 1),
                MenuNode::group_expanded("Mail", vec![MenuNode::leaf("Thunderbird", 2)]),
            ],
        ),
        MenuNode::separator(),
        MenuNode::group("Game", vec![MenuNode::leaf("Mines", 3)]),
        MenuNode::leaf("Show Desktop", 4),
    ]
}

fn titles(rows: &[FlatRow<u32>]) -> Vec<(String, u16)> {
    rows.iter().map(|r| (r.title.clone(), r.depth)).collect()
}

#[test]
fn flatten_orders_nested_groups_depth_first_with_depths() {
    let rows = flatten_nodes(&tree());
    assert_eq!(
        titles(&rows),
        vec![
            ("Network".to_string(), 0),
            ("Firefox".to_string(), 1),
            ("Mail".to_string(), 1),
            ("Thunderbird".to_string(), 2),
            (String::new(), 0),
            ("Game".to_string(), 0),
            ("Show Desktop".to_string(), 0),
        ]
    );
    assert!(rows[0].is_group());
    assert!(rows[4].is_separator());
    assert_eq!(rows[1].payload(), Some(&1));
    assert_eq!(rows[6].payload(), Some(&4));
}

#[test]
fn collapsed_group_hides_children_until_toggled() {
    let mut nodes = tree();
    let before = flatten_nodes(&nodes);
    assert!(!before.iter().any(|r| r.title == "Mines"));
    assert!(toggle_at(&mut nodes, &[2]));
    let after = flatten_nodes(&nodes);
    let mines = after.iter().find(|r| r.title == "Mines").expect("Mines");
    assert_eq!(mines.depth, 1);
    assert!(toggle_at(&mut nodes, &[2]));
    assert!(!flatten_nodes(&nodes).iter().any(|r| r.title == "Mines"));
}

#[test]
fn toggle_reaches_nested_groups_by_path() {
    let mut nodes = tree();
    assert!(toggle_at(&mut nodes, &[0, 1]));
    let rows = flatten_nodes(&nodes);
    assert!(!rows.iter().any(|r| r.title == "Thunderbird"));
    assert!(!toggle_at(&mut nodes, &[0, 0]));
    assert!(!toggle_at(&mut nodes, &[9]));
}

#[test]
fn filter_keeps_ancestors_of_a_deep_match_and_expands_them() {
    let kept = filter_nodes(&tree(), "thunder");
    assert_eq!(kept.len(), 1);
    let rows = flatten_nodes(&kept);
    assert_eq!(
        titles(&rows),
        vec![
            ("Network".to_string(), 0),
            ("Mail".to_string(), 1),
            ("Thunderbird".to_string(), 2),
        ]
    );
    assert!(rows.iter().filter(|r| r.is_group()).all(|r| matches!(
        r.entry,
        FlatEntry::Group { expanded: true, .. }
    )));
}

#[test]
fn filter_drops_non_matching_branches_and_separators() {
    let kept = filter_nodes(&tree(), "mines");
    let rows = flatten_nodes(&kept);
    assert_eq!(
        titles(&rows),
        vec![("Game".to_string(), 0), ("Mines".to_string(), 1)]
    );
    assert!(!rows.iter().any(FlatRow::is_separator));
    assert!(filter_nodes(&tree(), "zzz").is_empty());
}

#[test]
fn filter_on_group_title_keeps_the_whole_subtree() {
    let kept = filter_nodes(&tree(), "network");
    let rows = flatten_nodes(&kept);
    assert!(rows.iter().any(|r| r.title == "Firefox"));
    assert!(rows.iter().any(|r| r.title == "Thunderbird"));
    assert!(!rows.iter().any(|r| r.title == "Mines"));
}

#[test]
fn filter_is_case_insensitive_and_ignores_mnemonics() {
    let nodes = vec![MenuNode::group(
        "_Utility",
        vec![MenuNode::leaf("_Calculator", 1)],
    )];
    let kept = filter_nodes(&nodes, "CALC");
    let rows = flatten_nodes(&kept);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].title, "_Calculator");
    assert_eq!(rows[1].depth, 1);
}
