use super::*;

#[test]
fn client_events_are_ignored() {
    assert!(!line_matters("Event 'new' on client #21818"));
    assert!(!line_matters("Event 'change' on client #21818"));
    assert!(!line_matters("Event 'remove' on client #21818"));
}

#[test]
fn sink_and_source_events_matter() {
    assert!(line_matters("Event 'change' on sink #0"));
    assert!(line_matters("Event 'change' on source #1"));
    assert!(line_matters("Event 'new' on source-output #7"));
    assert!(line_matters("Event 'change' on server #0"));
}

#[test]
fn unrelated_facilities_are_ignored() {
    assert!(!line_matters("Event 'change' on sink-input #3"));
    assert!(!line_matters("Event 'new' on card #0"));
    assert!(!line_matters(""));
    assert!(!line_matters("garbage"));
}
