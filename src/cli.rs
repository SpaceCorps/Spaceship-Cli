//! The command tree and argument parsing definitions for Spaceship CLI.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "spaceship",
    version,
    about = "CLI for the Spaceship API - domains, DNS records, contacts, SellerHub, Hyperlift and more",
    after_help = "An LLM agent should start with: spaceship agent-readme",
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Print raw JSON instead of YAML, for scripting
    #[arg(long, global = true)]
    pub json: bool,

    /// Output format: yaml or json (for compatibility)
    #[arg(long, value_name = "FORMAT", global = true)]
    pub format: Option<String>,

    /// Print HTTP method, URL, and status code to stderr
    #[arg(long, global = true)]
    pub verbose: bool,

    #[command(flatten)]
    pub auth: AuthArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args, Clone, Default)]
pub struct AuthArgs {
    /// Account to run against (see 'spaceship accounts list')
    #[arg(short = 'a', long, value_name = "ACCOUNT", global = true)]
    pub account: Option<String>,

    /// Override the SPACESHIP_API_KEY environment variable
    #[arg(long, value_name = "KEY", global = true)]
    pub api_key: Option<String>,

    /// Override the SPACESHIP_API_SECRET environment variable
    #[arg(long, value_name = "SECRET", global = true)]
    pub api_secret: Option<String>,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Print the operating manual for an LLM agent driving this CLI
    AgentReadme,
    /// Log in and store API credentials securely in the OS keystore
    Login(LoginArgs),
    /// Manage Spaceship accounts and their credentials
    #[command(subcommand)]
    Accounts(AccountsCommand),
    /// Domain management
    #[command(subcommand)]
    Domains(DomainsCommand),
    /// DNS record management
    #[command(subcommand)]
    Dns(DnsCommand),
    /// Contact management
    #[command(subcommand)]
    Contacts(ContactsCommand),
    /// SellerHub management
    #[command(subcommand)]
    Sellerhub(SellerhubCommand),
    /// Hyperlift application hosting
    #[command(subcommand)]
    Hyperlift(HyperliftCommand),
    /// Async operation tracking
    #[command(subcommand)]
    Operations(OperationsCommand),
}

// ---------------------------------------------------------------------------------------------
// login

#[derive(Args, Clone)]
pub struct LoginArgs {
    /// Account name to store (default: "default")
    #[arg(value_name = "NAME", default_value = "default")]
    pub name: String,

    /// Spaceship API key (prompted securely if omitted)
    #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
    pub api_key: Option<String>,

    /// Read the API key from stdin
    #[arg(long)]
    pub api_key_stdin: bool,

    /// Spaceship API secret (prompted securely if omitted)
    #[arg(long, value_name = "SECRET", conflicts_with = "api_secret_stdin")]
    pub api_secret: Option<String>,

    /// Read the API secret from stdin
    #[arg(long)]
    pub api_secret_stdin: bool,

    /// Replace credentials on an account that already exists
    #[arg(long)]
    pub force: bool,

    /// Store credentials without calling the API to verify first
    #[arg(long)]
    pub no_verify: bool,
}

// ---------------------------------------------------------------------------------------------
// accounts

