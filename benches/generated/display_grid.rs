pub fn compute() {
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
    let node = stretch
        .new_node(
            stretch::style::Style {
                display: stretch::style::Display::Grid,
                grid_template: stretch::style::GridTemplate { rows: vec![], columns: vec![] },
                grid_item: stretch::style::GridItem { row: Default::default(), column: Default::default() },
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
}
