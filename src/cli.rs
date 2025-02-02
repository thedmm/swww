/// Note: this file only has basic declarations and some definitions in order to be possible to
/// import it in the build script, to automate shell completion
use clap::Parser;
use std::path::PathBuf;

fn from_hex(hex: &str) -> Result<[u8; 3], String> {
    let chars = hex
        .chars()
        .filter(|&c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase() as u8);

    if chars.clone().count() != 6 {
        return Err(format!(
            "expected 6 characters, found {}",
            chars.clone().count()
        ));
    }

    let mut color = [0, 0, 0];

    for (i, c) in chars.enumerate() {
        match c {
            b'A'..=b'F' => color[i / 2] += c - b'A' + 10,
            b'0'..=b'9' => color[i / 2] += c - b'0',
            _ => {
                return Err(format!(
                    "expected [0-9], [a-f], or [A-F], found '{}'",
                    char::from(c)
                ))
            }
        }
        if i % 2 == 0 {
            color[i / 2] *= 16;
        }
    }
    Ok(color)
}

#[derive(Clone)]
pub enum Filter {
    Nearest,
    Bilinear,
    CatmullRom,
    Mitchell,
    Lanczos3,
}

impl std::str::FromStr for Filter {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Nearest" => Ok(Self::Nearest),
            "Bilinear" => Ok(Self::Bilinear),
            "CatmullRom" => Ok(Self::CatmullRom),
            "Mitchell" => Ok(Self::Mitchell),
            "Lanczos3" => Ok(Self::Lanczos3),
            _ => Err("unrecognized filter. Valid filters are:\
                     Nearest | Bilinear | CatmullRom | Mitchell | Lanczos3\
                     see swww img --help for more details"),
        }
    }
}

#[derive(Clone)]
pub enum TransitionType {
    Simple,
    Left,
    Right,
    Top,
    Bottom,
    Center,
    Outer,
    Any,
    Random,
    Wipe,
    Wave,
    Grow,
}

impl std::str::FromStr for TransitionType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "simple" => Ok(Self::Simple),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "top" => Ok(Self::Top),
            "bottom" => Ok(Self::Bottom),
            "wipe" => Ok(Self::Wipe),
            "grow" => Ok(Self::Grow),
            "center" => Ok(Self::Center),
            "outer" => Ok(Self::Outer),
            "any" => Ok(Self::Any),
            "wave" => Ok(Self::Wave),
            "random" => Ok(Self::Random),
            _ => Err("unrecognized transition type.\nValid transitions are:\n\
                     \tsimple | left | right | top | bottom | wipe | grow | center | outer | random | wave\n\
                     see swww img --help for more details"),
        }
    }
}

#[derive(Parser)]
#[command(version, name = "swww")]
///A Solution to your Wayland Wallpaper Woes
///
///Change what your monitors display as a background by controlling the swww daemon at runtime.
///Supports animated gifs and putting different stuff in different monitors. I also did my best to
///make it as resource efficient as possible.
pub enum Swww {
    ///Fills the specified outputs with the given color.
    ///
    ///Defaults to filling all outputs with black.
    Clear(Clear),

    /// Send an image (or animated gif) for the daemon to display.
    ///
    /// Use `-` to read from stdin
    Img(Img),

    /// Initialize the daemon.
    ///
    /// Exits if there is already a daemon running. We check thay by seeing if
    /// $XDG_RUNTIME_DIR/swww.socket exists.
    Init {
        ///Don't fork the daemon. This will keep it running in the current terminal.
        ///
        ///The only advantage of this would be seeing the logging real time. Note that for release
        ///builds we only log info, warnings and errors, so you won't be seeing much (ideally).
        #[clap(long)]
        no_daemon: bool,
    },

    ///Kills the daemon
    Kill,

    ///Asks the daemon to print output information (names and dimensions).
    ///
    ///You may use this to find out valid values for the <swww-img --outputs> option. If you want
    ///more detailed information about your outputs, I would recommed trying wlr-randr.
    Query,
}

#[derive(Parser)]
pub struct Clear {
    /// Color to fill the screen with.
    ///
    /// Must be given in rrggbb format (note there is no prepended '#').
    #[arg(value_parser = from_hex, default_value = "000000")]
    pub color: [u8; 3],

    /// Comma separated list of outputs to display the image at.
    ///
    /// If it isn't set, the image is displayed on all outputs.
    #[clap(short, long, default_value = "")]
    pub outputs: String,
}

