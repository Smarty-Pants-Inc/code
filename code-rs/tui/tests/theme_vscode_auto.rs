#[test]
fn vscode_auto_prefers_light_when_overridden() {
    code_tui::theme::super_set_vscode_auto_override_for_test(Some(false));
    let t = code_tui::theme::super_get_predefined_theme_for_test(code_core::config_types::ThemeName::Vscode);
    assert!(code_tui::theme::super_is_light_theme(&t));
    code_tui::theme::super_set_vscode_auto_override_for_test(None);
}

#[test]
fn vscode_auto_prefers_dark_when_overridden() {
    code_tui::theme::super_set_vscode_auto_override_for_test(Some(true));
    let t = code_tui::theme::super_get_predefined_theme_for_test(code_core::config_types::ThemeName::Vscode);
    assert!(!code_tui::theme::super_is_light_theme(&t));
    code_tui::theme::super_set_vscode_auto_override_for_test(None);
}
