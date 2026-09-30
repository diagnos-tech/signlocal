//! Motion tokens (`docs/ux.md` §11.4) and the "reduce motion" switch.
//!
//! Animation durations become 0 when the OS asks for reduced motion; the
//! safety and state timers (arming, holds, timeout) never change, because
//! they protect the person rather than decorate. egui runs reactively, so an
//! animation only repaints while [`animate`] is in progress.

use std::time::Duration;

use egui::style::ScrollAnimation;
use egui::{Context, Id};

/// Hover, press, focus ring (seconds, as egui animates in `f32` seconds).
pub const MOTION_FAST: f32 = 0.120;
/// Expand/collapse, row entry, arming the button, tab switch.
pub const MOTION_BASE: f32 = 0.180;
/// Success check.
pub const MOTION_SLOW: f32 = 0.260;

/// The Sign button's arming delay: longer than a double click (500 ms), so
/// the second click on the site's own button never lands on ours.
pub const DELAY_ARMING: Duration = Duration::from_millis(600);
/// Loading states only appear after this, so fast answers do not flicker.
pub const DELAY_LOADING: Duration = Duration::from_millis(150);
/// "Still reading {device}…".
pub const DELAY_SLOW_HINT: Duration = Duration::from_millis(2000);
pub const HOLD_SUCCESS: Duration = Duration::from_millis(900);
pub const HOLD_SITE_CANCELLED: Duration = Duration::from_millis(1500);
pub const TIMEOUT_REQUEST: Duration = Duration::from_secs(300);

/// The single easing curve (`cubic-bezier(0.33, 1, 0.68, 1)` in CSS).
pub fn easing(t: f32) -> f32 {
    egui::emath::easing::cubic_out(t)
}

/// Where the reduce-motion flag lives in the context's temporary data.
fn flag_id() -> Id {
    Id::new("websign.theme.reduce_motion")
}

/// Records the OS "reduce motion" preference on `ctx` and applies it to
/// egui's own animations (collapsing headers, scrolling into view,
/// tooltips).
pub fn set_reduced(ctx: &Context, reduce: bool) {
    ctx.data_mut(|data| data.insert_temp(flag_id(), reduce));
    ctx.all_styles_mut(|style| {
        style.animation_time = duration_of(reduce, MOTION_FAST);
        // Keeping a focused row or the PIN field in view scrolls; with
        // reduced motion it jumps.
        style.scroll_animation = if reduce {
            ScrollAnimation::none()
        } else {
            ScrollAnimation::default()
        };
    });
}

/// Whether animations are off on `ctx`.
pub fn reduced(ctx: &Context) -> bool {
    ctx.data(|data| data.get_temp(flag_id())).unwrap_or(false)
}

/// `token` seconds, or 0 when motion is reduced.
pub fn duration(ctx: &Context, token: f32) -> f32 {
    duration_of(reduced(ctx), token)
}

fn duration_of(reduce: bool, token: f32) -> f32 {
    if reduce { 0.0 } else { token }
}

/// Animates `id` towards `on` over `token` seconds with the shared easing;
/// returns 0..=1. Instant when motion is reduced.
pub fn animate(ctx: &Context, id: Id, on: bool, token: f32) -> f32 {
    ctx.animate_bool_with_time_and_easing(id, on, duration(ctx, token), easing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduced_motion_zeroes_animations_but_not_timers() {
        let ctx = Context::default();
        assert!(!reduced(&ctx));
        assert_eq!(duration(&ctx, MOTION_BASE), MOTION_BASE);
        set_reduced(&ctx, true);
        assert!(reduced(&ctx));
        assert_eq!(duration(&ctx, MOTION_BASE), 0.0);
        assert_eq!(ctx.global_style().animation_time, 0.0);
        assert_eq!(ctx.global_style().scroll_animation, ScrollAnimation::none());
        assert_eq!(DELAY_ARMING, Duration::from_millis(600));
    }

    #[test]
    fn easing_is_cubic_out() {
        assert_eq!(easing(0.0), 0.0);
        assert_eq!(easing(1.0), 1.0);
        assert!((easing(0.5) - 0.875).abs() < 1e-6);
    }
}
