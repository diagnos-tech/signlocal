//! "Reduce motion" (`docs/ux.md` §11.4): `SPI_GETCLIENTAREAANIMATION`,
//! `accessibilityDisplayShouldReduceMotion`, GNOME `enable-animations`.

/// Whether the OS asks for no animations; `false` when it cannot be read
/// (animations are short and never carry meaning, so the default is safe).
pub fn reduce_motion() -> bool {
    super::os::settings::reduce_motion()
}
