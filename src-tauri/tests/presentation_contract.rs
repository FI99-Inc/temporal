use temporal_app::{catalog, snapshot};

#[test]
fn every_exposed_scenario_repeats_at_base_and_advanced_time() {
    assert_eq!(catalog().len(), 12);
    for scenario in catalog() {
        for offset in [0, 60, 1440, 4320] {
            let first = serde_json::to_value(snapshot(&scenario.id, offset).unwrap()).unwrap();
            assert_eq!(
                first,
                serde_json::to_value(snapshot(&scenario.id, offset).unwrap()).unwrap()
            );
            assert_eq!(first["scenario"]["id"], scenario.id);
            assert!(
                first["items"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|i| i["source"].as_str().is_some_and(|s| !s.is_empty()))
            );
        }
    }
}

#[test]
fn ipc_rejects_unknown_scenarios_and_out_of_range_clock_values() {
    assert!(snapshot("not-a-scenario", 0).is_err());
    assert!(snapshot("S01", 14 * 1440).is_err());
    assert!(snapshot("S01", u32::MAX).is_err());
    assert!(snapshot("S01", 14 * 1440 - 1).is_ok());
}

#[test]
fn presentation_keeps_real_overdue_and_expired_advice_distinct() {
    let overdue = serde_json::to_value(snapshot("S09", 0).unwrap()).unwrap();
    assert!(
        overdue["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["species"] == "deadline" && i["risk"] == "overdue")
    );
    let next_day = serde_json::to_value(snapshot("S10", 1440).unwrap()).unwrap();
    let items = next_day["items"].as_array().unwrap();
    assert!(
        items
            .iter()
            .any(|i| i["species"] == "suggestion" && i["phase"] == "expired")
    );
    assert!(
        items
            .iter()
            .any(|i| i["species"] == "task" && i["phase"] == "now" && i["start"].is_null())
    );
    assert!(
        items
            .iter()
            .all(|i| i["risk"] != "overdue" && i["species"] != "deadline")
    );
}

#[test]
fn imported_milestone_is_one_anchor_and_source_staleness_is_visible() {
    let imported = serde_json::to_value(snapshot("S08", 0).unwrap()).unwrap();
    let anchors: Vec<_> = imported["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["species"] == "anchor")
        .collect();
    assert_eq!(anchors.len(), 1);
    assert_eq!(anchors[0]["milestone"], true);
    assert_eq!(anchors[0]["ownership"], "Imported fact");
    let stale = serde_json::to_value(snapshot("S12", 60).unwrap()).unwrap();
    assert!(
        stale["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["species"] == "deadline" && i["conditional"] == true)
    );
    assert!(
        stale["sources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["health"] == "stale")
    );
    assert!(stale["items"].as_array().unwrap().iter().any(|i| {
        i["species"] == "anchor"
            && i["conditional"] == true
            && i["facts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["label"] == "Source freshness" && f["value"] == "stale")
    }));
}
