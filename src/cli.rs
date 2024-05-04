use clap::{Args, Parser, Subcommand};

// Top level clap::Command
#[derive(Parser)]
#[clap(name = "mycli")]
pub struct Cli {
    /// Error messages are suppressed
    #[clap(long, short, global = true, default_value_t = false)]
    pub quiet: bool,
    /// Whether to disable retry of throttled requests which would incure sleeps
    #[clap(long, global = true, default_value_t = false)]
    pub no_retry: bool,
    /// Maximum time to wait between retries of throttled requests
    #[clap(long, global = true, required = false)]
    pub max_wait: Option<u64>,
    /// Maximum number of retries per request
    #[clap(long, global = true, required = false)]
    pub max_retries: Option<usize>,
    // Subcommands
    // You can only have one subcommand section
    // so we point this to the Commands struct
    #[structopt(subcommand)]
    pub command: Command,
    // Add global-level flags here
}

// Second tier of commands -  clap::Subcommand
// Normally this would point to a clap::Args object,
// but in this case we are passing in another clap::Subcommand
// This enum defines our subcommand groups and passes on down
#[derive(Subcommand)]
pub enum Command {
    /// Manage account or create a new one
    #[clap(name = "account")]
    Account(Account),
    /// Manage domains
    #[clap(name = "domain")]
    Domain(Domain),
    /// Manage Resource Record Sets
    #[clap(name = "rrset")]
    ResourceRecordSet(ResourceRecordSet),
    /// Manage Token
    #[clap(name = "token")]
    Token(Token),
    /// Manage Token Policies
    #[clap(name = "policy")]
    TokenPolicy(TokenPolicy),
}

// The 'account' command itself
// The only way this differs from the Commands struct
// is that it directes clap::Parser, instead of clap::Subcommand
// This is so it gains the ability to auto-parse into
// the structure tree as a clap::Command with args, instead of as
// a clap::Args object
#[derive(Parser)]
pub struct Account {
    #[structopt(subcommand)]
    pub command: AccountCommand,
}

#[derive(Parser)]
pub struct Domain {
    #[structopt(subcommand)]
    pub command: DomainCommand,
}

#[derive(Parser)]
pub struct ResourceRecordSet {
    #[structopt(subcommand)]
    pub command: ResourceRecordSetCommand,
}

#[derive(Parser)]
pub struct Token {
    #[structopt(subcommand)]
    pub command: TokenCommand,
}

#[derive(Parser)]
pub struct TokenPolicy {
    #[structopt(subcommand)]
    pub command: TokenPolicyCommand,
}

// The command enum for the 'account' command
#[derive(Subcommand, Clone)]
pub enum AccountCommand {
    /// Shows the accounts information
    Show,
    /// Retrieve a captcha
    Captcha,
    /// Register a new account
    Register(RegisterArgs),
    /// Login
    Login(LoginArgs),
}

// The command enum for the 'domain' command
#[derive(Subcommand, Clone)]
pub enum DomainCommand {
    Get(DomainNameArg),
    List,
    Create(DomainNameArg),
    Delete(DomainNameArg),
    Responsible(DomainNameArg),
    Export(DomainNameArg),
}

// The command enum for the 'rrset' command
#[derive(Subcommand, Clone)]
pub enum ResourceRecordSetCommand {
    Get(ResourceRecordSetGetArgs),
    List(ResourceRecordSetListArgs),
    Create(ResourceRecordSetCreateArgs),
    Delete(ResourceRecordSetDeleteArgs),
}

// The command enum for the 'token' command
#[derive(Subcommand, Clone)]
pub enum TokenCommand {
    List,
    Get(TokenIdArgs),
    Create(TokenCreateArgs),
    Delete(TokenIdArgs),
    Patch(TokenPatchArgs),
}

// The command enum for the 'policy' command
#[derive(Subcommand, Clone)]
pub enum TokenPolicyCommand {
    List(TokenPolicyListArgs),
    Get(TokenPolicyGetArgs),
    Create(TokenPolicyCreateArgs),
    Delete(TokenPolicyDeleteArgs),
    Patch(TokenPolicyPatchArgs),
}

// The final clap::Args struct for the domain get command
#[derive(Args, Clone)]
pub struct DomainNameArg {
    /// The name off the domain to get
    pub name: String,
}

// The final clap::Args struct for the account register command
#[derive(Args, Clone)]
pub struct RegisterArgs {
    /// The email address for the new account
    #[clap(index = 1)]
    pub email: String,
    /// The password for the new account
    #[clap(index = 2)]
    pub password: String,
    /// The id of the solved captcha
    #[clap(index = 3)]
    pub id: String,
    /// The solution for the captcha
    #[clap(index = 4)]
    pub solution: String,
    /// Optional domain to create with new account
    #[clap(index = 5)]
    pub domain: Option<String>,
}