#[derive(Subcommand)]
pub enum AccountsCommand {
    /// Add an account and store its credentials in the OS keystore
    Add {
        /// Short name for this account, used with --account
        name: String,
        /// Spaceship API key (prompted securely if omitted)
        #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
        api_key: Option<String>,
        /// Read the API key from stdin
        #[arg(long)]
        api_key_stdin: bool,
        /// Spaceship API secret (prompted securely if omitted)
        #[arg(long, value_name = "SECRET", conflicts_with = "api_secret_stdin")]
        api_secret: Option<String>,
        /// Read the API secret from stdin
        #[arg(long)]
        api_secret_stdin: bool,
        /// Replace credentials on an account that already exists
        #[arg(long)]
        force: bool,
        /// Store credentials without calling the API to verify first
        #[arg(long)]
        no_verify: bool,
    },
    /// List configured accounts
    List {
        /// Call the API to verify credentials for each account
        #[arg(long)]
        check: bool,
    },
    /// Check that an account's stored credentials still work
    Test {
        /// Account name
        name: String,
    },
    /// Remove an account and delete its stored credentials
    Remove {
        /// Account name
        name: String,
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

// ---------------------------------------------------------------------------------------------
// domains

#[derive(Subcommand)]
pub enum DomainsCommand {
    /// List all domains
    List {
        /// Number of items to return (1-100)
        #[arg(long, default_value = "20")]
        take: u32,
        /// Number of items to skip
        #[arg(long, default_value = "0")]
        skip: u32,
        /// Sort by: name, unicodeName, registrationDate, expirationDate
        #[arg(long)]
        order_by: Option<String>,
    },
    /// Domain details
    Get {
        /// Domain name (ASCII format)
        domain: String,
    },
    /// Check domain availability
    Check {
        /// Domain name to check
        domain: String,
    },
    /// Check availability of multiple domains (max 20)
    CheckBatch {
        /// Comma-separated domain names to check
        domains: String,
    },
    /// Register a domain
    Register {
        /// Domain name to register
        domain: String,
        /// Registration period in years (1-10)
        #[arg(long, default_value = "1")]
        years: u32,
        /// Enable auto-renewal
        #[arg(long)]
        auto_renew: bool,
        /// Privacy level: public or high
        #[arg(long, default_value = "high")]
        privacy: String,
        /// Registrant contact ID
        #[arg(long)]
        registrant: String,
        /// Admin contact ID
        #[arg(long)]
        admin: Option<String>,
        /// Tech contact ID
        #[arg(long)]
        tech: Option<String>,
        /// Billing contact ID
        #[arg(long)]
        billing: Option<String>,
    },
    /// Delete a domain
    Delete {
        /// Domain name to delete
        domain: String,
    },
    /// Renew a domain
    Renew {
        /// Domain name to renew
        domain: String,
        /// Renewal period in years (1-10)
        #[arg(long, default_value = "1")]
        years: u32,
        /// Current expiration date (ISO 8601)
        #[arg(long)]
        expiration_date: String,
    },
    /// Restore a deleted domain
    Restore {
        /// Domain name to restore
        domain: String,
    },
    /// Update domain auto-renewal
    Autorenew {
        /// Domain name
        domain: String,
        /// Enable auto-renewal
        #[arg(long)]
        enable: bool,
        /// Disable auto-renewal
        #[arg(long)]
        disable: bool,
    },
    /// Update domain nameservers
    Nameservers {
        /// Domain name
        domain: String,
        /// Nameserver provider: basic or custom
        #[arg(long)]
        provider: String,
        /// Comma-separated nameserver hosts (required for custom, 2-12)
        #[arg(long)]
        hosts: Option<String>,
    },
    /// Update domain contacts
    Contacts {
        /// Domain name
        domain: String,
        /// Registrant contact ID
        #[arg(long)]
        registrant: String,
        /// Admin contact ID
        #[arg(long)]
        admin: Option<String>,
        /// Tech contact ID
        #[arg(long)]
        tech: Option<String>,
        /// Billing contact ID
        #[arg(long)]
        billing: Option<String>,
    },
    /// Update domain privacy preference
    Privacy {
        /// Domain name
        domain: String,
        /// Privacy level: public or high
        #[arg(long)]
        level: String,
    },
    /// Update domain email protection (contact form) preference
    EmailProtection {
        /// Domain name
        domain: String,
        /// Enable the contact form
        #[arg(long)]
        enable: bool,
        /// Disable the contact form
        #[arg(long)]
        disable: bool,
    },
    /// Get domain transfer auth code
    AuthCode {
        /// Domain name
        domain: String,
    },
    /// Request domain transfer
    Transfer {
        /// Domain name to transfer
        domain: String,
        /// Enable auto-renewal
        #[arg(long)]
        auto_renew: bool,
        /// Privacy level: public or high
        #[arg(long, default_value = "high")]
        privacy: String,
        /// Registrant contact ID
        #[arg(long)]
        registrant: String,
        /// Admin contact ID
        #[arg(long)]
        admin: Option<String>,
        /// Tech contact ID
        #[arg(long)]
        tech: Option<String>,
        /// Billing contact ID
        #[arg(long)]
        billing: Option<String>,
        /// Domain transfer authorization code
        #[arg(long)]
        auth_code: Option<String>,
    },
    /// Get domain transfer status
    TransferStatus {
        /// Domain name
        domain: String,
    },
    /// Update domain transfer lock
    TransferLock {
        /// Domain name
        domain: String,
        /// Lock the domain
        #[arg(long)]
        lock: bool,
        /// Unlock the domain
        #[arg(long)]
        unlock: bool,
    },
    /// Personal nameservers (glue records)
    #[command(subcommand)]
    PersonalNs(PersonalNsCommand),
}

#[derive(Subcommand)]
pub enum PersonalNsCommand {
    /// List personal nameservers (glue records)
    List {
        /// Domain name
        domain: String,
    },
    /// Create or update a personal nameserver
    Save {
        /// Domain name
        domain: String,
        /// Nameserver host, e.g. ns1 or ns1.example.com
        host: String,
        /// Comma-separated IPv4/IPv6 addresses (1-16)
        #[arg(long)]
        ips: String,
        /// New host name; the old name stops resolving
        #[arg(long)]
        rename: Option<String>,
    },
    /// Delete a personal nameserver
    Delete {
        /// Domain name
        domain: String,
        /// Nameserver host, e.g. ns1 or ns1.example.com
        host: String,
    },
}

// ---------------------------------------------------------------------------------------------
// dns

#[derive(Subcommand)]
pub enum DnsCommand {
    /// List DNS records
    List {
        /// Domain name
        domain: String,
        /// Records per page (1-500)
        #[arg(long, default_value = "100")]
        take: u32,
        /// Number of records to skip
        #[arg(long, default_value = "0")]
        skip: u32,
        /// Fetch every page instead of one
        #[arg(long)]
        all: bool,
        /// Sort by: type, -type, name, -name
        #[arg(long)]
        order_by: Option<String>,
        /// Only records of this type, e.g. CNAME (filtered locally across all pages)
        #[arg(long = "type")]
        record_type: Option<String>,
        /// Only records with this name, e.g. www or @ (filtered locally across all pages)
        #[arg(long)]
        name: Option<String>,
    },
    /// Add DNS records, or update the TTL of matching ones
    Save {
        /// Domain name
        domain: String,
        /// JSON file with records array (or pipe via stdin)
        #[arg(long)]
        file: Option<String>,
        /// Turn off the API's conflict checks and force the zone update
        #[arg(long)]
        force: bool,
    },
    /// Delete DNS records
    Delete {
        /// Domain name
        domain: String,
        /// JSON file with records array to delete (or pipe via stdin)
        #[arg(long)]
        file: Option<String>,
    },
}

// ---------------------------------------------------------------------------------------------
// contacts

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ContactsCommand {
    /// Save contact details
    Save {
        /// First name
        #[arg(long)]
        first_name: String,
        /// Last name
        #[arg(long)]
        last_name: String,
        /// Email address
        #[arg(long)]
        email: String,
        /// Primary address
        #[arg(long)]
        address: String,
        /// City
        #[arg(long)]
        city: String,
        /// ISO 3166-1 alpha-2 country code
        #[arg(long)]
        country: String,
        /// Phone number as `+<country code>.<number>`, e.g. +46.701234567
        #[arg(long)]
        phone: String,
        /// Phone extension
        #[arg(long)]
        phone_ext: Option<String>,
        /// Fax number, same format as --phone
        #[arg(long)]
        fax: Option<String>,
        /// Fax extension
        #[arg(long)]
        fax_ext: Option<String>,
        /// Tax number
        #[arg(long)]
        tax_number: Option<String>,
        /// Organization name
        #[arg(long)]
        organization: Option<String>,
        /// Secondary address
        #[arg(long)]
        address2: Option<String>,
        /// State or province
        #[arg(long)]
        state: Option<String>,
        /// Postal code
        #[arg(long)]
        postal_code: Option<String>,
    },
    /// Get contact details
    Get {
        /// Contact ID
        id: String,
    },
    /// Contact attributes (registry-specific data)
    #[command(subcommand)]
    Attributes(ContactAttributesCommand),
}

#[derive(Subcommand)]
pub enum ContactAttributesCommand {
    /// Save contact attributes; returns the attributes ID
    Save {
        /// JSON file with attributes (or pipe via stdin)
        #[arg(long)]
        file: Option<String>,
    },
    /// Get contact attributes
    Get {
        /// Contact ID
        id: String,
    },
}

// ---------------------------------------------------------------------------------------------
// sellerhub

#[derive(Subcommand)]
pub enum SellerhubCommand {
    /// List SellerHub domains
    List {
        /// Number of items to return (1-100)
        #[arg(long, default_value = "20")]
        take: u32,
        /// Number of items to skip
        #[arg(long, default_value = "0")]
        skip: u32,
    },
    /// Get SellerHub domain details
    Get {
        /// Listed domain name
        domain: String,
    },
    /// Create SellerHub domain
    Create {
        /// JSON file with domain details (or pipe via stdin)
        #[arg(long)]
        file: Option<String>,
    },
    /// Update SellerHub domain
    Update {
        /// Listed domain name
        domain: String,
        /// JSON file with fields to update (or pipe via stdin)
        #[arg(long)]
        file: Option<String>,
    },
    /// Delete SellerHub domain
    Delete {
        /// Listed domain name
        domain: String,
    },
    /// Create SellerHub checkout link
    Checkout {
        /// JSON file with checkout details (or pipe via stdin)
        #[arg(long)]
        file: Option<String>,
    },
    /// Get SellerHub ownership verification record options
    Verification,
    /// List sold SellerHub domains
    Sold {
        /// Number of items to return (1-100)
        #[arg(long, default_value = "20")]
        take: u32,
        /// Cursor from previous page
        #[arg(long)]
        cursor: Option<String>,
        /// Only sales at or after this time (ISO 8601)
        #[arg(long)]
        from: Option<String>,
        /// Only sales before this time (ISO 8601)
        #[arg(long)]
        to: Option<String>,
    },
    /// SafePay (escrow) transactions
    #[command(subcommand)]
    Safepay(SafepayCommand),
}

#[derive(Subcommand)]
pub enum SafepayCommand {
    /// List SafePay transactions
    List {
        /// Number of items to return (1-100)
        #[arg(long, default_value = "20")]
        take: u32,
        /// Number of items to skip
        #[arg(long, default_value = "0")]
        skip: u32,
    },
    /// Get a SafePay transaction
    Get {
        /// SafePay transaction ID
        transaction_id: String,
    },
    /// Create a SafePay (escrow) transaction
    Create {
        /// JSON file with transaction (or pipe via stdin)
        #[arg(long)]
        file: Option<String>,
    },
}

// ---------------------------------------------------------------------------------------------
// hyperlift

#[derive(Subcommand)]
pub enum HyperliftCommand {
    /// List Hyperlift applications
    List {
        /// Number of items to return (1-100)
        #[arg(long, default_value = "20")]
        take: u32,
        /// Number of items to skip
        #[arg(long, default_value = "0")]
        skip: u32,
    },
    /// Get a Hyperlift application
    Get {
        /// Application ID
        id: String,
    },
    /// Start a build of a Hyperlift application
    Build {
        /// Application ID
        id: String,
    },
    /// Get build logs
    BuildLogs {
        /// Application ID
        id: String,
        /// Number of log lines to return (1-100)
        #[arg(long, default_value = "100")]
        take: u32,
        /// Cursor from previous page
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Get runtime logs
    Logs {
        /// Application ID
        id: String,
        /// Number of log lines to return (1-100)
        #[arg(long, default_value = "100")]
        take: u32,
        /// Cursor from previous page
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Get application metrics
    Metrics {
        /// Application ID
        id: String,
        /// Start of range, UTC ISO 8601
        #[arg(long)]
        start: String,
        /// End of range, UTC ISO 8601
        #[arg(long)]
        end: String,
        /// Bucket size, e.g. 10m
        #[arg(long)]
        interval: String,
        /// Comma-separated metric names
        #[arg(long)]
        metrics: String,
    },
    /// Restart a Hyperlift application
    Restart {
        /// Application ID
        id: String,
    },
    /// Start (1) or stop (0) a Hyperlift application
    Scale {
        /// Application ID
        id: String,
        /// 0 stops the application, 1 starts it
        #[arg(long)]
        scale: u32,
    },
    /// Environment variables
    #[command(subcommand)]
    Env(HyperliftEnvCommand),
}

#[derive(Subcommand)]
pub enum HyperliftEnvCommand {
    /// Get environment variables
    Get {
        /// Application ID
        id: String,
    },
    /// Replace ALL environment variables
    Set {
        /// Application ID
        id: String,
        /// JSON map of NAME to value (or pipe via stdin)
        #[arg(long)]
        file: Option<String>,
    },
}

// ---------------------------------------------------------------------------------------------
// operations

#[derive(Subcommand)]
pub enum OperationsCommand {
    /// Get async operation status
    Get {
        /// Async operation ID
        id: String,
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn command_tree_is_valid() {
        use clap::CommandFactory;
        super::Cli::command().debug_assert();
    }
}
