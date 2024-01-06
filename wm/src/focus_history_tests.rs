use super::*;

fn id(n: u32) -> ClientId {
    ClientId(u64::from(n))
}

fn all(_: ClientId) -> bool {
    true
}

#[test]
fn test_most_recent_comes_first() {
    let mut h = FocusHistory::new();
    h.note(id(1));
    h.note(id(2));
    h.note(id(3));
    assert_eq!(h.entries(), [id(3), id(2), id(1)]);
}

#[test]
fn test_refocusing_moves_a_window_to_the_front_without_duplicating() {
    let mut h = FocusHistory::new();
    h.note(id(1));
    h.note(id(2));
    h.note(id(1));
    assert_eq!(h.entries(), [id(1), id(2)]);
}

#[test]
fn test_repeating_the_current_window_is_a_no_op() {
    let mut h = FocusHistory::new();
    h.note(id(1));
    h.note(id(1));
    assert_eq!(h.entries(), [id(1)]);
}

#[test]
fn test_a_closed_window_is_forgotten() {
    let mut h = FocusHistory::new();
    h.note(id(1));
    h.note(id(2));
    h.forget(id(1));
    assert_eq!(h.entries(), [id(2)]);
}

#[test]
fn test_the_ring_is_bounded() {
    let mut h = FocusHistory::new();
    for n in 0..(CAPACITY as u32 + 20) {
        h.note(id(n));
    }
    assert_eq!(h.entries().len(), CAPACITY);
    assert_eq!(h.entries()[0], id(CAPACITY as u32 + 19), "newest kept");
}

#[test]
fn test_order_puts_the_current_window_first_then_the_rest_by_recency() {
    let mut h = FocusHistory::new();
    h.note(id(1));
    h.note(id(2));
    h.note(id(3));
    assert_eq!(h.order(Some(id(3)), all), [id(3), id(2), id(1)]);
}

#[test]
fn test_order_skips_windows_that_are_gone() {
    let mut h = FocusHistory::new();
    h.note(id(1));
    h.note(id(2));
    h.note(id(3));
    let live = |c: ClientId| c != id(2);
    assert_eq!(h.order(Some(id(3)), live), [id(3), id(1)]);
}

#[test]
fn test_step_back_walks_into_the_past() {
    let mut h = FocusHistory::new();
    h.note(id(1));
    h.note(id(2));
    h.note(id(3));
    assert_eq!(h.step_back(Some(id(3)), all), Some(id(2)));
    assert_eq!(h.step_back(Some(id(2)), all), Some(id(1)));
    assert_eq!(h.step_back(Some(id(1)), all), None, "the history ends");
}

#[test]
fn test_step_forward_returns_toward_the_present() {
    let mut h = FocusHistory::new();
    h.note(id(1));
    h.note(id(2));
    h.note(id(3));
    assert_eq!(h.step_forward(Some(id(1)), all), Some(id(2)));
    assert_eq!(h.step_forward(Some(id(2)), all), Some(id(3)));
    assert_eq!(h.step_forward(Some(id(3)), all), None, "already at the present");
}

#[test]
fn test_stepping_skips_windows_that_are_gone() {
    let mut h = FocusHistory::new();
    h.note(id(1));
    h.note(id(2));
    h.note(id(3));
    let live = |c: ClientId| c != id(2);
    assert_eq!(h.step_back(Some(id(3)), live), Some(id(1)));
    assert_eq!(h.step_forward(Some(id(1)), live), Some(id(3)));
}

#[test]
fn test_a_window_focused_as_a_side_effect_does_not_become_the_previous_window() {
    let mut h = FocusHistory::new();
    h.note(id(3));
    h.note(id(2));
    h.note(id(1));
    assert_eq!(h.order(Some(id(1)), all), [id(1), id(2), id(3)]);
    h.note(id(3));
    assert_eq!(
        h.order(Some(id(3)), all),
        [id(3), id(1), id(2)],
        "the window we came from stays next in line"
    );
}