#[derive(Parser)]
pub struct Img {
    /// Path to the image to display
    pub path: PathBuf,

    /// Comma separated list of outputs to display the image at.
    ///
    /// If it isn't set, the image is displayed on all outputs.
    #[arg(short, long, default_value = "")]
    pub outputs: String,

    #[arg(long)]
    pub no_resize: bool,

    #[arg(value_parser = from_hex, long, default_value = "000000")]
    pub fill_color: [u8; 3],

    ///Filter to use when scaling images (run swww img --help to see options).
    ///
    ///Note that image scaling can sometimes significantly increase RAM usage. If you want to use
    ///as little RAM as possible, I recommend scaling the images before sending them to swww
    ///
    ///Available options are:
    ///
    ///Nearest | Bilinear | CatmullRom | Mitchell | Lanczos3
    ///
    ///These are offered by the image crate (https://crates.io/crates/image). 'Nearest' is
    ///what I recommend for pixel art stuff, and ONLY for pixel art stuff. It is also the
    ///fastest filter.
    ///
    ///For non pixel art stuff, I would usually recommend one of the last three, though some
    ///experimentation will be necessary to see which one you like best. Also note they are
    ///all slower than Nearest. For some examples, see
    ///https://docs.rs/image/latest/image/imageops/enum.FilterType.html.
    #[arg(short, long, default_value = "Lanczos3")]
    pub filter: Filter,

    ///Sets the type of transition. Default is 'simple', that fades into the new image
    ///
    ///Possible transitions are:
    ///
    ///simple | left | right | top | bottom | wipe | grow | center | any | outer | random
    ///
    ///The 'left', 'right', 'top' and 'bottom' options make the transition happen from that
    ///position to its oposite in the screen.
    ///
    ///'wipe' is simillar to 'left' but allows you to specify the angle for transition (with the --transition-angle flag).
    ///
    ///'grow' causes a growing circle to transition across the screen and allows changing the circle's center
    /// position (with --transition-pos flag).
    ///
    ///'center' an alias to 'grow' with position set to center of screen.
    ///
    ///'any' an alias to 'grow' with position set to a random point on screen.
    ///
    ///'outer' same as grow but the circle shrinks instead of growing.
    ///
    ///Finally, 'random' will select a transition effect at random
    #[arg(short, long, env = "SWWW_TRANSITION", default_value = "simple")]
    pub transition_type: TransitionType,

    ///How fast the transition approaches the new image.
    ///
    ///The transition logic works by adding or subtracting from the current rgb values until the
    ///old image transforms in the new one. This controls by how much we add or subtract.
    ///
    ///Larger values will make the transition faster, but more abrupt. A value of 255 will always
    ///switch to the new image immediately.
    ///
    ///Broadly speaking, this is mostly only visible during the 'simple' transition. The other
    ///transitions tend to change more with the 'transition-step' and 'transition-speed' options
    #[arg(long, env = "SWWW_TRANSITION_STEP", default_value = "90")]
    pub transition_step: u8,

    ///How long the transition takes to complete in seconds.
    ///
    ///Note that this doesnt work with the 'simple' transition
    #[arg(long, env = "SWWW_TRANSITION_DURATION", default_value = "3")]
    pub transition_duration: f32,

    ///Frame rate for the transition effect.
    ///
    ///Note there is no point in setting this to a value smaller than what your monitor supports.
    ///
    ///Also note this is **different** from the transition-step. That one controls by how much we
    ///approach the new image every frame.
    #[arg(long, env = "SWWW_TRANSITION_FPS", default_value = "30")]
    pub transition_fps: u8,

    ///This is only used for the 'wipe' transition. It controls the angle of the wipe (default is '0').
    ///
    ///Note that the angle is in degrees, where '0' is right to left and '90' is top to bottom, and '270' bottom to top
    #[arg(long, env = "SWWW_TRANSITION_ANGLE", default_value = "45")]
    pub transition_angle: f64,

    ///This is only used for the 'grow','outer' transitions. It controls the center of circle (default is 'center').
    ///
    ///position values are in the format 'x,y' where x and y are in the range [0,1.0] and represent the percentage of screen dimension
    ///
    ///the value can also be an alias which will set the position accordingly):
    /// 'center' | 'top' | 'left' | 'right' | 'bottom' | 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right'
    #[arg(long, env = "SWWW_TRANSITION_POS", default_value = "center", value_parser = parse_coords)]
    pub transition_pos: (f32, f32),

