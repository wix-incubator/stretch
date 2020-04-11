#[test]
fn display_grid_explicit_fixed() {
    let mut stretch = stretch::Stretch::new();
    let node0 = stretch
        .new_node(
            stretch::style::Style {
                grid_area: stretch::style::GridArea::Manual {
                    row_start: 1i32,
                    row_end: 1i32,
                    column_start: 1i32,
                    column_end: 1i32,
                },
                grid_row_start: stretch::style::GridLine::Nth(1i32),
                grid_row_end: stretch::style::GridLine::Nth(1i32),
                grid_column_start: stretch::style::GridLine::Nth(1i32),
                grid_column_end: stretch::style::GridLine::Nth(1i32),
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node1 = stretch
        .new_node(
            stretch::style::Style {
                grid_area: stretch::style::GridArea::Manual {
                    row_start: 1i32,
                    row_end: 2i32,
                    column_start: 1i32,
                    column_end: 2i32,
                },
                grid_row_start: stretch::style::GridLine::Nth(1i32),
                grid_row_end: stretch::style::GridLine::Nth(1i32),
                grid_column_start: stretch::style::GridLine::Nth(2i32),
                grid_column_end: stretch::style::GridLine::Nth(2i32),
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node = stretch
        .new_node(
            stretch::style::Style {
                display: stretch::style::Display::Grid,
                grid_template_row_bounds: vec![stretch::style::TrackSizeBounds {
                    min: stretch::style::TrackSizeValues::Points(100f32),
                    max: stretch::style::TrackSizeValues::Points(100f32),
                }],
                grid_template_column_bounds: vec![
                    stretch::style::TrackSizeBounds {
                        min: stretch::style::TrackSizeValues::Points(30f32),
                        max: stretch::style::TrackSizeValues::Points(30f32),
                    },
                    stretch::style::TrackSizeBounds {
                        min: stretch::style::TrackSizeValues::Points(70f32),
                        max: stretch::style::TrackSizeValues::Points(70f32),
                    },
                ],
                grid_columns_template: stretch::style::GridTracksTemplate {
                    fill: stretch::style::TrackSizeBounds {
                        min: stretch::style::TrackSizeValues::Auto,
                        max: stretch::style::TrackSizeValues::Auto,
                    },
                    defined: Some(vec![
                        stretch::style::TrackSizeBounds {
                            min: stretch::style::TrackSizeValues::Points(30f32),
                            max: stretch::style::TrackSizeValues::Points(30f32),
                        },
                        stretch::style::TrackSizeBounds {
                            min: stretch::style::TrackSizeValues::Points(70f32),
                            max: stretch::style::TrackSizeValues::Points(70f32),
                        },
                    ]),
                },
                grid_rows_template: stretch::style::GridTracksTemplate {
                    fill: stretch::style::TrackSizeBounds {
                        min: stretch::style::TrackSizeValues::Auto,
                        max: stretch::style::TrackSizeValues::Auto,
                    },
                    defined: Some(vec![stretch::style::TrackSizeBounds {
                        min: stretch::style::TrackSizeValues::Points(100f32),
                        max: stretch::style::TrackSizeValues::Points(100f32),
                    }]),
                },
                size: stretch::geometry::Size {
                    width: stretch::style::Dimension::Points(100f32),
                    height: stretch::style::Dimension::Points(100f32),
                    ..Default::default()
                },
                ..Default::default()
            },
            vec![node0, node1],
        )
        .unwrap();
    stretch.compute_layout(node, stretch::geometry::Size::undefined()).unwrap();
    assert_eq!(stretch.layout(node).unwrap().size.width, 100f32);
    assert_eq!(stretch.layout(node).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node).unwrap().location.x, 0f32);
    assert_eq!(stretch.layout(node).unwrap().location.y, 0f32);
    assert_eq!(stretch.layout(node0).unwrap().size.width, 30f32);
    assert_eq!(stretch.layout(node0).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node0).unwrap().location.x, 0f32);
    assert_eq!(stretch.layout(node0).unwrap().location.y, 0f32);
    assert_eq!(stretch.layout(node1).unwrap().size.width, 70f32);
    assert_eq!(stretch.layout(node1).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node1).unwrap().location.x, 30f32);
    assert_eq!(stretch.layout(node1).unwrap().location.y, 0f32);
}
