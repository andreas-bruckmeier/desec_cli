use clap::Parser;
use desec_api::{account, Client, Error};
use std::env;
use std::process::ExitCode;
use serde::Serialize;

mod cli;

use cli::*;

#[tokio::main]
async fn main() -> ExitCode {
    #[cfg(feature = "logging")]
    env_logger::init();

    let cli = Cli::parse();

    // Create a new client from either a token from env var DESEC_API_TOKEN
    // or from credentials in env vars DESEC_EMAIL & DESEC_PASSWORD.
    // If we have neither a token nor credentials, we abort.
    let mut client = if let Ok(token) = env::var("DESEC_API_TOKEN") {
        match Client::new(token) {
            Ok(c) => c,
            Err(Error::ReqwestClientBuilder(e)) => panic!("{e}"),
            _ => unreachable!(),
        }
    } else if let (Ok(email), Ok(password)) = (env::var("DESEC_EMAIL"), env::var("DESEC_PASSWORD"))
    {
        match Client::new_from_credentials(&email, &password).await {
            Ok(c) => c,
            Err(Error::ReqwestClientBuilder(e)) => panic!("{e}"),
            _ => unreachable!(),
        }
    } else {
        eprintln!("Missing env var TOKEN_ENV_VAR");
        return ExitCode::FAILURE;
    };

    if let Some(max_retries) = cli.max_retries {
        client.set_max_retries(max_retries);
    }

    if let Some(max_wait) = cli.max_wait {
        client.set_max_wait_retry(max_wait);
    }

    client.set_retry(!cli.no_retry);

    match &cli.command {
        Command::Account(subcommand) => match &subcommand.command {
            AccountCommand::Captcha => return get_captcha(&cli).await,
            AccountCommand::Register(args) => return register(&cli, args).await,
            AccountCommand::Login(args) => return login(&cli, args).await,
            AccountCommand::Show => return show_account(&cli, &client).await,
        },
        Command::Domain(args) => match &args.command {
            DomainCommand::List => return list_domains(&cli, &client).await,
            DomainCommand::Get(args) => return get_domain(&cli, &client, args).await,
            DomainCommand::Create(args) => return create_domain(&cli, &client, args).await,
            DomainCommand::Delete(args) => return delete_domain(&client, args).await,
            DomainCommand::Responsible(args) => return get_domain_responsible(&cli, &client, args).await,
            DomainCommand::Export(args) => return export_domain(&client, args).await,
        },
        Command::ResourceRecordSet(subcommand) => match &subcommand.command {
            ResourceRecordSetCommand::List(args) => return get_all_rrsets(&cli, &client, args).await,
            ResourceRecordSetCommand::Get(args) => return get_rrset(&cli, &client, args).await,
            ResourceRecordSetCommand::Create(args) => return create_rrset(&cli, &client, args).await,
            ResourceRecordSetCommand::Delete(args) => {
                return delete_rrset(&cli, &client, args).await
            }
        },
        Command::Token(subcommand) => match &subcommand.command {
            TokenCommand::List => return list_token(&cli, &client).await,
            TokenCommand::Get(args) => return get_token(&cli, &client, args).await,
            TokenCommand::Create(args) => return create_token(&cli, &client, args).await,
            TokenCommand::Delete(args) => return delete_token(&client, args).await,
            TokenCommand::Patch(args) => return patch_token(&cli, &client, args).await,
        },
        Command::TokenPolicy(subcommand) => match &subcommand.command {
            TokenPolicyCommand::List(args) => return list_token_policies(&cli, &client, args).await,
            TokenPolicyCommand::Create(args) => return create_token_policy(&client, args).await,
            TokenPolicyCommand::Get(args) => return get_token_policy(&cli, &client, args).await,
            TokenPolicyCommand::Patch(args) => return patch_token_policy(&client, args).await,
            TokenPolicyCommand::Delete(args) => return delete_token_policy(&client, args).await,
        },
    }
}

fn print_formatted<T: Serialize>(arg: T, format: &OutputFormat) {
    let output = match format {
        OutputFormat::Json => serde_json::to_string(&arg).expect("Serializing should never fail here"),
        #[cfg(feature = "yaml")]
        OutputFormat::Yaml => serde_yaml::to_string(&arg).expect("Serializing should never fail here"),
    };
    println!("{output}");
}

