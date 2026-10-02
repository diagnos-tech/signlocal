//! The representative UI: embedded font, a button and a list. Shared by the
//! window path and the headless kittest path so both draw identical content.

use std::sync::Arc;

use egui::{Context, FontData, FontDefinitions, FontFamily, Ui};

/// Family name under which the embedded font is registered.
const EMBEDDED_FAMILY: &str = "embedded-dejavu";

/// Certificates shown in the sample list.
const SAMPLE_CERTIFICATES: [&str; 4] = [
    "Ana Souza - A3 token (PKCS#11)",
    "Jean Dupont - Windows Personal store",
    "Maria Rossi - macOS Keychain",
    "Hans Muller - Smart card (PC/SC)",
];

/// Installs the embedded font as the first choice for proportional text, so
/// rendering never depends on fonts installed on the machine.
pub fn install_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        EMBEDDED_FAMILY.to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/DejaVuSans.ttf"
        ))),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, EMBEDDED_FAMILY.to_owned());
    ctx.set_fonts(fonts);
}

/// Per-frame UI state.
#[derive(Debug, Default)]
pub struct UiState {
    /// Index of the selected certificate.
    pub selected: usize,
    /// How many times the button was pressed.
    pub confirmations: u32,
}

/// Draws the sample confirmation window content.
pub fn draw(ui: &mut Ui, state: &mut UiState, backend_label: &str) {
    ui.heading("Sign document?");
    ui.label("Choose a certificate. Ação, coração, größe: accents render from the embedded font.");
    ui.separator();
    for (index, name) in SAMPLE_CERTIFICATES.iter().enumerate() {
        ui.selectable_value(&mut state.selected, index, *name);
    }
    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Confirm").clicked() {
            state.confirmations += 1;
        }
        ui.label(format!("confirmations: {}", state.confirmations));
    });
    ui.small(format!("renderer: {backend_label}"));
}
