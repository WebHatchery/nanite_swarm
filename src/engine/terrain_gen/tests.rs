use super::*;

#[test]
fn the_same_seed_gives_the_same_ground_every_time() {
    for (x, y) in [(0, 0), (3, 7), (-4, 11)] {
        assert_eq!(field(42, x, y, 5.0), field(42, x, y, 5.0));
    }
    assert_ne!(field(42, 3, 7, 5.0), field(43, 3, 7, 5.0));
}

#[test]
fn the_field_stays_inside_its_range() {
    for x in -20..20 {
        for y in -20..20 {
            let value = field(7, x, y, 4.0);
            assert!((0.0..=1.0).contains(&value), "{} out of range", value);
        }
    }
}

#[test]
fn neighbouring_tiles_look_like_each_other() {
    // The whole point: a smooth field changes slowly from tile to tile,
    // where an independent roll per tile would not.
    let mut total = 0.0;
    let mut count = 0.0;
    for x in 0..30 {
        for y in 0..30 {
            total += (field(11, x, y, 6.0) - field(11, x + 1, y, 6.0)).abs();
            count += 1.0;
        }
    }
    let average_step = total / count;
    // Two independent uniform rolls differ by 1/3 on average.
    assert!(
        average_step < 0.12,
        "neighbours differ by {} — that is noise, not ground",
        average_step
    );
}

#[test]
fn a_threshold_hands_back_the_share_it_was_asked_for() {
    let mut values: Vec<f32> = (0..1000).map(|i| i as f32 / 1000.0).collect();
    let cut = threshold_for_share(&mut values.clone(), 0.25);
    let above = values.iter().filter(|value| **value >= cut).count();
    assert!(
        (240..=260).contains(&above),
        "asked for a quarter and got {}",
        above
    );
}

#[test]
fn asking_for_none_or_all_is_not_a_special_case_for_the_caller() {
    let mut values: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
    let none = threshold_for_share(&mut values.clone(), 0.0);
    assert!(values.iter().all(|value| *value < none));
    let all = threshold_for_share(&mut values.clone(), 1.0);
    assert!(values.iter().all(|value| *value >= all));
}