    ///bezier curve to use for the transition
    ///https://cubic-bezier.com is a good website to get these values from
    ///
    ///eg: 0.0,0.0,1.0,1.0 for linear animation
    #[arg(long, env = "SWWW_TRANSITION_BEZIER", default_value = ".54,0,.34,.99", value_parser = parse_bezier)]
    pub transition_bezier: (f32, f32, f32, f32),

    ///currently only used for 'wave' transition to control the width and height of each wave
    #[arg(long, env = "SWWW_TRANSITION_WAVE", default_value = "20,20", value_parser = parse_wave)]
    pub transition_wave: (f32, f32),
}

fn parse_wave(raw: &str) -> Result<(f32, f32), String> {
    let mut iter = raw.split(',');
    let mut parse = || {
        iter.next()
            .ok_or_else(|| "Not enough values".to_string())
            .and_then(|s| s.parse::<f32>().map_err(|e| e.to_string()))
    };

    let parsed = (parse()?, parse()?);
    Ok(parsed)
}

fn parse_bezier(raw: &str) -> Result<(f32, f32, f32, f32), String> {
    let mut iter = raw.split(',');
    let mut parse = || {
        iter.next()
            .ok_or_else(|| "Not enough values".to_string())
            .and_then(|s| s.parse::<f32>().map_err(|e| e.to_string()))
    };

    let parsed = (parse()?, parse()?, parse()?, parse()?);
    if parsed == (0.0, 0.0, 0.0, 0.0) {
        return Err("Invalid bezier curve: 0,0,0,0 (try using 0,0,1,1 instead)".to_string());
    }
    Ok(parsed)
}

// parses percentages and numbers in format of "<coord1>,<coord2>"
fn parse_coords(raw: &str) -> Result<(f32, f32), String> {
    let coords = raw.split(',').map(|s| s.trim()).collect::<Vec<&str>>();
    let x: &str;
    let y: &str;
    if coords.len() != 2 {
        match coords[0] {
            "center" => {
                x = "0.5";
                y = "0.5";
            }
            "top" => {
                x = "0.5";
                y = "1.0";
            }
            "bottom" => {
                x = "0.5";
                y = "0";
            }
            "left" => {
                x = "0";
                y = "0.5";
            }
            "right" => {
                x = "1.0";
                y = "0.5";
            }
            "top-left" => {
                x = "0";
                y = "1.0";
            }
            "top-right" => {
                x = "1.0";
                y = "1.0";
            }
            "bottom-left" => {
                x = "0";
                y = "0";
            }
            "bottom-right" => {
                x = "1.0%";
                y = "0";
            }
            _ => return Err(format!("Invalid position keyword: {raw}")),
        }
    } else {
        x = coords[0];
        y = coords[1];
    }

    //parse x coord
    let parsed_x = match x.parse::<f32>() {
        Ok(x) => {
            if !(0.0..=1.0).contains(&x) {
                return Err(format!("x coord not in range [0,1.0]: {x}"));
            }
            x
        }
        Err(_) => return Err(format!("Invalid x coord: {x}")),
    };

    //parse y coord
    let parsed_y = match y.parse::<f32>() {
        Ok(y) => {
            if !(0.0..=1.0).contains(&y) {
                return Err(format!("y coord not in range [0,1.0]: {y}"));
            }
            y
        }
        Err(_) => return Err(format!("Invalid y coord: {y}")),
    };

    Ok((parsed_x, parsed_y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_reject_wrong_colors() {
        assert!(
            from_hex("0012231").is_err(),
            "function is accepting strings with more than 6 chars"
        );
        assert!(
            from_hex("00122").is_err(),
            "function is accepting strings with less than 6 chars"
        );
        assert!(
            from_hex("00r223").is_err(),
            "function is accepting strings with chars that aren't hex"
        );
    }

    #[test]
    fn should_convert_colors_from_hex() {
        let color = from_hex("101010").unwrap();
        assert_eq!(color, [16, 16, 16]);

        let color = from_hex("ffffff").unwrap();
        assert_eq!(color, [255, 255, 255]);

        let color = from_hex("000000").unwrap();
        assert_eq!(color, [0, 0, 0]);
    }
}
