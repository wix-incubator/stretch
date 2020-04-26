#[test]
fn display_grid_column_templates() {
    let mut stretch = stretch::Stretch::new();
    let node0 = stretch
        .new_node(
            stretch::style::Style {
                grid_template: stretch::style::GridTemplate { rows: vec![], columns: vec![] },
                grid_item: stretch::style::GridItem {
                    row: stretch::style::GridItemPlacement::ImplicitSpan {
                        start: stretch::style::GridLine::FromStart(1u16),
                        end: stretch::style::GridLine::FromStart(1u16),
                    },
                    column: stretch::style::GridItemPlacement::ImplicitSpan {
                        start: stretch::style::GridLine::FromStart(1u16),
                        end: stretch::style::GridLine::FromStart(1u16),
                    },
                },
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node1 = stretch
        .new_node(
            stretch::style::Style {
                grid_template: stretch::style::GridTemplate { rows: vec![], columns: vec![] },
                grid_item: stretch::style::GridItem {
                    row: stretch::style::GridItemPlacement::ImplicitSpan {
                        start: stretch::style::GridLine::FromStart(1u16),
                        end: stretch::style::GridLine::FromStart(1u16),
                    },
                    column: stretch::style::GridItemPlacement::ImplicitSpan {
                        start: stretch::style::GridLine::FromStart(2u16),
                        end: stretch::style::GridLine::FromStart(2u16),
                    },
                },
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node2 = stretch
        .new_node(
            stretch::style::Style {
                grid_template: stretch::style::GridTemplate { rows: vec![], columns: vec![] },
                grid_item: stretch::style::GridItem {
                    row: stretch::style::GridItemPlacement::ImplicitSpan {
                        start: stretch::style::GridLine::FromStart(1u16),
                        end: stretch::style::GridLine::FromStart(1u16),
                    },
                    column: stretch::style::GridItemPlacement::ImplicitSpan {
                        start: stretch::style::GridLine::FromStart(3u16),
                        end: stretch::style::GridLine::FromStart(3u16),
                    },
                },
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node3 = stretch
        .new_node(
            stretch::style::Style {
                grid_template: stretch::style::GridTemplate { rows: vec![], columns: vec![] },
                grid_item: stretch::style::GridItem {
                    row: stretch::style::GridItemPlacement::ImplicitSpan {
                        start: stretch::style::GridLine::FromStart(1u16),
                        end: stretch::style::GridLine::FromStart(1u16),
                    },
                    column: stretch::style::GridItemPlacement::ImplicitSpan {
                        start: stretch::style::GridLine::FromStart(4u16),
                        end: stretch::style::GridLine::FromStart(4u16),
                    },
                },
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node4 = stretch
        .new_node(
            stretch::style::Style {
                grid_template: stretch::style::GridTemplate { rows: vec![], columns: vec![] },
                grid_item: stretch::style::GridItem {
                    row: stretch::style::GridItemPlacement::ImplicitSpan {
                        start: stretch::style::GridLine::FromStart(1u16),
                        end: stretch::style::GridLine::FromStart(1u16),
                    },
                    column: stretch::style::GridItemPlacement::ImplicitSpan {
                        start: stretch::style::GridLine::FromStart(5u16),
                        end: stretch::style::GridLine::FromStart(5u16),
                    },
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
                grid_template: stretch::style::GridTemplate {
                    rows: vec![],
                    columns: vec![
                        stretch::style::TrackSizingFunction::Inflexible(stretch::style::InflexibleSize::MinContent),
                        stretch::style::TrackSizingFunction::Inflexible(stretch::style::InflexibleSize::MaxContent),
                        stretch::style::TrackSizingFunction::Flex(1f32),
                    ],
                },
                grid_item: stretch::style::GridItem { row: Default::default(), column: Default::default() },
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
    assert_eq!(stretch.layout(node2).unwrap().size.width, 200f32);
    assert_eq!(stretch.layout(node2).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node2).unwrap().location.x, 0f32);
    assert_eq!(stretch.layout(node2).unwrap().location.y, 0f32);
    assert_eq!(stretch.layout(node3).unwrap().size.width, 0f32);
    assert_eq!(stretch.layout(node3).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node3).unwrap().location.x, 200f32);
    assert_eq!(stretch.layout(node3).unwrap().location.y, 0f32);
    assert_eq!(stretch.layout(node4).unwrap().size.width, 0f32);
    assert_eq!(stretch.layout(node4).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node4).unwrap().location.x, 200f32);
    assert_eq!(stretch.layout(node4).unwrap().location.y, 0f32);
}
