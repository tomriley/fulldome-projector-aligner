extern crate pretty_env_logger;
use aligner::{produce_calibration, locate_camera, WarpResolution};
use aligner::surfaces;
use clap::Clap;

/// Projection warp and alignment generator
#[derive(Clap)]
#[clap(version = "0.1.0", author = "Tom Riley")]
struct Opts {
    /// Surface type. Either "dome" or "wall".
    #[clap(short = "s", long = "surface-type", default_value = "wall", possible_values=&["wall", "dome"])]
    surface: String,
    /// Path to XML file containing camera intrinsic and extrinsic parameters
    #[clap(short = "x", long = "camera-xml-file", default_value = "noop.xml")]
    camera_calib_xml: String,
    /// URL to control and show images on projector
    #[clap(short = "h", long = "control-url")]
    control_url: Option<String>,
    /// Pass a http(s) URL or file name to use for camera images (instead of a USB tethered camera)
    #[clap(short = "c", long = "camera")]
    camera: Option<String>,
    
    
    /// A level of verbosity, and can be used multiple times
    #[clap(short = "v", long = "verbose", parse(from_occurrences))]
    verbosity: i32,
    #[clap(subcommand)]
    subcmd: SubCommand
}

#[derive(Clap)]
enum SubCommand {
    /// A help message for the Test subcommand
    #[clap(name = "generate-warp")]
    GenerateWarpCommand(GenerateWarpCommand),
    #[clap(name = "locate-camera")]
    LocateCameraCommand(LocateCameraCommand),
}

/// Start process of algining and warping for a static virtual camera. Results in
/// a camera "look at" coordinate in scene space and a UV matrix for warping the resulting
/// frame buffer to the projection surface geometry.
#[derive(Clap)]
struct GenerateWarpCommand {
    /// JSON file containing camera location information (output of locate command).
    /// Ignored if surface type is "dome"
    #[clap(short = "j", long = "camera-location-json")]
    camera_location_json: Option<String>,
    /// Chessboard pattern size
    #[clap(short = "p", long = "pattern-size", default_value = "25x16")]
    pattern_size: String,
    /// Projector output resolution
    #[clap(short = "z", long = "resolution", default_value = "1024x768")]
    resolution: String,
    /// HTTP POST generated warp and eye point configuration to a URL.
    /// If not specified the confuration will be printed to stdout.
    #[clap(long = "post-to")]
    post_json_to: String,
    /// Radius of dome (if using surface type dome)
    #[clap(short = "r", long = "radius")]
    radius: Option<f32>,
}

/// Locate the camera in physical space. Place an aruco marker at 0,0,0 facing along Z axis.
#[derive(Clap)]
struct LocateCameraCommand {
    /// Aruco marker size in meters
    #[clap(short = "m", long = "marker-size")]
    marker_size: Option<f32>,
}

fn main() {
    pretty_env_logger::init();
    let opts: Opts = Opts::parse();
    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    match opts.subcmd {
        SubCommand::GenerateWarpCommand(cmd) => {
            produce_calibration(
                surface_type(&opts.surface, &cmd),
                &opts.camera_calib_xml,
                opts.control_url.as_deref(),
                opts.camera.as_deref(),
                cmd.camera_location_json.as_deref(),
                WarpResolution::parse(&cmd.pattern_size).expect("bad pattern size"),
                opts.verbosity
            );
        }
        SubCommand::LocateCameraCommand(cmd) => {
            locate_camera(
                &opts.camera_calib_xml,
                opts.camera.as_deref(),
                cmd.marker_size.expect("missing maker size option")
            );
        }
    }
}

fn surface_type(surface: &str, opts: &GenerateWarpCommand) -> surfaces::SurfaceType {
    match surface {
        "dome" => surfaces::SurfaceType::HemisphericalDome {radius: opts.radius.expect("Please proide a dome radius in meters with -r")},
        "wall" => surfaces::SurfaceType::Wall,
        _ => panic!("Unknown surface type. Please specify 'dome' or 'wall'")
    }
}