extern crate pretty_env_logger;
use aligner::{warp_hemispherical_dome, warp_wall, WarpResolution};
use clap::Clap;

/// Projection warp and alignment generator
#[derive(Clap)]
#[clap(version = "0.1.0", author = "Tom Riley")]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short = "s", long = "surface", default_value = "dome")]
    surface: String,
    /// Path to XML file containing camera intrinsic and extrinsic parameters
    #[clap(short = "xml", long = "camera-xml-file", default_value = "sigma19mm.xml")]
    camera_calib_xml: String,
    /// URL to control and show images on projector
    #[clap(long = "control-url")]
    control_url: Option<String>,
    /// URL to fetch camera photo from
    #[clap(long = "photo-url")]
    photo_url: Option<String>,
    /// Chessboard pattern size
    #[clap(long = "pattern-size", default_value = "25x16")]
    pattern_size: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short = "v", long = "verbose", parse(from_occurrences))]
    verbosity: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// A help message for the Test subcommand
    #[clap(name = "warp", version = "1.3", author = "Someone Else")]
    GenerateWarpCommand(GenerateWarpCommand),
}



/// Start process of algining and warping for a static virtual camera. Results in
/// a camera "look at" coordinate in scene space and a UV matrix for warping the resulting
/// frame buffer to the projection surface geometry.
#[derive(Clap)]
struct GenerateWarpCommand {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short = "r", long = "radius")]
    radius: Option<f32>,
}

fn main() {
    pretty_env_logger::init();
    let opts: Opts = Opts::parse();
    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    match opts.subcmd {
        SubCommand::GenerateWarpCommand(t) => {
            match &opts.surface[..] {
                "dome" => warp_hemispherical_dome(
                    t.radius.expect("Please proide a dome radius in meters with -r"), // FIXME maybe dome radius is not required
                    &opts.camera_calib_xml,
                    opts.control_url.as_deref(),
                    opts.photo_url.as_deref(),
                    WarpResolution::parse(&opts.pattern_size).unwrap(),
                    opts.verbosity
                ),
                "wall" => warp_wall(
                    &opts.camera_calib_xml,
                    opts.control_url.as_deref(),
                    opts.photo_url.as_deref(),
                    WarpResolution::parse(&opts.pattern_size).unwrap(),
                    opts.verbosity
                ),
                _ => panic!("Unknown surface type. Please specify 'dome' or 'wall'")
            };
        }
    }
}

