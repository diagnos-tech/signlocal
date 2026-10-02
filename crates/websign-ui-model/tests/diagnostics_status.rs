//! SPEC §3.1 (ux §8.1): the traffic light of each tab and the overall one.

use websign_ui_model::diagnostics::status::{
    BrowsersFacts, CertificatesFacts, DevicesFacts, Light, browsers_light, certificates_light,
    devices_light, overall,
};

fn browsers(connected: u32, with_problems: u32, missing: bool) -> Light {
    browsers_light(&BrowsersFacts {
        connected,
        with_problems,
        registration_missing_everywhere: missing,
    })
}

#[test]
fn browsers_are_red_when_none_is_connected() {
    assert_eq!(browsers(0, 0, false), Light::Red);
    assert_eq!(browsers(0, 2, false), Light::Red);
    assert_eq!(BrowsersFacts::default().connected, 0);
    assert_eq!(browsers_light(&BrowsersFacts::default()), Light::Red);
}

#[test]
fn browsers_are_red_when_the_registration_is_missing_everywhere() {
    assert_eq!(browsers(2, 0, true), Light::Red);
    assert_eq!(browsers(0, 0, true), Light::Red);
}

#[test]
fn browsers_are_yellow_when_one_is_connected_but_another_has_problems() {
    assert_eq!(browsers(1, 1, false), Light::Yellow);
    assert_eq!(browsers(3, 2, false), Light::Yellow);
}

#[test]
fn browsers_are_green_when_all_installed_ones_work() {
    assert_eq!(browsers(1, 0, false), Light::Green);
    assert_eq!(browsers(4, 0, false), Light::Green);
}

fn devices(facts: DevicesFacts) -> Light {
    devices_light(&facts)
}

#[test]
fn devices_are_green_when_nothing_is_wrong_or_nothing_is_detected() {
    assert_eq!(devices(DevicesFacts::default()), Light::Green);
}

#[test]
fn devices_are_yellow_for_each_soft_problem() {
    let soft = [
        DevicesFacts {
            devices_without_certificates: 1,
            ..DevicesFacts::default()
        },
        DevicesFacts {
            drivers_failed: 1,
            ..DevicesFacts::default()
        },
        DevicesFacts {
            complement_outdated: true,
            ..DevicesFacts::default()
        },
    ];
    for facts in soft {
        assert_eq!(devices(facts.clone()), Light::Yellow, "{facts:?}");
    }
}

#[test]
fn devices_are_red_when_pcscd_is_stopped_whatever_else_is_true() {
    let stopped = DevicesFacts {
        pcscd_stopped: true,
        ..DevicesFacts::default()
    };
    assert_eq!(devices(stopped.clone()), Light::Red);
    let with_others = DevicesFacts {
        drivers_failed: 2,
        devices_without_certificates: 1,
        complement_outdated: true,
        ..stopped
    };
    assert_eq!(devices(with_others), Light::Red);
}

fn certificates(usable: u32, expiring: u32) -> Light {
    certificates_light(&CertificatesFacts {
        usable,
        expiring_within_30_days: expiring,
    })
}

#[test]
fn certificates_are_red_without_a_usable_one() {
    assert_eq!(certificates(0, 0), Light::Red);
    assert_eq!(certificates(0, 3), Light::Red);
}

#[test]
fn certificates_are_yellow_when_a_usable_one_expires_within_30_days() {
    assert_eq!(certificates(3, 1), Light::Yellow);
    assert_eq!(certificates(1, 1), Light::Yellow);
}

#[test]
fn certificates_are_green_otherwise() {
    assert_eq!(certificates(1, 0), Light::Green);
    assert_eq!(certificates(9, 0), Light::Green);
}

#[test]
fn lights_rank_red_over_yellow_over_green_over_gray() {
    assert!(Light::Red > Light::Yellow);
    assert!(Light::Yellow > Light::Green);
    assert!(Light::Green > Light::Gray);
}

#[test]
fn the_overall_light_is_the_worst_one() {
    use Light::*;
    let table: [(&[Light], Light); 9] = [
        (&[], Gray),
        (&[Gray], Gray),
        (&[Green], Green),
        (&[Green, Gray], Green),
        (&[Gray, Green, Gray], Green),
        (&[Green, Yellow, Gray], Yellow),
        (&[Green, Green, Yellow], Yellow),
        (&[Red, Green], Red),
        (&[Yellow, Red, Green, Gray], Red),
    ];
    for (lights, expected) in table {
        assert_eq!(overall(lights), expected, "{lights:?}");
    }
}