async fn get_captcha(cli: &Cli) -> ExitCode {
    let captcha = match account::get_captcha().await {
        Ok(captcha) => captcha,
        Err(error) => {
            eprintln!("An error occurred: {}", error);
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&captcha, &cli.format);
    ExitCode::SUCCESS
}

async fn register(cli: &Cli, args: &RegisterArgs) -> ExitCode {
    let account = match account::register(
        &args.email,
        &args.password,
        &args.id,
        &args.solution,
        args.domain.as_deref(),
    )
    .await
    {
        Ok(account) => account,
        Err(error) => {
            eprintln!("An error occurred: {}", error);
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&account, &cli.format);
    ExitCode::SUCCESS
}

async fn login(cli: &Cli, args: &LoginArgs) -> ExitCode {
    let login = match account::login(&args.email, &args.password).await {
        Ok(login) => login,
        Err(Error::ReqwestClientBuilder(e)) => panic!("{e}"),
        _ => unreachable!(),
    };
    print_formatted(&login, &cli.format);
    ExitCode::SUCCESS
}

async fn show_account(cli: &Cli, client: &Client) -> ExitCode {
    let account_info = match client.account().get_account_info().await {
        Ok(info) => info,
        Err(_) => {
            eprintln!("An error occurred");
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&account_info, &cli.format);
    ExitCode::SUCCESS
}

async fn create_domain(cli: &Cli, client: &Client, args: &DomainNameArg) -> ExitCode {
    let domain_name = &args.name;
    let domain = match client.domain().create_domain(domain_name).await {
        Ok(domain) => domain,
        Err(error) => {
            eprintln!("Creation of domain failed: {}", error);
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&domain, &cli.format);
    ExitCode::SUCCESS
}

async fn list_domains(cli: &Cli, client: &Client) -> ExitCode {
    let domains = match client.domain().get_domains().await {
        Ok(domains) => domains,
        Err(_) => {
            eprintln!("An error occurred");
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&domains, &cli.format);
    ExitCode::SUCCESS
}

async fn get_domain(cli: &Cli, client: &Client, args: &DomainNameArg) -> ExitCode {
    let domain_name = &args.name;
    let domain = match client.domain().get_domain(domain_name).await {
        Ok(domain) => domain,
        Err(Error::NotFound) => {
            eprintln!(
                "Domain {} does not exist or you are not the owner",
                domain_name
            );
            return ExitCode::FAILURE;
        }
        Err(_) => {
            eprintln!("An error occurred");
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&domain, &cli.format);
    ExitCode::SUCCESS
}

async fn get_domain_responsible(cli: &Cli, client: &Client, args: &DomainNameArg) -> ExitCode {
    let domain_name = &args.name;
    let domain = match client.domain().get_owning_domain(domain_name).await {
        Ok(domain) => domain,
        Err(Error::NotFound) => {
            eprintln!(
                "Domain {} does not exist or you are not the owner",
                domain_name
            );
            return ExitCode::FAILURE;
        }
        Err(_) => {
            eprintln!("An error occurred");
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&domain, &cli.format);
    ExitCode::SUCCESS
}

async fn export_domain(client: &Client, args: &DomainNameArg) -> ExitCode {
    let domain_name = &args.name;
    let zonefile = match client.domain().get_zonefile(domain_name).await {
        Ok(domain) => domain,
        Err(Error::NotFound) => {
            eprintln!(
                "Domain {} does not exist or you are not the owner",
                domain_name
            );
            return ExitCode::FAILURE;
        }
        Err(_) => {
            eprintln!("An error occurred");
            return ExitCode::FAILURE;
        }
    };
    println!("{zonefile}");
    ExitCode::SUCCESS
}

async fn delete_domain(client: &Client, args: &DomainNameArg) -> ExitCode {
    if let Err(error) = client.domain().delete_domain(&args.name).await {
        eprintln!("Deletion of domain failed: {}", error);
        return ExitCode::FAILURE;
    };
    ExitCode::SUCCESS
}

async fn create_rrset(cli: &Cli, client: &Client, args: &ResourceRecordSetCreateArgs) -> ExitCode {
    let subname = if args.subname == "@" {
        None
    } else {
        Some(args.subname.as_str())
    };
    let rrset = match client
        .rrset()
        .create_rrset(&args.name, subname, &args.r#type, args.ttl, &args.records)
        .await
    {
        Ok(rrset) => rrset,
        Err(Error::NotFound) => {
            eprintln!(
                "RRSet {} does not exist or you are not the owner",
                args.name
            );
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("An error occurred: {}", error);
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&rrset, &cli.format);
    ExitCode::SUCCESS
}
async fn get_rrset(cli: &Cli, client: &Client, args: &ResourceRecordSetGetArgs) -> ExitCode {
    let subname = if args.subname == "@" {
        None
    } else {
        Some(args.subname.clone())
    };
    let rrset = match client
        .rrset()
        .get_rrset(&args.name, subname.as_deref(), &args.r#type)
        .await
    {
        Ok(rrset) => rrset,
        Err(Error::NotFound) => {
            eprintln!(
                "RRSet {} does not exist or you are not the owner",
                args.name
            );
            return ExitCode::FAILURE;
        }
        Err(_) => {
            eprintln!("An error occurred");
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&rrset, &cli.format);
    ExitCode::SUCCESS
}

async fn get_all_rrsets(cli: &Cli, client: &Client, args: &ResourceRecordSetListArgs) -> ExitCode {
    let rrset = match client.rrset().get_rrsets(&args.name).await {
        Ok(rrset) => rrset,
        Err(Error::NotFound) => {
            eprintln!(
                "RRSet {} does not exist or you are not the owner",
                args.name
            );
            return ExitCode::FAILURE;
        }
        Err(_) => {
            eprintln!("An error occurred");
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&rrset, &cli.format);
    ExitCode::default()
}

async fn delete_rrset(cli: &Cli, client: &Client, args: &ResourceRecordSetDeleteArgs) -> ExitCode {
    let subname = if args.subname == "@" {
        None
    } else {
        Some(args.subname.as_str())
    };
    match client
        .rrset()
        .delete_rrset(&args.name, subname, &args.r#type)
        .await
    {
        Ok(_) => {
            if !cli.quiet {
                eprintln!(
                    "rrset {} {}.{} has been deleted or did not exist",
                    args.r#type, args.subname, args.name
                )
            }
        }
        Err(Error::NotFound) => {
            if !cli.quiet {
                eprintln!(
                    "RRSet {} does not exist or you are not the owner",
                    args.name
                );
            }
            return ExitCode::FAILURE;
        }
        Err(error) => {
            if !cli.quiet {
                eprintln!(
                    "Deletion of rrset {} {}.{} failed: {}",
                    args.r#type, args.subname, args.name, error
                );
            }
            return ExitCode::FAILURE;
        }
    };
    ExitCode::default()
}

async fn list_token(cli: &Cli, client: &Client) -> ExitCode {
    let tokens = match client.token().list().await {
        Ok(rrset) => rrset,
        Err(error) => {
            eprintln!("Failed to list tokens: {}", error);
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&tokens, &cli.format);
    ExitCode::default()
}

async fn get_token(cli: &Cli, client: &Client, args: &TokenIdArgs) -> ExitCode {
    let tokens = match client.token().get(&args.token_id).await {
        Ok(rrset) => rrset,
        Err(error) => {
            eprintln!("Failed to get token: {}", error);
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&tokens, &cli.format);
    ExitCode::default()
}

async fn create_token(cli: &Cli, client: &Client, args: &TokenCreateArgs) -> ExitCode {
    let tokens = match client
        .token()
        .create(
            args.name.clone(),
            args.subnets.clone(),
            args.manage,
            args.max_age.clone(),
            args.max_unused_period.clone(),
        )
        .await
    {
        Ok(rrset) => rrset,
        Err(error) => {
            eprintln!("Failed to get token: {}", error);
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&tokens, &cli.format);
    ExitCode::default()
}

async fn patch_token(cli: &Cli, client: &Client, args: &TokenPatchArgs) -> ExitCode {
    let tokens = match client
        .token()
        .patch(
            &args.token_id,
            args.name.clone(),
            args.subnets.clone(),
            args.manage,
            args.max_age.clone(),
            args.max_unused_period.clone(),
        )
        .await
    {
        Ok(rrset) => rrset,
        Err(error) => {
            eprintln!("Failed to patch token: {}", error);
            return ExitCode::FAILURE;
        }
    };
    print_formatted(&tokens, &cli.format);
    ExitCode::default()
}

async fn delete_token(client: &Client, args: &TokenIdArgs) -> ExitCode {
    match client.token().delete(&args.token_id).await {
        Ok(_) => (),
        Err(error) => {
            eprintln!("Failed to delete token: {}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::default()
}

async fn get_token_policy(cli: &Cli, client: &Client, args: &TokenPolicyGetArgs) -> ExitCode {
    match client.token().get_policy(&args.token_id, &args.policy_id).await {
        Ok(response) => print_formatted(&response, &cli.format),
        Err(error) => {
            eprintln!("Failed to get token policy: {}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::default()
}

async fn list_token_policies(cli: &Cli, client: &Client, args: &TokenPolicyListArgs) -> ExitCode {
    match client.token().list_policies(&args.token_id).await {
        Ok(response) => print_formatted(&response, &cli.format),
        Err(error) => {
            eprintln!("Failed to get list of token policies: {}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::default()
}

async fn create_token_policy(client: &Client, args: &TokenPolicyCreateArgs) -> ExitCode {
    match client
        .token()
        .create_policy(
            &args.token_id,
            args.domain.clone().filter(|d| !d.is_empty()),
            args.subname.clone().filter(|s| !s.is_empty()),
            args.r#type.clone().filter(|r| !r.is_empty()),
            args.perm_write,
        )
        .await
    {
        Ok(_) => (),
        Err(error) => {
            eprintln!("Failed to create the token policies: {}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::default()
}

async fn patch_token_policy(client: &Client, args: &TokenPolicyPatchArgs) -> ExitCode {
    match client
        .token()
        .patch_policy(
            &args.token_id,
            &args.policy_id,
            args.domain.clone().filter(|d| !d.is_empty()),
            args.subname.clone().filter(|s| !s.is_empty()),
            args.r#type.clone().filter(|r| !r.is_empty()),
            args.perm_write,
        )
        .await
    {
        Ok(_) => (),
        Err(error) => {
            eprintln!("Failed to create the token policies: {}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::default()
}

async fn delete_token_policy(client: &Client, args: &TokenPolicyDeleteArgs) -> ExitCode {
    match client
        .token()
        .delete_policy(&args.token_id, &args.policy_id)
        .await
    {
        Ok(_) => (),
        Err(error) => {
            eprintln!("Failed to delete the token policies: {}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::default()
}
