use std::fmt::Debug;
use std::net::{IpAddr, UdpSocket};
use std::time::Duration;

use regex::Regex;

/// Get the local IP address
#[cfg(not(target_arch = "wasm32"))]
pub fn get_local_ip() -> IpAddr {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Socket not found.");

    socket.connect("8.8.8.8:80").expect("Failed to connect to socket.");
    socket.local_addr().ok().map(|addr| addr.ip()).unwrap()
}

#[cfg(target_arch = "wasm32")]
pub fn get_local_ip() -> IpAddr {
    // WebAssembly in browsers cannot access local network interfaces
    "127.0.0.1".parse().unwrap()
}

/// Scale a Duration by a factor
pub fn scale_duration(duration: Duration, scale: f32) -> Duration {
    let sec = (duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9) * scale;
    Duration::new(sec.trunc() as u64, (sec.fract() * 1e9) as u32)
}

/// Add dots to thousands
pub fn format_thousands(n: usize) -> String {
    let s = n.to_string();
    let chars: Vec<char> = s.chars().rev().collect();
    let mut result = Vec::new();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push('.');
        }
        result.push(*c);
    }

    result.iter().rev().collect()
}

/// Helper function to extract only the variant name (removes tuple/struct fields)
fn extract_variant_name(text: String) -> String {
    text.split_once('(')
        .or_else(|| text.split_once('{'))
        .map(|(variant, _)| variant)
        .unwrap_or(&text)
        .trim_matches(&['"', ' '][..])
        .to_string()
}

/// Trait to get the text of an enum variant
pub trait NameFromEnum {
    fn to_name(&self) -> String;
    fn to_lowername(&self) -> String;
    fn to_title(&self) -> String;
}

impl<T: Debug> NameFromEnum for T {
    fn to_name(&self) -> String {
        let re = Regex::new(r"([a-z])([A-Z])").unwrap();

        let text = extract_variant_name(format!("{:?}", self));
        re.replace_all(&text, "$1 $2").to_string()
    }

    fn to_lowername(&self) -> String {
        self.to_name().to_lowercase()
    }

    fn to_title(&self) -> String {
        let mut name = self.to_lowername();

        // Capitalize only the first letter
        name.replace_range(0..1, &name[0..1].to_uppercase());

        name
    }
}

/// Trait to safely divide by zero
pub trait SafeDiv: Sized + PartialEq + Copy {
    fn safe_div(self, b: Self) -> Self;
}

impl SafeDiv for f32 {
    #[inline]
    fn safe_div(self, b: Self) -> Self {
        if b == 0.0 {
            0.0
        } else {
            self / b
        }
    }
}

/// Trait to convert a large number to a nice formatted string
pub trait FmtNumb {
    fn fmt(self) -> String;
}

impl FmtNumb for usize {
    fn fmt(self) -> String {
        match self {
            n if n > 1_000_000 => format!("{:.2}M", self as f32 / 1_000_000.),
            n if n > 100_000 => format!("{:.0}k", self as f32 / 100_000.),
            n if n >= 1_000 => format!("{:.1}k", self as f32 / 1_000.),
            _ => self.to_string(),
        }
    }
}
