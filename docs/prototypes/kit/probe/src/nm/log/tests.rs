use super::*;

fn at(secs: u64, millis: u32) -> SystemTime {
    UNIX_EPOCH + Duration::new(secs, millis * 1_000_000)
}

#[test]
fn formats_utc_timestamps() {
    assert_eq!(timestamp(at(0, 0)), "1970-01-01T00:00:00.000Z");
    assert_eq!(timestamp(at(951_782_400, 5)), "2000-02-29T00:00:00.005Z");
    assert_eq!(
        timestamp(at(1_798_761_599, 999)),
        "2026-12-31T23:59:59.999Z"
    );
}

#[test]
fn client_text_cannot_break_the_line_format() {
    assert_eq!(sanitize("a\nb\r\"c\u{7}"), "abc");
    assert_eq!(sanitize(&"x".repeat(1000)).len(), MAX_FIELD_CHARS);
    assert_eq!(field(None), "-");
    assert_eq!(field(Some("Chrome/141")), "\"Chrome/141\"");
}

#[test]
fn a_disabled_log_writes_nothing_and_does_not_panic() {
    let log = HostLog::disabled();
    log.started(false);
    log.request("ping", 10);
    log.reply("ok", 20, Duration::from_millis(3));
    log.stopped("eof");
}
