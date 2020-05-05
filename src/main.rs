
use aligner::{produce_calibration, locate_camera, Resolution};
use aligner::surfaces;
use clap::Clap;

/// Projection warp and alignment generator
#[derive(Clap)]
#[clap(version = "0.1.0", author = "Tom Riley")]
struct Opts {
    /// Surface type. Either "dome" or "wall".
    #[clap(short = "s", long = "surface-type", default_value = "dome", possible_values=&["wall", "dome"])]
    surface_type: String,
    /// Path to XML file containing camera intrinsic and extrinsic parameters
    #[clap(short = "x", long = "camera-xml-file", default_value = "noop.xml")]
    camera_calib_xml: String,
    /// URL to control and show images on projector.
    /// new line
    #[clap(short = "h", long = "control-url")]
    control_url: Option<String>,
    /// Pass a http(s) URL or file name to use for camera images (instead of a USB tethered camera)
    #[clap(short = "c", long = "camera")]
    camera: Option<String>,

    #[clap(subcommand)]
    subcmd: SubCommand
}

#[derive(Clap)]
enum SubCommand {
    /// Calculate frustum and warp for a single projector
    #[clap(name = "generate-warp")]
    GenerateWarpCommand(GenerateWarpCommand),
    /// Locate the physical camera relative to a single aruco marker
    #[clap(name = "locate-camera")]
    LocateCameraCommand(LocateCameraCommand),
}

/// Start process of aligning and warping for a static virtual camera. Results in
/// a camera "look at" coordinate in scene space and a UV matrix for warping the resulting
/// frame buffer to the projection surface geometry.
#[derive(Clap)]
struct GenerateWarpCommand {
    /// JSON file containing camera location information (output of locate command).
    /// Ignored if surface type is "dome" (physical camera is assumed to be at the center of the dome).
    #[clap(short = "j", long = "camera-location-json")]
    camera_location_json: Option<String>,
    
    /// Chessboard pattern size
    #[clap(short = "p", long = "pattern-size
    ", default_value = "25x16")]
    pattern_size: String,

    /// Projector output resolution
    #[clap(short = "z", long = "resolution", default_value = "1024x768")]
    resolution: String,

    /// HTTP POST generated warp and eye point configuration to a URL.
    /// If not specified the configuration will be printed to stdout.
    #[clap(long = "post-to-url")]
    post_json_to: Option<String>,

    /// Eye position in scene space. Necessary to generate the UV warp and the vertical FOV for the
    /// scene viewing frustum.
    #[clap(long = "eye", default_value = "0,0,0")]
    eye_position: String,

    /// Radius of dome [required if --surface-type=dome]
    #[clap(long = "radius", default_value = "5")]
    radius: f32,
}

/// Locate the camera in physical space. Place an aruco marker at 0,0,0 facing Z axis.
#[derive(Clap)]
struct LocateCameraCommand {
    /// Aruco marker size in meters
    #[clap(short = "m", long = "marker-size")]
    marker_size: Option<f32>,
}

fn main() {
    simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default()).unwrap();

    let opts: Opts = Opts::parse();
    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    match opts.subcmd {
        SubCommand::GenerateWarpCommand(cmd) => {
            produce_calibration(
                surface_type(&opts.surface_type, &cmd),
                &opts.camera_calib_xml,
                opts.control_url.as_deref(),
                opts.camera.as_deref(),
                cmd.camera_location_json.as_deref(),
                parse_vec3(&cmd.eye_position).expect("invalid eye position"),
                Resolution::parse(&cmd.pattern_size).expect("invalid pattern size"),
                Resolution::parse(&cmd.resolution).expect("invalid projector resolution"),
                cmd.post_json_to.as_deref()
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
        "dome" => surfaces::SurfaceType::HemisphericalDome {radius: opts.radius},
        "wall" => surfaces::SurfaceType::Wall,
        _ => panic!("Unknown surface type. Please specify 'dome' or 'wall'")
    }
}

fn parse_vec3(input: &str) -> Result<glm::Vec3, &'static str> {
    let mut floats = [0_f32; 3];
    for (i, word) in input.split(|c| c == ',').enumerate() {
        if i >= 3 {
            return Err("invalid 3D vector, too many components");
        }
        floats[i] = word.parse().unwrap();
    }
    Ok(glm::vec3(floats[0], floats[1], floats[2]))
}