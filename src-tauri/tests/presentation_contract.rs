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

#[test]
fn the_daily_edit_stays_bounded_and_never_presents_advice_as_an_obligation() {
    for scenario in catalog() {
        for offset in [0, 60, 1440] {
            let view = serde_json::to_value(snapshot(&scenario.id, offset).unwrap()).unwrap();
            let today = &view["today"];
            assert_eq!(today["policy"], "today-v1");
            let group = |name: &str| today[name].as_array().unwrap().len();
            assert!(group("worth_doing") <= 3 && group("loose") <= 2 && group("radar") <= 3);
            for name in ["worth_doing", "loose"] {
                for row in today[name].as_array().unwrap() {
                    let notes = row["notes"].as_array().unwrap();
                    assert!(!notes.is_empty());
                    assert!(notes.iter().any(|note| {
                        let note = note.as_str().unwrap();
                        note.contains("No time is reserved") || note.contains("expires at")
                    }));
                }
            }
        }
    }
}

#[test]
fn the_daily_edit_separates_an_overdue_cutoff_from_advice_that_merely_expired() {
    // A real cutoff that passed stays visible as awareness.
    let overdue = serde_json::to_value(snapshot("S09", 0).unwrap()).unwrap();
    assert!(
        overdue["today"]["radar"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["species"] == "deadline" && row["risk"] == "overdue")
    );
    assert!(
        overdue["today"]["worth_doing"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    // Advice whose proposed day passed is simply gone; it creates no obligation.
    let expired = serde_json::to_value(snapshot("S10", 1440).unwrap()).unwrap();
    let today = &expired["today"];
    assert!(today["fixed"].as_array().unwrap().is_empty());
    assert!(today["radar"].as_array().unwrap().is_empty());
    assert!(
        today["worth_doing"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["risk"].is_null() && row["species"] == "task")
    );
}
