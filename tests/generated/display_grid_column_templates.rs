#[test]
fn display_grid_column_templates() {
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
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node2 = stretch
        .new_node(
            stretch::style::Style {
                grid_area: stretch::style::GridArea::Manual {
                    row_start: 1i32,
                    row_end: 3i32,
                    column_start: 1i32,
                    column_end: 3i32,
                },
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node3 = stretch
        .new_node(
            stretch::style::Style {
                grid_area: stretch::style::GridArea::Manual {
                    row_start: 1i32,
                    row_end: 4i32,
                    column_start: 1i32,
                    column_end: 4i32,
                },
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node4 = stretch
        .new_node(
            stretch::style::Style {
                grid_area: stretch::style::GridArea::Manual {
                    row_start: 1i32,
                    row_end: 5i32,
                    column_start: 1i32,
                    column_end: 5i32,
                },
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node = stretch
        .new_node(
            stretch::style::Style {
                display: stretch::style::Display::Grid,
                grid_columns_template: stretch::style::GridTracksTemplate {
                    fill: stretch::style::TrackSizeBounds {
                        min: stretch::style::TrackSizeValues::Auto,
                        max: stretch::style::TrackSizeValues::Auto,
                    },
                    defined: Some(vec![
                        stretch::style::TrackSizeBounds {
                            min: stretch::style::TrackSizeValues::MinContent,
                            max: stretch::style::TrackSizeValues::MinContent,
                        },
                        stretch::style::TrackSizeBounds {
                            min: stretch::style::TrackSizeValues::MaxContent,
                            max: stretch::style::TrackSizeValues::MaxContent,
                        },
                        stretch::style::TrackSizeBounds {
                            min: stretch::style::TrackSizeValues::Points(30f32),
                            max: stretch::style::TrackSizeValues::Percent(0.5f32),
                        },
                        stretch::style::TrackSizeBounds {
                            min: stretch::style::TrackSizeValues::Flex(1f32),
                            max: stretch::style::TrackSizeValues::Flex(1f32),
                        },
                    ]),
                },
                size: stretch::geometry::Size {
                    width: stretch::style::Dimension::Points(200f32),
                    height: stretch::style::Dimension::Points(100f32),
                    ..Default::default()
                },
                ..Default::default()
            },
            vec![node0, node1, node2, node3, node4],
        )
        .unwrap();
    stretch.compute_layout(node, stretch::geometry::Size::undefined()).unwrap();
    assert_eq!(stretch.layout(node).unwrap().size.width, 200f32);
    assert_eq!(stretch.layout(node).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node).unwrap().location.x, 0f32);
    assert_eq!(stretch.layout(node).unwrap().location.y, 0f32);
    assert_eq!(stretch.layout(node0).unwrap().size.width, 0f32);
    assert_eq!(stretch.layout(node0).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node0).unwrap().location.x, 0f32);
    assert_eq!(stretch.layout(node0).unwrap().location.y, 0f32);
    assert_eq!(stretch.layout(node1).unwrap().size.width, 0f32);
    assert_eq!(stretch.layout(node1).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node1).unwrap().location.x, 0f32);
    assert_eq!(stretch.layout(node1).unwrap().location.y, 0f32);
    assert_eq!(stretch.layout(node2).unwrap().size.width, 100f32);
    assert_eq!(stretch.layout(node2).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node2).unwrap().location.x, 0f32);
    assert_eq!(stretch.layout(node2).unwrap().location.y, 0f32);
    assert_eq!(stretch.layout(node3).unwrap().size.width, 100f32);
    assert_eq!(stretch.layout(node3).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node3).unwrap().location.x, 100f32);
    assert_eq!(stretch.layout(node3).unwrap().location.y, 0f32);
    assert_eq!(stretch.layout(node4).unwrap().size.width, 0f32);
    assert_eq!(stretch.layout(node4).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node4).unwrap().location.x, 200f32);
    assert_eq!(stretch.layout(node4).unwrap().location.y, 0f32);
}
