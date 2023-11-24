use firedbg_lib::fire;
use visioncortex::{
    color_clusters::{KeyingAction, Runner, RunnerConfig, HIERARCHICAL_MAX},
    Color, ColorImage, PointI32, Shape,
};

struct Object {
    sym: Symbol,
    pos: PointI32,
    color: Color,
}

enum Symbol {
    Circle,
    Triangle,
    Diamond,
}

// It would be uninteresting to input a black/white image. So here is a ColorImage!
pub fn decode_image(image: ColorImage) -> Vec<char> {
    let width = image.width;

    // A clustering algorithm for processing color images
    // adapted from https://github.com/visioncortex/vtracer/blob/master/cmdapp/src/converter.rs
    let runner = Runner::new(
        RunnerConfig {
            diagonal: false,
            hierarchical: HIERARCHICAL_MAX,
            batch_size: 25600,
            good_min_area: 25,
            good_max_area: (image.width * image.height),
            is_same_color_a: 2,
            is_same_color_b: 1,
            deepen_diff: 16,
            hollow_neighbours: 1,
            key_color: Color::default(),
            keying_action: KeyingAction::Discard,
        },
        image,
    );

    let mut clusters = runner.run();
    let view = clusters.view();
    let mut objects = Vec::new();

    // the root cluster is the background
    let background = *view.clusters_output.last().unwrap();
    let background = view.get_cluster(background).residue_color().to_color_i32();

    // iterate the clusters and filter out non-symbols like the english words
    for &cluster_index in view.clusters_output.iter() {
        let cluster = view.get_cluster(cluster_index);
        let rect = &cluster.rect;
        // a few assumptions about our symbols
        if (rect.width() < 100 && rect.height() < 100) // small enough
        && rect.aspect_ratio_doubled() == 2 // more or less square
        // clusters with same color as background are holes
            && background
                .diff(&cluster.residue_color().to_color_i32())
                .absolute()
                > 10
        {
            let shape = cluster.to_shape(&view);
            objects.push(Object {
                sym: if let Some(sym) = determine_symbol(&shape) {
                    sym
                } else {
                    continue;
                },
                pos: rect.center(),
                // unused right now, but could be used to encode data
                color: cluster.residue_color(),
            });
        }
    }

    // align these objects on a grid of 50px spaced, and sort from top-to-bottom (primary), left-to-right (secondary)
    // in more complex case we can use the disjoint_set algorithm to group them into rows
    let cols = width as i32 / 50;
    objects.sort_by_key(|o| (o.pos.y / 50) * cols + o.pos.x / 50);
    fire::dbg!(&objects);

    fire::dbg!(
        "return",
        objects
            .iter()
            .map(|o| match o.sym {
                Symbol::Diamond => '🔷',
                Symbol::Triangle => '🔺',
                Symbol::Circle => '🟡',
            })
            .collect()
    )
}

fn determine_symbol(shape: &Shape) -> Option<Symbol> {
    if shape.is_isosceles_triangle() {
        // triangle should come first, because a triangle is also a degenerated quadrilateral
        Some(Symbol::Triangle)
    } else if shape.is_quadrilateral() {
        Some(Symbol::Diamond)
    } else if shape.is_circle() {
        Some(Symbol::Circle)
    } else {
        // not a symbol
        None
    }
}
