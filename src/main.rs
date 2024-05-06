use clap::Parser;
use desec_api::{account, Client, Error};
use std::env;
use std::process::ExitCode;

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
            AccountCommand::Captcha => return get_captcha().await,
            AccountCommand::Register(args) => return register(args).await,
            AccountCommand::Login(args) => return login(args).await,
            AccountCommand::Show => return show_account(&client).await,
        },
        Command::Domain(args) => match &args.command {
            DomainCommand::List => return list_domains(&client).await,
            DomainCommand::Get(args) => return get_domain(&client, args).await,
            DomainCommand::Create(args) => return create_domain(&client, args).await,
            DomainCommand::Delete(args) => return delete_domain(&client, args).await,
            DomainCommand::Responsible(args) => return get_domain_responsible(&client, args).await,
            DomainCommand::Export(args) => return export_domain(&client, args).await,
        },
        Command::ResourceRecordSet(subcommand) => match &subcommand.command {
            ResourceRecordSetCommand::List(args) => return get_all_rrsets(&client, args).await,
            ResourceRecordSetCommand::Get(args) => return get_rrset(&client, args).await,
            ResourceRecordSetCommand::Create(args) => return create_rrset(&client, args).await,
            ResourceRecordSetCommand::Delete(args) => {
                return delete_rrset(&cli, &client, args).await
            }
        },
        Command::Token(subcommand) => match &subcommand.command {
            TokenCommand::List => return list_token(&client).await,
            TokenCommand::Get(args) => return get_token(&client, args).await,
            TokenCommand::Create(args) => return create_token(&client, args).await,
            TokenCommand::Delete(args) => return delete_token(&client, args).await,
            TokenCommand::Patch(args) => return patch_token(&client, args).await,
        },
        Command::TokenPolicy(subcommand) => match &subcommand.command {
            TokenPolicyCommand::List(args) => return list_token_policies(&client, args).await,
            TokenPolicyCommand::Create(args) => return create_token_policy(&client, args).await,
            TokenPolicyCommand::Get(args) => return get_token_policy(&client, args).await,
            TokenPolicyCommand::Patch(args) => return patch_token_policy(&client, args).await,
            TokenPolicyCommand::Delete(args) => return delete_token_policy(&client, args).await,
        },
    }
}

