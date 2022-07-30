use std::io::Write;

pub fn warn(s: &str) {
    #[cfg(feature = "color")]
    warn_color(s);
    #[cfg(not(feature = "color"))]
    warn_no_color(s);
}

fn warn_no_color(s: &str) {
    let mut stderr = std::io::stderr();
    writeln!(&mut stderr, "Warning: {}", s).unwrap();
}

#[cfg(feature = "color")]
fn warn_color(s: &str) {
    use termcolor::*;

    if atty::is(atty::Stream::Stderr) {
        let mut stderr = StandardStream::stderr(ColorChoice::Auto);
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true)).unwrap();
        write!(&mut stderr, "Warning: ").unwrap();
        stderr.set_color(ColorSpec::new().set_fg(None).set_bold(false)).unwrap();
        writeln!(&mut stderr, "{}", s).unwrap();
    }
    else {
        warn_no_color(s);
    }
}

pub fn error(s: &str) {
    #[cfg(feature = "color")]
    error_color(s);
    #[cfg(not(feature = "color"))]
    error_no_color(s);
}

fn error_no_color(s: &str) {
    let mut stderr = std::io::stderr();
    writeln!(&mut stderr, "Error: {}", s).unwrap();
}

#[cfg(feature = "color")]
fn error_color(s: &str) {
    use termcolor::*;

    if atty::is(atty::Stream::Stderr) {
        let mut stderr = StandardStream::stderr(ColorChoice::Auto);
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).unwrap();
        write!(&mut stderr, "Error: ").unwrap();
        stderr.set_color(ColorSpec::new().set_fg(None).set_bold(false)).unwrap();
        writeln!(&mut stderr, "{}", s).unwrap();
    }
    else {
        error_no_color(s);
    }
}
