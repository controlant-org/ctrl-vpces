use aws_types::region::Region;
use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
  /// AWS IAM roles to assume, repeat to provide multiple
  #[clap(long, short, conflicts_with_all = ["root_role", "sub_role"])]
  assume: Option<Vec<String>>,
  /// AWS IAM role used to fetch all accounts from an organization
  #[clap(long, requires = "sub_role")]
  root_role: Option<String>,
  /// AWS IAM role name (with path, but not the full ARN) the controller will try to assume in all accounts under the organization, discovery controlled by `root_role` or using local credentials
  #[clap(long)]
  sub_role: Option<String>,
  /// AWS regions to manage, repeat to list all regions to manage. If not specified, simply loads the current region.
  #[clap(long, short)]
  region: Option<Vec<String>>,
  /// The tag "key" to use for endpoint service management
  #[clap(long, short, default_value = "endpoint.controlant.com/managed")]
  endpoint_key: String,
  /// Read and generate modification actions but do not actually execute them
  #[clap(long)]
  dry_run: bool,
  /// Run controller logic just once, instead of running as a service
  #[clap(long)]
  once: bool
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert()
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct App {
  pub accounts: Vec<String>,
  pub auth_mode: AuthMode,
  pub regions: Option<Vec<Region>>,
  pub dry_run: bool,
  pub once: bool,
  pub endpoint_key: String
}

#[derive(Debug, Clone)]
pub enum AuthMode {
  /// Use the default credentials provider chain on local machine
  Local,
  /// Use the provided list of roles to assume
  Assume(Vec<String>),
  /// Discover accounts and roles to assume from an organization
  Discover(Option<String>, String),
}

impl App {
  pub fn from_cli() -> Self {
    let cli = Cli::parse();
    let auth_mode = match (cli.assume, cli.root_role, cli.sub_role) {
      (Some(roles), _, _) => AuthMode::Assume(roles),
      (None, root_role, Some(sub_role)) => AuthMode::Discover(root_role, sub_role),
      (None, None, None) => AuthMode::Local,
      _ => panic!("invalid auth mode"),
    };

    Self {
      accounts: Vec::new(),
      auth_mode,
      regions: cli.region.map(|rs| {
        rs.iter()
          .map(|x| aws_types::region::Region::new(x.clone()))
          .collect::<Vec<_>>()
      }),
      dry_run: cli.dry_run,
      once: cli.once,
      endpoint_key: cli.endpoint_key
    }
  }
}