async fn get_captcha() -> ExitCode {
    let captcha = match account::get_captcha().await {
        Ok(captcha) => captcha,
        Err(error) => {
            eprintln!("An error occurred: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let captcha_json = match serde_json::to_string(&captcha) {
        Ok(json) => json,
        Err(error) => panic!("{}", error),
    };
    println!("{captcha_json}");
    ExitCode::SUCCESS
}

async fn register(args: &RegisterArgs) -> ExitCode {
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
    let account_json = match serde_json::to_string(&account) {
        Ok(json) => json,
        Err(error) => panic!("{}", error),
    };
    println!("{account_json}");
    ExitCode::SUCCESS
}

async fn login(args: &LoginArgs) -> ExitCode {
    let login = match account::login(&args.email, &args.password).await {
        Ok(login) => login,
        Err(Error::ReqwestClientBuilder(e)) => panic!("{e}"),
        _ => unreachable!(),
    };
    let account_json = match serde_json::to_string(&login) {
        Ok(json) => json,
        Err(error) => panic!("{}", error),
    };
    println!("{account_json}");
    ExitCode::SUCCESS
}

async fn show_account(client: &Client) -> ExitCode {
    let account_info = match client.account().get_account_info().await {
        Ok(info) => info,
        Err(_) => {
            eprintln!("An error occurred");
            return ExitCode::FAILURE;
        }
    };
    let account_info_json = match serde_json::to_string(&account_info) {
        Ok(json) => json,
        Err(error) => panic!("{}", error),
    };
    println!("{account_info_json}");
    ExitCode::SUCCESS
}

async fn create_domain(client: &Client, args: &DomainNameArg) -> ExitCode {
    let domain_name = &args.name;
    let domain = match client.domain().create_domain(domain_name).await {
        Ok(domain) => domain,
        Err(error) => {
            eprintln!("Creation of domain failed: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let domain_json = match serde_json::to_string(&domain) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Failed to serialize the returned domain: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("{domain_json}");
    ExitCode::SUCCESS
}

async fn list_domains(client: &Client) -> ExitCode {
    let domains = match client.domain().get_domains().await {
        Ok(domains) => domains,
        Err(_) => {
            eprintln!("An error occurred");
            return ExitCode::FAILURE;
        }
    };
    let domains_json = match serde_json::to_string(&domains) {
        Ok(json) => json,
        Err(error) => panic!("{}", error),
    };
    println!("{domains_json}");
    ExitCode::SUCCESS
}

async fn get_domain(client: &Client, args: &DomainNameArg) -> ExitCode {
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
    let domain_json = match serde_json::to_string(&domain) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Failed to serialize the data: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("{domain_json}");
    ExitCode::SUCCESS
}

async fn get_domain_responsible(client: &Client, args: &DomainNameArg) -> ExitCode {
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
    let domain_json = match serde_json::to_string(&domain) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Failed to serialize the data: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("{domain_json}");
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

async fn create_rrset(client: &Client, args: &ResourceRecordSetCreateArgs) -> ExitCode {
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
    let rrset_json = match serde_json::to_string(&rrset) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Failed to serialize the data: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("{rrset_json}");

    ExitCode::SUCCESS
}
async fn get_rrset(client: &Client, args: &ResourceRecordSetGetArgs) -> ExitCode {
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
    let rrset_json = match serde_json::to_string(&rrset) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Failed to serialize the data: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("{rrset_json}");
    ExitCode::SUCCESS
}

async fn get_all_rrsets(client: &Client, args: &ResourceRecordSetListArgs) -> ExitCode {
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
    let rrset_json = match serde_json::to_string(&rrset) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Failed to serialize the data: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("{rrset_json}");
    ExitCode::SUCCESS
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
    ExitCode::SUCCESS
}

async fn list_token(client: &Client) -> ExitCode {
    let tokens = match client.token().list().await {
        Ok(rrset) => rrset,
        Err(error) => {
            eprintln!("Failed to list tokens: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let tokens_json = match serde_json::to_string(&tokens) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Failed to serialize tokin list: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("{tokens_json}");
    ExitCode::SUCCESS
}

async fn get_token(client: &Client, args: &TokenIdArgs) -> ExitCode {
    let tokens = match client.token().get(&args.token_id).await {
        Ok(rrset) => rrset,
        Err(error) => {
            eprintln!("Failed to get token: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let tokens_json = match serde_json::to_string(&tokens) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Failed to serialize tokin list: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("{tokens_json}");
    ExitCode::SUCCESS
}

async fn create_token(client: &Client, args: &TokenCreateArgs) -> ExitCode {
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
    let tokens_json = match serde_json::to_string(&tokens) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Failed to serialize tokin list: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("{tokens_json}");
    ExitCode::SUCCESS
}

async fn patch_token(client: &Client, args: &TokenPatchArgs) -> ExitCode {
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
    let tokens_json = match serde_json::to_string(&tokens) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Failed to serialize tokin list: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("{tokens_json}");
    ExitCode::SUCCESS
}

async fn delete_token(client: &Client, args: &TokenIdArgs) -> ExitCode {
    match client.token().delete(&args.token_id).await {
        Ok(_) => (),
        Err(error) => {
            eprintln!("Failed to delete token: {}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
}

async fn get_token_policy(client: &Client, args: &TokenPolicyGetArgs) -> ExitCode {
    match client.token().get_policy(&args.token_id, &args.policy_id).await {
        Ok(response) => match serde_json::to_string(&response) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("Failed to serialize tokin policy: {error}");
                return ExitCode::FAILURE;
            }
        },
        Err(error) => {
            eprintln!("Failed to get token policy: {}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
}

async fn list_token_policies(client: &Client, args: &TokenPolicyListArgs) -> ExitCode {
    match client.token().list_policies(&args.token_id).await {
        Ok(response) => match serde_json::to_string(&response) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("Failed to serialize tokin policy list: {error}");
                return ExitCode::FAILURE;
            }
        },
        Err(error) => {
            eprintln!("Failed to get list of token policies: {}", error);
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
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
    ExitCode::SUCCESS
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
    ExitCode::SUCCESS
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
    ExitCode::SUCCESS
}
