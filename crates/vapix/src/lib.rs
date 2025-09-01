use std::{
    io::Read,
    process::{Command, Stdio},
    str::FromStr,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{bail, Context};
use log::{debug, warn};
use rs4a_vapix::{Client, ClientBuilder, Scheme};
use url::Host;

static COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug)]
struct Credentials {
    username: String,
    // TODO: Consider using something like secrecy
    password: String,
}

impl Credentials {
    fn try_get(name: &str) -> anyhow::Result<Self> {
        let mut child = Command::new("/usr/bin/gdbus")
            .arg("call")
            .arg("--system")
            .args(["--dest", "com.axis.HTTPConf1"])
            .args(["--object-path", "/com/axis/HTTPConf1/VAPIXServiceAccounts1"])
            .args([
                "--method",
                "com.axis.HTTPConf1.VAPIXServiceAccounts1.GetCredentials",
            ])
            .arg(name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdout = child
            .stdout
            .take()
            .expect("stdout is piped and has not been taken");
        let mut stderr = child
            .stderr
            .take()
            .expect("stderr is piped and has not been taken");

        let status = child.wait()?;

        let mut stderr_text = String::new();
        stderr.read_to_string(&mut stderr_text)?;
        if !stderr_text.is_empty() {
            warn!("Discarding stderr {stderr_text:?}");
        }

        let mut stdout_text = String::new();
        stdout.read_to_string(&mut stdout_text)?;
        if !status.success() {
            debug!("Discarding stdout {stdout_text:?}");
            bail!("Command exited with status {status}")
        }

        Self::from_str(&stdout_text)
    }
}

impl FromStr for Credentials {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (username, password) = s
            .trim()
            .strip_prefix("('")
            .context("Expected response to start with ('")?
            .strip_suffix("',)")
            .context("Expected response to end with ',)")?
            .split_once(':')
            .context("Expected response to contain :")?;
        Ok(Self {
            username: username.to_string(),
            password: password.to_string(),
        })
    }
}

/// Construct a new [`Client`] for use with VAPIX.
///
/// # Warning
///
/// The resulting client may use HTTP and may accept invalid server certificates.
pub async fn new_client() -> anyhow::Result<Client> {
    if cfg!(feature = "host") {
        debug!("Building client from env");
        ClientBuilder::from_dut()?
            .context("No client configuration found")?
            .with_inner(|b| b.danger_accept_invalid_certs(true))
            .build_with_automatic_scheme()
            .await
    } else {
        let name = format!("n{}", COUNT.fetch_add(1, Ordering::Relaxed));
        debug!("Getting credentials from dbus for name {name}");
        let Credentials { username, password } = Credentials::try_get(&name)?;
        let host = Host::parse("127.0.0.12").expect("Literal is valid");
        debug!("Building client using username {username} from dbus");
        Client::builder(host)
            .basic_authentication(&username, &password)
            .build_with_scheme(Scheme::Plain)
    }
}
