//! Tests persistence of per-object CAM cutter offset modes in GCKD files.

use gcodekit5_designer::DesignerState;
use gcodekit5_designer::model::{DesignRectangle, ToolOffsetMode};
use gcodekit5_designer::serialization::{DesignFile, ShapeData};
use std::path::Path;

#[test]
fn test_tool_offset_mode_survives_gckd_save_and_load() {
    let scratch_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/temp");
    std::fs::create_dir_all(&scratch_dir).expect("create test scratch directory");
    let file = tempfile::Builder::new()
        .suffix(".gckd")
        .tempfile_in(scratch_dir)
        .expect("create temporary GCKD file");
    let path = file.path();

    let mut state = DesignerState::new();
    let shape_id = state.canvas.add_rectangle(0.0, 0.0, 20.0, 10.0);
    state
        .canvas
        .get_shape_mut(shape_id)
        .expect("shape exists")
        .selected = true;
    for mode in [
        ToolOffsetMode::Center,
        ToolOffsetMode::Internal,
        ToolOffsetMode::External,
    ] {
        state.set_selected_tool_offset_mode(mode);
        assert_eq!(
            state
                .canvas
                .get_shape(shape_id)
                .expect("shape exists")
                .tool_offset_mode,
            mode
        );
        state.save_to_file(path).expect("save GCKD file");

        let mut restored = DesignerState::new();
        restored.load_from_file(path).expect("load GCKD file");
        let restored_mode = restored
            .canvas
            .get_shape(shape_id)
            .expect("restored shape exists")
            .tool_offset_mode;
        assert_eq!(restored_mode, mode);
    }
}

#[test]
fn test_missing_tool_offset_mode_defaults_to_center() {
    let mut shape = gcodekit5_designer::canvas::DrawingObject::new(
        1,
        gcodekit5_designer::model::Shape::Rectangle(DesignRectangle::new(0.0, 0.0, 20.0, 10.0)),
    );
    shape.tool_offset_mode = ToolOffsetMode::External;
    let data = DesignFile::from_drawing_object(&shape);
    let mut json = serde_json::to_value(data).expect("serialize shape data");
    json.as_object_mut()
        .expect("shape data is an object")
        .remove("tool_offset_mode");

    let legacy: ShapeData = serde_json::from_value(json).expect("deserialize legacy shape data");
    let restored = DesignFile::to_drawing_object(&legacy, 1).expect("restore legacy shape");
    assert_eq!(restored.tool_offset_mode, ToolOffsetMode::Center);
}
