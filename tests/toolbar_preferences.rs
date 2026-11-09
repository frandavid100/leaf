use serde_json::json;
use wordcraft_gnome::toolbar::normalize;
#[test]
fn reordered_disabled_controls_survive_normalization_and_unknown_ids_do_not_duplicate_actions() {
    let saved = json!([
        {"id":"table","visible":true}, {"id":"image","visible":false},
        {"id":"table","visible":false}, {"id":"removed-command","visible":true}
    ]);
    let order = normalize("elements", &saved);
    assert_eq!(
        &order[..2],
        &[("table".to_owned(), true), ("image".to_owned(), false)]
    );
    assert_eq!(order.iter().filter(|(id, _)| id == "table").count(), 1);
    assert!(!order.iter().any(|(id, _)| id == "removed-command"));
    let saved = serde_json::Value::Array(
        order
            .iter()
            .map(|(id, on)| json!({"id":id,"visible":on}))
            .collect(),
    );
    assert_eq!(normalize("elements", &saved), order);
    for group in ["paragraph", "text", "elements"] {
        let off = serde_json::Value::Array(
            normalize(group, &json!(null))
                .iter()
                .map(|(id, _)| json!({"id":id,"visible":false}))
                .collect(),
        );
        assert!(normalize(group, &off).iter().all(|(_, on)| !on));
    }
}
