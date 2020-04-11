pub fn compute() {
    let mut stretch = stretch::Stretch::new();
    let node0 = stretch
        .new_node(
            stretch::style::Style {
                grid_area: stretch::style::GridArea::Manual {
                    row_start: 1i32,
                    row_end: -1i32,
                    column_start: 1i32,
                    column_end: -1i32,
                },
                grid_row_start: stretch::style::GridLine::Nth(1i32),
                grid_row_end: stretch::style::GridLine::Nth(1i32),
                grid_column_start: stretch::style::GridLine::Nth(-1i32),
                grid_column_end: stretch::style::GridLine::Nth(-1i32),
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
                    row_end: -2i32,
                    column_start: 1i32,
                    column_end: -2i32,
                },
                grid_row_start: stretch::style::GridLine::Nth(1i32),
                grid_row_end: stretch::style::GridLine::Nth(1i32),
                grid_column_start: stretch::style::GridLine::Nth(-2i32),
                grid_column_end: stretch::style::GridLine::Nth(-2i32),
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node = stretch
        .new_node(
            stretch::style::Style {
                display: stretch::style::Display::Grid,
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
