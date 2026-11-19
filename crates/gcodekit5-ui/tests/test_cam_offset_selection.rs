//! Tests that CAM offset radio-button changes reach the selected design object.

use gcodekit5_core::shared;
use gcodekit5_designer::DesignerState;
use gcodekit5_designer::designer_state::MachineMode;
use gcodekit5_designer::model::{Shape, ToolOffsetMode};
use gcodekit5_settings::SettingsPersistence;
use gcodekit5_ui::ui::gtk::designer_properties::PropertiesPanel;
use gtk4::prelude::*;

fn find_external_button(widget: &gtk4::Widget) -> Option<gtk4::CheckButton> {
    if let Ok(button) = widget.clone().downcast::<gtk4::CheckButton>()
        && matches!(
            button.label().as_deref(),
            Some("External" | "Extérieur" | "Außen" | "Esterno" | "Externo")
        )
    {
        return Some(button);
    }

    let mut child = widget.first_child();
    while let Some(current) = child {
        if let Some(button) = find_external_button(&current) {
            return Some(button);
        }
        child = current.next_sibling();
    }
    None
}

#[test]
fn test_cam_external_radio_updates_selected_shape() {
    if !crate::gtk_available("test_cam_external_radio_updates_selected_shape") {
        return;
    }

    let state = shared(DesignerState::new());
    state.borrow_mut().set_machine_mode(MachineMode::Cnc3D);
    let shape_id = state
        .borrow_mut()
        .canvas
        .add_rectangle(0.0, 0.0, 20.0, 10.0);
    {
        let mut state = state.borrow_mut();
        let shape = state.canvas.get_shape_mut(shape_id).expect("shape exists");
        shape.selected = true;
        shape.use_custom_values = true;
    }

    let panel = PropertiesPanel::new(
        state.clone(),
        shared(SettingsPersistence::new()),
        shared(Vec::<Shape>::new()),
    );
    panel.update_from_selection();

    let external =
        find_external_button(panel.widget.upcast_ref()).expect("External radio button exists");
    assert!(
        external.is_sensitive(),
        "CAM custom values should be enabled"
    );
    external.set_active(true);

    assert_eq!(
        state
            .borrow()
            .canvas
            .get_shape(shape_id)
            .expect("shape exists")
            .tool_offset_mode,
        ToolOffsetMode::External
    );
}