// The final clap::Args struct for the account login command
#[derive(Args, Clone)]
pub struct LoginArgs {
    /// The email address used to login
    #[clap(index = 1)]
    pub email: String,
    /// The password usede to login
    #[clap(index = 2)]
    pub password: String,
}

#[derive(Args, Clone)]
pub struct ResourceRecordSetGetArgs {
    /// The domain name
    #[clap(index = 1)]
    pub name: String,
    /// The subname for the rrset
    #[clap(index = 2)]
    pub subname: String,
    /// The type of rrset
    #[clap(index = 3)]
    pub r#type: String,
}

#[derive(Args, Clone)]
pub struct ResourceRecordSetListArgs {
    /// The domain name
    #[clap(index = 1)]
    pub name: String,
}

#[derive(Args, Clone)]
pub struct ResourceRecordSetCreateArgs {
    /// The domain name
    #[clap(index = 1)]
    pub name: String,
    /// The subname for the rrset
    #[clap(index = 2)]
    pub subname: String,
    /// The type of rrset
    #[clap(index = 3)]
    pub r#type: String,
    /// TTL of the rrset
    #[clap(index = 4)]
    pub ttl: u64,
    /// TTL of the rrset
    #[clap(index = 5)]
    pub records: Vec<String>,
}

#[derive(Args, Clone)]
pub struct ResourceRecordSetDeleteArgs {
    /// The domain name
    #[clap(index = 1)]
    pub name: String,
    /// The subname for the rrset
    #[clap(index = 2)]
    pub subname: String,
    /// The type of rrset
    #[clap(index = 3)]
    pub r#type: String,
}

#[derive(Args, Clone)]
pub struct TokenIdArgs {
    /// The token id
    pub token_id: String,
}

#[derive(Args, Clone)]
pub struct TokenCreateArgs {
    /// Name for the token
    #[clap(long)]
    pub name: Option<String>,
    /// Allowed subnets
    #[clap(long)]
    #[arg(num_args(0..))]
    pub subnets: Option<Vec<String>>,
    /// Can manage tokens
    #[clap(long)]
    pub manage: Option<bool>,
    /// Maximum age for the new token
    #[clap(long)]
    pub max_age: Option<String>,
    /// Maximum unused period before automatic invalidation
    #[clap(long)]
    pub max_unused_period: Option<String>,
}

#[derive(Args, Clone)]
pub struct TokenPatchArgs {
    /// Id of the token to patch
    #[clap(long)]
    pub token_id: String,
    /// Name for the token
    #[clap(long)]
    pub name: Option<String>,
    /// Allowed subnets
    #[clap(long)]
    #[arg(num_args(0..))]
    pub subnets: Option<Vec<String>>,
    /// Can manage tokens
    #[clap(long)]
    pub manage: Option<bool>,
    /// Maximum age for the new token
    #[clap(long)]
    pub max_age: Option<String>,
    /// Maximum unused period before automatic invalidation
    #[clap(long)]
    pub max_unused_period: Option<String>,
}

#[derive(Args, Clone)]
pub struct TokenPolicyListArgs {
    /// Id of the token to create a policy for
    pub token_id: String,
}

#[derive(Args, Clone)]
pub struct TokenPolicyGetArgs {
    /// Id of the token to get a policy for
    pub token_id: String,
    /// Id of the policy to get. If missing, the default policy is returned.
    pub policy_id: String
}

#[derive(Args, Clone)]
pub struct TokenPolicyDeleteArgs {
    /// Id of the token to create a policy for
    pub token_id: String,
    /// Id of the policy to modify
    pub policy_id: String,
}

#[derive(Args, Clone)]
pub struct TokenPolicyCreateArgs {
    /// Id of the token to create a policy for
    pub token_id: String,
    /// Domain name to which the policy applies. None for the default policy.
    pub domain: Option<String>,
    /// Subname to which the policy applies. None for the default policy.
    pub subname: Option<String>,
    /// Record type to which the policy applies. None for the default policy.
    pub r#type: Option<String>,
    /// Indicates write permission for the RRset specified by (domain, subname, type)
    /// when using the general RRset management or dynDNS interface. Defaults to false.
    pub perm_write: Option<bool>,
}

#[derive(Args, Clone)]
pub struct TokenPolicyPatchArgs {
    /// Id of the token to modify a policy for
    pub token_id: String,
    /// Id of the token policy to modify
    pub policy_id: String,
    /// Domain name to which the policy applies. Empty string for the default policy.
    pub domain: Option<String>,
    /// Subname to which the policy applies. Empty string for the default policy.
    pub subname: Option<String>,
    /// Record type to which the policy applies. Empty string for the default policy.
    pub r#type: Option<String>,
    /// Indicates write permission for the RRset specified by (domain, subname, type)
    /// when using the general RRset management or dynDNS interface. Defaults to false.
    pub perm_write: Option<bool>,
}
