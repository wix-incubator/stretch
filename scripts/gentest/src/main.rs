use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use failure::*;
use fantoccini::{Client, Locator};
use futures::{future::Future, stream::Stream, sync::oneshot::channel};
use json;
use log::*;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

fn main() {
    env_logger::init();
    // this requires being run by cargo, which is iffy
    let root_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = root_dir.parent().and_then(Path::parent).unwrap();

    let fixtures_root = repo_root.join("test_fixtures");
    let fixtures = fs::read_dir(fixtures_root).unwrap();

    info!("reading test fixtures from disk");
    let mut fixtures: Vec<_> = fixtures
        .into_iter()
        .filter_map(|a| a.ok())
        .filter(|f| f.path().is_file() && f.path().extension().map(|p| p == "html").unwrap_or(false))
        .map(|f| {
            let fixture_path = f.path().to_path_buf();
            let name = fixture_path.file_stem().unwrap().to_str().unwrap().to_string();
            (name, fixture_path)
        })
        .collect();
    fixtures.sort_unstable_by_key(|f| f.0.clone());

    info!("starting webdriver instance");
    let webdriver_url = "http://localhost:4444";
    let mut webdriver_handle = Command::new("chromedriver").arg("--port=4444").spawn().unwrap();

    // this is silly, but it works
    std::thread::sleep(std::time::Duration::from_secs(1));

    let mut caps = serde_json::map::Map::new();
    let chrome_opts = serde_json::json!({ "args": ["--headless", "--disable-gpu"] });
    caps.insert("goog:chromeOptions".to_string(), chrome_opts.clone());

    use indicatif::ProgressBar;
    let pb = ProgressBar::new(fixtures.len() as u64);
    let (test_descs_sink, mut test_descs) = channel();

    info!("spawning webdriver client and collecting test descriptions");
    tokio::run(
        Client::with_capabilities(webdriver_url, caps.clone())
            .map_err(|e| Error::from(e))
            .and_then(move |client| {
                futures::stream::iter_ok(pb.wrap_iter(fixtures.into_iter()))
                    .and_then(move |(name, fixture_path)| {
                        pb.set_message(&name);
                        test_root_element(client.clone(), name, fixture_path)
                    })
                    .collect()
                    .map(move |descs| {
                        info!("finished collecting descriptions, sending them back to main");
                        let _ = test_descs_sink.send(descs);
                    })
            })
            .map_err(|e| {
                error!("top-level error encountered: {:?}", e);
            }),
    );

    info!("killing webdriver instance...");
    webdriver_handle.kill().unwrap();

    info!("collecting test descriptions from async runtime...");
    let test_descs = loop {
        if let Ok(Some(descs)) = test_descs.try_recv() {
            break descs;
        }
    };

    info!("generating test sources and concatenating...");

    let bench_descs: Vec<_> = test_descs
        .iter()
        .map(|(name, description)| {
            debug!("generating bench contents for {}", &name);
            (name.clone(), generate_bench(description))
        })
        .collect();

    let test_descs: Vec<_> = test_descs
        .iter()
        .map(|(name, description)| {
            debug!("generating test contents for {}", &name);
            (name.clone(), generate_test(name, description))
        })
        .collect();

    let benchmarks: Vec<_> = test_descs
        .iter()
        .map(|(name, _)| {
            let bench_mod = Ident::new(name, Span::call_site());
            quote!(#bench_mod::compute())
        })
        .collect();

    let test_mods = test_descs
        .iter()
        .map(|(name, _)| {
            let name = Ident::new(name, Span::call_site());
            quote!(mod #name;)
        })
        .fold(quote!(), |a, b| quote!(#a #b));

    for (name, bench_body) in bench_descs {
        let mut bench_filename = repo_root.join("benches").join("generated").join(&name);
        bench_filename.set_extension("rs");
        debug!("writing {} to disk...", &name);
        fs::write(bench_filename, bench_body.to_string()).unwrap();
    }

    for (name, test_body) in test_descs {
        let mut test_filename = repo_root.join("tests").join("generated").join(&name);
        test_filename.set_extension("rs");
        debug!("writing {} to disk...", &name);
        fs::write(test_filename, test_body.to_string()).unwrap();
    }

    let bench_mods = quote!(
        use criterion::{criterion_group, criterion_main, Criterion};

        #test_mods

        fn benchmark(c: &mut Criterion) {
            c.bench_function("generated benchmarks", |b| {
                b.iter(|| { #(#benchmarks;)* })
            });
        }

        criterion_group!(benches, benchmark);
        criterion_main!(benches);
    );

    info!("writing generated test file to disk...");
    fs::write(repo_root.join("benches").join("generated").join("mod.rs"), bench_mods.to_string()).unwrap();
    fs::write(repo_root.join("tests").join("generated").join("mod.rs"), test_mods.to_string()).unwrap();

    info!("formatting the source directory");
    Command::new("cargo").arg("fmt").current_dir(repo_root).status().unwrap();
}

fn test_root_element(
    client: Client,
    name: String,
    fixture_path: impl AsRef<Path>,
) -> impl Future<Item = (String, json::JsonValue), Error = Error> {
    let fixture_path = fixture_path.as_ref();

    let url = format!("file://{}", fixture_path.display());
    let nav_client = client.clone();
    let mut locate_client = client.clone();

    nav_client
        .goto(&url)
        .map_err(|e| e.context("navigating to file"))
        .and_then(move |_| {
            locate_client.find(Locator::Css("#test-root")).map_err(|e| e.context("finding #test-root")).and_then(
                |mut root| {
                    root.attr("__stretch_description__")
                        .map_err(|e| e.context("retrieving stretch description from test root"))
                        .and_then(|description_string| {
                            json::parse(&description_string.unwrap())
                                .map(|d| (name, d))
                                .context("parsing test description")
                        })
                },
            )
        })
        .map_err(|c| c.into())
}

fn generate_bench(description: &json::JsonValue) -> TokenStream {
    let node_description = generate_node("node", &description);

    quote!(
        pub fn compute() {
            let mut stretch = stretch::Stretch::new();
            #node_description
            stretch.compute_layout(node, stretch::geometry::Size::undefined()).unwrap();
        }
    )
}

fn generate_test(name: impl AsRef<str>, description: &json::JsonValue) -> TokenStream {
    let name = name.as_ref();
    let name = Ident::new(name, Span::call_site());
    let node_description = generate_node("node", &description);
    let assertions = generate_assertions("node", &description);

    quote!(
        #[test]
        fn #name() {
            let mut stretch = stretch::Stretch::new();
            #node_description
            stretch.compute_layout(node, stretch::geometry::Size::undefined()).unwrap();
            #assertions
        }
    )
}

fn generate_assertions(ident: &str, node: &json::JsonValue) -> TokenStream {
    let layout = &node["layout"];

    let read_f32 = |s: &str| layout[s].as_f32().unwrap();
    let width = read_f32("width");
    let height = read_f32("height");
    let x = read_f32("x");
    let y = read_f32("y");

    let children = {
        let mut c = Vec::new();
        match node["children"] {
            json::JsonValue::Array(ref value) => {
                for i in 0..value.len() {
                    let child = &value[i];
                    c.push(generate_assertions(&format!("{}{}", ident, i), child));
                }
            }
            _ => (),
        };
        c.into_iter().fold(quote!(), |a, b| quote!(#a #b))
    };

    let ident = Ident::new(ident, Span::call_site());

    quote!(
        assert_eq!(stretch.layout(#ident).unwrap().size.width, #width);
        assert_eq!(stretch.layout(#ident).unwrap().size.height, #height);
        assert_eq!(stretch.layout(#ident).unwrap().location.x, #x);
        assert_eq!(stretch.layout(#ident).unwrap().location.y, #y);

        #children
    )
}

fn generate_node(ident: &str, node: &json::JsonValue) -> TokenStream {
    let style = &node["style"];

    let display = match style["display"] {
        json::JsonValue::Short(ref value) => match value.as_ref() {
            "none" => quote!(display: stretch::style::Display::None,),
            "grid" => quote!(display: stretch::style::Display::Grid,),
            _ => quote!(),
        },
        _ => quote!(),
    };

    let position_type = match style["position_type"] {
        json::JsonValue::Short(ref value) => match value.as_ref() {
            "absolute" => quote!(position_type: stretch::style::PositionType::Absolute,),
            _ => quote!(),
        },
        _ => quote!(),
    };

    let direction = match style["direction"] {
        json::JsonValue::Short(ref value) => match value.as_ref() {
            "rtl" => quote!(direction: stretch::style::Direction::RTL,),
            "ltr" => quote!(direction: stretch::style::Direction::LTR,),
            _ => quote!(),
        },
        _ => quote!(),
    };

    let flex_direction = match style["flexDirection"] {
        json::JsonValue::Short(ref value) => match value.as_ref() {
            "row-reverse" => quote!(flex_direction: stretch::style::FlexDirection::RowReverse,),
            "column" => quote!(flex_direction: stretch::style::FlexDirection::Column,),
            "column-reverse" => quote!(flex_direction: stretch::style::FlexDirection::ColumnReverse,),
            _ => quote!(),
        },
        _ => quote!(),
    };

    let flex_wrap = match style["flexWrap"] {
        json::JsonValue::Short(ref value) => match value.as_ref() {
            "wrap" => quote!(flex_wrap: stretch::style::FlexWrap::Wrap,),
            "wrap-reverse" => quote!(flex_wrap: stretch::style::FlexWrap::WrapReverse,),
            _ => quote!(),
        },
        _ => quote!(),
    };

    let overflow = match style["overflow"] {
        json::JsonValue::Short(ref value) => match value.as_ref() {
            "hidden" => quote!(overflow: stretch::style::Overflow::Hidden,),
            "scroll" => quote!(overflow: stretch::style::Overflow::Scroll,),
            _ => quote!(),
        },
        _ => quote!(),
    };

    let align_items = match style["alignItems"] {
        json::JsonValue::Short(ref value) => match value.as_ref() {
            "flex-start" => quote!(align_items: stretch::style::AlignItems::FlexStart,),
            "flex-end" => quote!(align_items: stretch::style::AlignItems::FlexEnd,),
            "center" => quote!(align_items: stretch::style::AlignItems::Center,),
            "baseline" => quote!(align_items: stretch::style::AlignItems::Baseline,),
            _ => quote!(),
        },
        _ => quote!(),
    };

    let align_self = match style["alignSelf"] {
        json::JsonValue::Short(ref value) => match value.as_ref() {
            "flex-start" => quote!(align_self: stretch::style::AlignSelf::FlexStart,),
            "flex-end" => quote!(align_self: stretch::style::AlignSelf::FlexEnd,),
            "center" => quote!(align_self: stretch::style::AlignSelf::Center,),
            "baseline" => quote!(align_self: stretch::style::AlignSelf::Baseline,),
            "stretch" => quote!(align_self: stretch::style::AlignSelf::Stretch,),
            _ => quote!(),
        },
        _ => quote!(),
    };

    let align_content = match style["alignContent"] {
        json::JsonValue::Short(ref value) => match value.as_ref() {
            "flex-start" => quote!(align_content: stretch::style::AlignContent::FlexStart,),
            "flex-end" => quote!(align_content: stretch::style::AlignContent::FlexEnd,),
            "center" => quote!(align_content: stretch::style::AlignContent::Center,),
            "space-between" => quote!(align_content: stretch::style::AlignContent::SpaceBetween,),
            "space-around" => quote!(align_content: stretch::style::AlignContent::SpaceAround,),
            _ => quote!(),
        },
        _ => quote!(),
    };

    let justify_content = match style["justifyContent"] {
        json::JsonValue::Short(ref value) => match value.as_ref() {
            "flex-end" => quote!(justify_content: stretch::style::JustifyContent::FlexEnd,),
            "center" => quote!(justify_content: stretch::style::JustifyContent::Center,),
            "space-between" => quote!(justify_content: stretch::style::JustifyContent::SpaceBetween,),
            "space-around" => quote!(justify_content: stretch::style::JustifyContent::SpaceAround,),
            "space-evenly" => quote!(justify_content: stretch::style::JustifyContent::SpaceEvenly,),
            _ => quote!(),
        },
        _ => quote!(),
    };

    let flex_grow = match style["flexGrow"] {
        json::JsonValue::Number(value) => {
            let value: f32 = value.into();
            quote!(flex_grow: #value,)
        }
        _ => quote!(),
    };

    let flex_shrink = match style["flexShrink"] {
        json::JsonValue::Number(value) => {
            let value: f32 = value.into();
            quote!(flex_shrink: #value,)
        }
        _ => quote!(),
    };

    let flex_basis = match style["flexBasis"] {
        json::JsonValue::Object(ref value) => {
            let value = generate_dimension(value);
            quote!(flex_basis: #value,)
        }
        _ => quote!(),
    };

    let grid_area = match style["gridArea"] {
        json::JsonValue::Object(ref value) => {
            let grid_area = generate_grid_area(value);
            quote!(grid_area: #grid_area,)
        }
        _ => quote!(),
    };

    let grid_template_row_bounds = match style["gridTemplateRowBounds"] {
        json::JsonValue::Array(ref value) => {
            let track_bounds_list = generate_track_bounds_list(value);
            quote!(grid_template_row_bounds: #track_bounds_list,)
        }
        _ => quote!(),
    };

    let grid_template_column_bounds = match style["gridTemplateColumnBounds"] {
        json::JsonValue::Array(ref value) => {
            let track_bounds_list = generate_track_bounds_list(value);
            quote!(grid_template_column_bounds: #track_bounds_list,)
        }
        _ => quote!(),
    };

    let grid_row_start = match style["gridRowStart"] {
        json::JsonValue::Object(ref value) => {
            let grid_line = generate_grid_line(value);
            quote!(grid_row_start: #grid_line,)
        }
        _ => quote!(),
    };

    let grid_row_end = match style["gridRowEnd"] {
        json::JsonValue::Object(ref value) => {
            let grid_line = generate_grid_line(value);
            quote!(grid_row_end: #grid_line,)
        }
        _ => quote!(),
    };

    let grid_column_start = match style["gridColumnStart"] {
        json::JsonValue::Object(ref value) => {
            let grid_line = generate_grid_line(value);
            quote!(grid_column_start: #grid_line,)
        }
        _ => quote!(),
    };

    let grid_column_end = match style["gridColumnEnd"] {
        json::JsonValue::Object(ref value) => {
            let grid_line = generate_grid_line(value);
            quote!(grid_column_end: #grid_line,)
        }
        _ => quote!(),
    };

    let grid_gaps = match style["gridGap"] {
        json::JsonValue::Object(ref value) => {
            let width = match value.get("width") {
                Some(json::JsonValue::Object(ref width_obj)) => width_obj.get("value").unwrap().as_f32().unwrap_or(0.0),
                _ => 0.0,
            };
            let height = match value.get("height") {
                Some(json::JsonValue::Object(ref height_obj)) => {
                    height_obj.get("value").unwrap().as_f32().unwrap_or(0.0)
                }
                _ => 0.0,
            };
            quote!(grid_gaps: stretch::geometry::Size {width: #width, height: #height},)
        }
        _ => quote!(),
    };

    let grid_columns = match style["gridColumns"] {
        json::JsonValue::Array(ref value) => {
            let def = generate_grid_tracks_definition(value);
            quote!(grid_columns_template: #def)
        }
        _ => quote!(),
    };

    let grid_rows = match style["gridRows"] {
        json::JsonValue::Array(ref value) => {
            let def = generate_grid_tracks_definition(value);
            quote!(grid_rows_template: #def)
        }
        _ => quote!(),
    };

    let size = match style["size"] {
        json::JsonValue::Object(ref value) => {
            let size = generate_size(value);
            quote!(size: #size,)
        }
        _ => quote!(),
    };

    let min_size = match style["min_size"] {
        json::JsonValue::Object(ref value) => {
            let min_size = generate_size(value);
            quote!(min_size: #min_size,)
        }
        _ => quote!(),
    };

    let max_size = match style["max_size"] {
        json::JsonValue::Object(ref value) => {
            let max_size = generate_size(value);
            quote!(max_size: #max_size,)
        }
        _ => quote!(),
    };

    macro_rules! edges_quoted {
        ($style:ident, $val:ident) => {
            let $val = match $style[stringify!($val)] {
                json::JsonValue::Object(ref value) => {
                    let edges = generate_edges(value);
                    quote!($val: #edges,)
                },
                _ => quote!(),
            };
        };
    }

    edges_quoted!(style, margin);
    edges_quoted!(style, padding);
    edges_quoted!(style, position);
    edges_quoted!(style, border);

    let (children_body, children) = match node["children"] {
        json::JsonValue::Array(ref value) => {
            if value.len() > 0 {
                let body = value
                    .iter()
                    .enumerate()
                    .map(|(i, child)| generate_node(&format!("{}{}", ident, i), child))
                    .collect();
                let idents = value
                    .iter()
                    .enumerate()
                    .map(|(i, _)| Ident::new(&format!("{}{}", ident, i), Span::call_site()))
                    .collect::<Vec<_>>();
                (body, quote!(vec![#(#idents),*]))
            } else {
                (quote!(), quote!(vec![]))
            }
        }
        _ => (quote!(), quote!()),
    };

    let ident = Ident::new(&format!("{}", ident), Span::call_site());

    quote!(
        #children_body
        let #ident = stretch.new_node(
        stretch::style::Style {
            #display
            #direction
            #position_type
            #flex_direction
            #flex_wrap
            #overflow
            #align_items
            #align_self
            #align_content
            #justify_content
            #flex_grow
            #flex_shrink
            #flex_basis
            #grid_area
            #grid_template_row_bounds
            #grid_template_column_bounds
            #grid_row_start
            #grid_row_end
            #grid_column_start
            #grid_column_end
            #grid_gaps
            #grid_columns
            #grid_rows
            #size
            #min_size
            #max_size
            #margin
            #padding
            #position
            #border
            ..Default::default()
        },
        #children
    ).unwrap();)
}

macro_rules! dim_quoted {
    ($obj:ident, $dim_name:ident) => {
        let $dim_name = match $obj.get(stringify!($dim_name)) {
            Some(json::JsonValue::Object(ref value)) => {
                let dim = generate_dimension(value);
                quote!($dim_name: #dim,)
            }
            _ => quote!(),
        };
    };
}

fn generate_size(size: &json::object::Object) -> TokenStream {
    dim_quoted!(size, width);
    dim_quoted!(size, height);
    quote!(
        stretch::geometry::Size {
            #width #height
            ..Default::default()
        }
    )
}

fn generate_dimension(dimen: &json::object::Object) -> TokenStream {
    let unit = dimen.get("unit").unwrap();
    let value = || dimen.get("value").unwrap().as_f32().unwrap();

    match unit {
        json::JsonValue::Short(ref unit) => match unit.as_ref() {
            "auto" => quote!(stretch::style::Dimension::Auto),
            "points" => {
                let value = value();
                quote!(stretch::style::Dimension::Points(#value))
            }
            "percent" => {
                let value = value();
                quote!(stretch::style::Dimension::Percent(#value))
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn generate_grid_area(grid_area: &json::object::Object) -> TokenStream {
    let kind = grid_area.get("kind").unwrap();

    match kind {
        json::JsonValue::Short(ref kind) => match kind.as_ref() {
            "auto" => quote!(stretch::style::GridArea::Auto),
            "explicit" => {
                let value = grid_area.get("value").unwrap();
                let row_start = value[0].as_f32().unwrap() as i32;
                let row_end = value[1].as_f32().unwrap() as i32;
                let column_start = value[2].as_f32().unwrap() as i32;
                let column_end = value[3].as_f32().unwrap() as i32;
                quote!(stretch::style::GridArea::Manual{row_start:#row_start, row_end:#row_end, column_start:#column_start, column_end:#column_end})
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn generate_track_bounds_list(track_bounds_list: &json::Array) -> TokenStream {
    let track_bounds_list: Vec<TokenStream> = track_bounds_list
        .iter()
        .map(|val| match val {
            json::JsonValue::Array(ref arr) => arr,
            _ => unreachable!(),
        })
        .map(|arr| {
            let (min_val, max_val) = match (&arr[0], &arr[1]) {
                (json::JsonValue::Object(ref v0), json::JsonValue::Object(ref v1)) => {
                    (generate_track_size(v0), generate_track_size(v1))
                }
                _ => unreachable!(),
            };
            quote!(stretch::style::TrackSizeBounds {min: #min_val, max: #max_val})
        })
        .collect();

    quote!(vec![#(#track_bounds_list),*])
}

fn generate_grid_line(grid_line: &json::object::Object) -> TokenStream {
    let kind = grid_line.get("kind").unwrap();

    match kind {
        json::JsonValue::Short(ref kind) => match kind.as_ref() {
            "nth" => {
                let value = grid_line.get("value").unwrap().as_f32().unwrap() as i32;
                quote!(stretch::style::GridLine::Nth(#value))
            }
            _ => unreachable!("Unsupported kind for grid-line: {}", kind),
        },
        _ => unreachable!(),
    }
}

fn generate_grid_tracks_definition(track_def: &json::Array) -> TokenStream {
    let track_sizes: Vec<TokenStream> = track_def
        .iter()
        .map(|val| match val {
            json::JsonValue::Object(ref value) => generate_track_size(value),
            _ => unreachable!(),
        })
        .collect();
    let mut track_defs: Vec<TokenStream> = (0..track_sizes.len())
        .step_by(2)
        .map(|i| {
            let min_val = &track_sizes[i];
            let max_val = &track_sizes[i + 1];
            quote!(stretch::style::TrackSizeBounds {min: #min_val, max: #max_val})
        })
        .collect();
    let fill = track_defs.pop();
    quote!(stretch::style::GridTracksTemplate {fill:#fill, defined:Some(vec![#(#track_defs),*])},)
}

fn generate_track_size(grid_size_value: &json::object::Object) -> TokenStream {
    let unit = grid_size_value.get("unit").unwrap();
    let value = || grid_size_value.get("value").unwrap().as_f32().unwrap();

    match unit {
        json::JsonValue::Short(ref unit) => match unit.as_ref() {
            "auto" => quote!(stretch::style::TrackSizeValues::Auto),
            "min-content" => quote!(stretch::style::TrackSizeValues::MinContent),
            "max-content" => quote!(stretch::style::TrackSizeValues::MaxContent),
            "points" => {
                let val = value();
                quote!(stretch::style::TrackSizeValues::Points(#val))
            }
            "percent" => {
                let val = value();
                quote!(stretch::style::TrackSizeValues::Percent(#val))
            }
            "flex" => {
                let val = value();
                quote!(stretch::style::TrackSizeValues::Flex(#val))
            }
            "auto-capped" => {
                let val = value();
                quote!(stretch::style::TrackSizeValues::ClampedAuto(#val))
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn generate_edges(dimen: &json::object::Object) -> TokenStream {
    dim_quoted!(dimen, start);
    dim_quoted!(dimen, end);
    dim_quoted!(dimen, top);
    dim_quoted!(dimen, bottom);

    quote!(stretch::geometry::Rect {
        #start #end #top #bottom
        ..Default::default()
    })
}
