//! Color, type, spacing and motion tokens (`docs/ux.md` §11.1–§11.4).

/// One theme's colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colors {
    pub bg_canvas: egui::Color32,
    pub bg_surface: egui::Color32,
    pub fg: egui::Color32,
    pub accent: egui::Color32,
}

/// Light theme colors.
pub fn light() -> Colors {
    todo!("ui track: ux.md §11.1")
}

/// Dark theme colors.
pub fn dark() -> Colors {
    todo!("ui track: ux.md §11.1")
}
