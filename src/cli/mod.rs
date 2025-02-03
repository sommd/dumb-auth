use clap::{Parser, Subcommand};

pub use self::{passwd::passwd, run::run};
use self::{passwd::PasswdArgs, run::RunArgs};

mod common;
pub mod passwd;
pub mod run;

#[derive(Debug, PartialEq, Parser)]
#[command(about, author, version, args_conflicts_with_subcommands = true)]
pub struct Cli {
    #[command(flatten)]
    pub args: Option<RunArgs>,

    #[command(subcommand)]
    pub cmd: Option<Cmd>,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum Cmd {
    Passwd(PasswdArgs),
}

#[cfg(test)]
mod tests {
    use std::{env, iter, path::PathBuf};

    use clap::CommandFactory;

    use super::*;

    const PWARG: &str = "--password=hunter2";

    fn sut(args: &[&str]) -> Result<Cli, String> {
        Cli::try_parse_from(iter::once(&"dumb-auth").chain(args)).map_err(|e| e.to_string())
    }

    #[test]
    fn verify_args() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_threads() {
        // Disallows both --threads and --single-thread
        assert!(sut(&[PWARG, "--threads=2", "--single-thread"])
            .unwrap_err()
            .contains("cannot be used with '--single-thread"));
    }

    #[test]
    fn test_password() {
        // Requires a password arg
        assert!(sut(&[])
            .unwrap_err()
            .contains("required arguments were not provided"));

        // Accepts a single password arg
        assert_eq!(
            sut(&["--password=hunter2"])
                .unwrap()
                .args
                .unwrap()
                .password
                .as_deref(),
            Some("hunter2")
        );
        assert_eq!(
            sut(&["--password-file=password.txt"])
                .unwrap()
                .args
                .unwrap()
                .password_file,
            Some(PathBuf::from("password.txt"))
        );
        assert_eq!(
            sut(&["--password-hash=$1$n8iaq2sR$S2FSu61ixElrdPp/TUxtM0"])
                .unwrap()
                .args
                .unwrap()
                .password_hash
                .as_deref(),
            Some("$1$n8iaq2sR$S2FSu61ixElrdPp/TUxtM0")
        );
        assert_eq!(
            sut(&["--password-hash-file=password-hash.txt"])
                .unwrap()
                .args
                .unwrap()
                .password_hash_file,
            Some(PathBuf::from("password-hash.txt"))
        );

        // Disallows multiple password args
        assert!(sut(&["--password=hunter2", "--password-file=password.txt"])
            .unwrap_err()
            .contains("cannot be used with '--password"),);
    }

    #[test]
    fn test_allow_session() {
        // Defaults to true
        assert_eq!(sut(&[PWARG]).unwrap().args.unwrap().allow_session, true);
        assert_eq!(
            sut(&[PWARG, "--allow-session"])
                .unwrap()
                .args
                .unwrap()
                .allow_session,
            true
        );

        // Parses explicit value
        assert_eq!(
            sut(&[PWARG, "--allow-session=true"])
                .unwrap()
                .args
                .unwrap()
                .allow_session,
            true
        );
        assert_eq!(
            sut(&[PWARG, "--allow-session=false"])
                .unwrap()
                .args
                .unwrap()
                .allow_session,
            false
        );

        // Parses value from env
        env::set_var("DUMB_AUTH_ALLOW_SESSION", "false");
        assert_eq!(sut(&[PWARG]).unwrap().args.unwrap().allow_session, false);
        env::set_var("DUMB_AUTH_ALLOW_SESSION", "true");
        assert_eq!(sut(&[PWARG]).unwrap().args.unwrap().allow_session, true);
        env::remove_var("DUMB_AUTH_ALLOW_SESSION");
    }

    #[test]
    fn test_passwd() {
        // Does not require run args
        assert_eq!(
            sut(&["passwd"]).unwrap().cmd.unwrap(),
            Cmd::Passwd(PasswdArgs { output: "-".into() })
        );

        // Accepts outfile
        assert_eq!(
            sut(&["passwd", "outfile"]).unwrap().cmd.unwrap(),
            Cmd::Passwd(PasswdArgs {
                output: "outfile".into()
            })
        );

        // Disallows run args
        assert!(sut(&[PWARG, "passwd"])
            .unwrap_err()
            .contains("subcommand 'passwd' cannot be used with '--password"));
    }
}
