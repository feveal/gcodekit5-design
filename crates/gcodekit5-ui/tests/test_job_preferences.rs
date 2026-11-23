//! Tests for Preferences that change what is sent to the machine
//! (owner decision A-02, issue #76): comments, line length and jog direction.

use gcodekit5_ui::device_status;
use gcodekit5_ui::ui::gtk::machine_control::grbl_stream::{
    linearize_program_for_display, prepare_program, prepare_program_with_arc_linearization,
};

#[test]
fn test_comments_are_removed_unless_preserved() {
    let program = "G0 X1 (rapid)\n; header\nG1 X2 ; cut\n(only a comment)\n";
    assert_eq!(
        prepare_program(program, false, 256).expect("ok"),
        "G0 X1\nG1 X2\n"
    );
    assert_eq!(
        prepare_program(program, true, 256).expect("ok"),
        "G0 X1 (rapid)\n; header\nG1 X2 ; cut\n(only a comment)\n"
    );
}

#[test]
fn test_line_longer_than_maximum_is_refused() {
    let long = format!("G1 X1 ({})", "a".repeat(100));
    let err = prepare_program(&format!("G0 X0\n{long}\n"), true, 80).expect_err("too long");
    assert!(err.contains("line 2"), "{err}");
    // Comments are removed before the length is checked.
    assert!(prepare_program(&long, false, 80).is_ok());
    // 0 disables the check.
    assert!(prepare_program(&long, true, 0).is_ok());
}

#[test]
fn test_inverted_axis_jogs_the_other_way() {
    let _guard = crate::global_env_lock();
    device_status::set_jog_inverted('Y', true);
    assert_eq!(device_status::jog_direction('Y'), -1.0);
    assert_eq!(device_status::jog_direction('X'), 1.0);
    device_status::set_jog_inverted('Y', false);
    assert_eq!(device_status::jog_direction('Y'), 1.0);
}

#[test]
fn test_semicolon_inside_parentheses_is_part_of_the_comment() {
    let out = prepare_program("G1 X1 (rough; finish) Y2 F100 ; tail\n", false, 256).expect("ok");
    assert!(out.starts_with("G1 X1"), "{out}");
    assert!(out.contains("Y2 F100"), "{out}");
    assert!(!out.contains("tail") && !out.contains("rough"), "{out}");
}

#[test]
fn test_small_g2_and_g3_arcs_are_linearized() {
    for arc_command in ["G2", "G3"] {
        let program = format!("G21 G90 G0 X0 Y0\n{arc_command} X0.4 Y0 I0.2 J0 F100\n");
        let out = prepare_program_with_arc_linearization(&program, false, 256, true, 0.5, 0.5)
            .expect("prepare program");

        assert!(
            !out.lines()
                .any(|line| line.starts_with(&format!("{arc_command} "))),
            "{out}"
        );
        assert_eq!(
            out.lines().filter(|line| line.starts_with("G1 ")).count(),
            18
        );
        assert!(
            out.lines()
                .last()
                .is_some_and(|line| line.contains("X0.4 Y0"))
        );
        assert!(out.contains("F100"));
    }
}

#[test]
fn test_arc_linearization_respects_threshold_and_disabled_setting() {
    let program = "G21 G90 G0 X0 Y0\nG3 X2 Y0 I1 J0\n";
    assert_eq!(
        prepare_program_with_arc_linearization(program, false, 256, true, 0.5, 0.5)
            .expect("prepare program"),
        program
    );
    assert_eq!(
        prepare_program_with_arc_linearization(program, false, 256, false, 0.5, 0.5)
            .expect("prepare program"),
        program
    );

    let full_circle = "G21 G90 G0 X1 Y0\nG3 X1 Y0 I-1 J0\n";
    assert_eq!(
        prepare_program_with_arc_linearization(full_circle, false, 256, true, 0.5, 0.5)
            .expect("prepare program"),
        full_circle
    );
}

#[test]
fn test_small_radius_or_chord_is_enough_to_linearize() {
    let programs = [
        "G21 G90 G0 X0 Y0\nG3 X0.6 Y0 I0.3 J0.264575\n",
        "G21 G90 G0 X0 Y0\nG3 X0.4 Y0 I0.2 J0.979796\n",
    ];

    for program in programs {
        let out = prepare_program_with_arc_linearization(program, false, 256, true, 0.5, 0.5)
            .expect("prepare program");
        assert!(out.lines().any(|line| line.starts_with("G1 ")), "{out}");
        assert!(!out.lines().any(|line| line.starts_with("G3 ")), "{out}");
    }
}

