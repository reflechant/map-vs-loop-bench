//! Inline a PNG via Kitty graphics (Ghostty, Kitty, WezTerm) or iTerm OSC 1337.
//!
//! Returns whether the image was actually sent to a terminal. Callers should
//! print a text summary when this is `false` (piped output, dumb tty, etc.).

use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

enum Protocol {
    Kitty,
    Iterm,
}

pub fn try_show_png(path: &Path) -> bool {
    if !io::stdout().is_terminal() {
        return false;
    }
    let Some(proto) = detect() else {
        return false;
    };
    transmit(path, proto).is_ok()
}

fn detect() -> Option<Protocol> {
    let term = env::var("TERM").unwrap_or_default().to_ascii_lowercase();
    let program = env::var("TERM_PROGRAM")
        .unwrap_or_default()
        .to_ascii_lowercase();

    if env::var_os("KITTY_WINDOW_ID").is_some()
        || env::var_os("GHOSTTY_RESOURCES_DIR").is_some()
        || term.contains("kitty")
        || term.contains("ghostty")
        || program.contains("ghostty")
        || program.contains("wezterm")
        || program.contains("kitty")
    {
        return Some(Protocol::Kitty);
    }

    if env::var_os("ITERM_SESSION_ID").is_some()
        || program.contains("iterm")
        || program.contains("warp")
    {
        return Some(Protocol::Iterm);
    }

    None
}

fn transmit(path: &Path, proto: Protocol) -> io::Result<()> {
    let bytes = std::fs::read(path)?;
    let b64 = BASE64.encode(&bytes);
    let mut out = io::stdout().lock();
    match proto {
        Protocol::Kitty => kitty_png(&mut out, &b64)?,
        Protocol::Iterm => iterm_png(&mut out, bytes.len(), &b64)?,
    }
    out.flush()
}

fn kitty_png(out: &mut impl Write, b64: &str) -> io::Result<()> {
    // q=2: suppress terminal responses so we do not print "OK" into the bench log.
    let chunks: Vec<&[u8]> = b64.as_bytes().chunks(4096).collect();
    let last = chunks.len().saturating_sub(1);
    for (i, chunk) in chunks.iter().enumerate() {
        let more = u8::from(i != last);
        if i == 0 {
            write!(out, "\x1b_Ga=T,f=100,q=2,m={more};")?;
        } else {
            write!(out, "\x1b_Gm={more};")?;
        }
        out.write_all(chunk)?;
        write!(out, "\x1b\\")?;
    }
    writeln!(out)
}

fn iterm_png(out: &mut impl Write, size: usize, b64: &str) -> io::Result<()> {
    writeln!(
        out,
        "\x1b]1337;File=inline=1;size={size};width=auto;preserveAspectRatio=1:{b64}\x07"
    )
}
