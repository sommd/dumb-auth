use std::{fs::File, io::Write, path::PathBuf};

use clap::Args;
use zeroize::Zeroize;

use super::common::fatal;

/// Hash a password, to be used with --password-hash[-file].
///
/// Password hashing follows the recommended OWASP password guidelines as of January 2025.
#[derive(Args, Debug, PartialEq)]
pub struct PasswdArgs {
    /// File to output password to instead of stdout, or "-" for stdout. File will be overwritten.
    #[arg(default_value = "-", hide_default_value = true)]
    pub output: PathBuf,
}

pub fn passwd(args: PasswdArgs) {
    let mut plain = rpassword::prompt_password("Enter password: ")
        .unwrap_or_else(|e| fatal("reading password", e));

    let hash = dumb_auth::hash_password(&plain).unwrap_or_else(|e| fatal("hashing password", e));

    plain.zeroize();

    if args.output.to_str() == Some("-") {
        println!("{}", &hash);
    } else {
        File::create(args.output)
            .and_then(|mut f| writeln!(f, "{}", hash))
            .unwrap_or_else(|e| fatal("writing to output file", e));
    }
}