#[test]
fn test_r_arc_in_inch_program_uses_millimetre_threshold() {
    let program = "G20 G90 G0 X0 Y0\nG3 X0.015748 Y0 R0.007874\n";
    let out = prepare_program_with_arc_linearization(program, false, 256, true, 0.5, 0.5)
        .expect("prepare program");

    assert_eq!(
        out.lines().filter(|line| line.starts_with("G1 ")).count(),
        18
    );
    assert!(
        out.lines()
            .last()
            .is_some_and(|line| line.contains("X0.015748"))
    );
}

#[test]
fn test_arc_expansion_reports_when_segment_limit_would_be_exceeded() {
    let program = "G21 G90 G0 X1000 Y0\nG3 X1000 Y0 I-1000 J0\n";
    let error = prepare_program_with_arc_linearization(program, false, 256, true, 2000.0, 0.5)
        .expect_err("arc expansion must stay bounded");

    assert!(error.contains("line 2"), "{error}");
    assert!(error.contains("maximum is 10000"), "{error}");
}

#[test]
fn test_modal_incremental_arc_mode_is_restored_after_linearization() {
    let program = "G91 G21 G0 X0 Y0\nG3 X0.4 Y0 I0.2 J0\nX2 Y0 I1 J0\n";
    let out = prepare_program_with_arc_linearization(program, false, 256, true, 0.5, 0.5)
        .expect("prepare program");

    assert!(out.contains("G3 X2 Y0 I1 J0"), "{out}");
    assert_eq!(
        out.lines().filter(|line| line.starts_with("G1 ")).count(),
        18
    );
}

#[test]
fn test_non_modal_move_is_not_given_an_inherited_arc_mode() {
    let program = "G21 G90 G0 X0 Y0\nG3 X0.4 Y0 I0.2 J0\nG28 Z0\nX2 Y0 I1 J0\n";
    let out = prepare_program_with_arc_linearization(program, false, 256, true, 0.5, 0.5)
        .expect("prepare program");
    let lines: Vec<_> = out.lines().collect();

    assert_eq!(lines[19], "G28 Z0");
    assert!(lines[20].starts_with("G3 X2 Y0 I1 J0"), "{out}");
}

#[test]
fn test_linearized_arc_preserves_comment_on_last_segment() {
    let program = "G21 G90 G0 X0 Y0\nG3 X0.4 Y0 I0.2 J0 ; small arc\n";
    let out = prepare_program_with_arc_linearization(program, true, 256, true, 0.5, 0.5)
        .expect("prepare program");

    assert!(
        out.lines()
            .last()
            .is_some_and(|line| line.ends_with("; small arc"))
    );
    assert_eq!(out.matches("; small arc").count(), 1);
}

#[test]
fn test_display_linearization_preserves_other_text_and_line_endings() {
    let program = "; header\r\nG21 G90 G0 X0 Y0\r\n\r\nG3 X0.4 Y0 I0.2 J0 ; small\r\nM5\r\n";
    let out = linearize_program_for_display(program, 0.5, 0.5).expect("linearize");

    assert!(
        out.starts_with("; header\r\nG21 G90 G0 X0 Y0\r\n\r\n"),
        "{out:?}"
    );
    assert!(!out.contains("\r\nG3 "), "{out:?}");
    assert!(out.contains(" ; small\r\nM5\r\n"), "{out:?}");
}

#[test]
fn test_designer_sized_arcs_need_a_larger_preview_threshold() {
    let program = concat!(
        "; Shape ID=118, Type=Path\n",
        "; Name: Path\n",
        "; Operation: Profile\n",
        "; Path bounds: (34.621, 238.260) to (42.701, 245.131)\n",
        "G00 X34.6214 Y240.9279   ; Rapid move\n",
        "M4 S50       ; Laser ON\n",
        "G01 X34.6214 Y241.1017 F5000\n",
        "G02 X38.6551 Y245.1310 I4.0230 J0.0064\n",
        "G02 X42.7012 Y241.1017 I0.0125 J-4.0335\n",
        "G02 X41.5345 Y238.2599 I-4.0245 J-0.0082\n",
        "M5          ; Laser OFF\n",
    );

    assert_eq!(
        linearize_program_for_display(program, 0.5, 0.5).expect("0.5 mm preview"),
        program
    );
    let converted = linearize_program_for_display(program, 4.1, 0.5).expect("4.1 mm preview");
    assert!(!converted.lines().any(|line| line.starts_with("G02 ")));
    assert!(converted.contains("; Shape ID=118, Type=Path"));
    assert!(converted.contains("M5          ; Laser OFF"));
}
