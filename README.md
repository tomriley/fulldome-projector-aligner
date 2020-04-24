

    aligner 0.1.0
    Tom Riley
    Projection warp and alignment generator

    USAGE:
        aligner [FLAGS] [OPTIONS] <SUBCOMMAND>

    FLAGS:
        -h, --help       Prints help information
        -v, --verbose    A level of verbosity, and can be used multiple times
        -V, --version    Prints version information

    OPTIONS:
        -x, --camera-xml-file <camera-calib-xml>    Path to XML file containing camera intrinsic and extrinsic parameters
                                                    [default: sigma19mm.xml]
            --control-url <control-url>             URL to control and show images on projector
            --pattern-size <pattern-size>           Chessboard pattern size [default: 25x16]
            --photo-url <photo-url>                 URL to fetch camera photo from
        -s, --surface <surface>                     Sets a custom config file. Could have been an Option<T> with no default
                                                    too [default: dome]

    SUBCOMMANDS:
        help    Prints this message or the help of the given subcommand(s)
        warp    A help message for the Test subcommand