// use std::borrow::BorrowMut;

use aws_config::environment::EnvironmentVariableCredentialsProvider;
use aws_config::sts::AssumeRoleProvider;
use aws_sdk_s3::{types::SdkError as SssSdkError, Client as SssClient};
use aws_sdk_sts::{types::SdkError as StsSdkError, Client, Error, Region};

use clap::Parser;

use std::{fmt::Debug, sync::Arc};

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about, long_about=None)]
struct Args {
    #[clap(short, long)]
    arn: String,
    #[clap(short, long)]
    session_name: String,
    #[clap(long)]
    region: Option<String>,
    #[clap(short)]
    verbose: bool,
    #[clap(short, long)]
    bucket: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let ar_provider = AssumeRoleProvider::builder(args.arn.clone())
        .session_name(args.session_name)
        .region(Region::new(args.region.clone().unwrap()))
        .build(Arc::new(EnvironmentVariableCredentialsProvider::new()) as Arc<_>);
    let config = aws_config::from_env()
        .credentials_provider(ar_provider)
        .load()
        .await;
    let client = Client::new(&config);
    let caller_identity = client.get_caller_identity().send().await;
    match caller_identity {
        Result::Ok(identity) => {
            println!(
                "Assumed {} session {}\n",
                identity.user_id.unwrap_or_default(),
                identity.arn.unwrap_or_default()
            )
        }
        // The return type was a bit hard to find via google/ddg. Needed to
        // remember that I could find it via the get_caller_identity docs
        Result::Err(e) => match e {
            StsSdkError::ConstructionFailure(t) => println!("Construction failed: {}", t),
            StsSdkError::DispatchFailure(t) => println!("Dispatch failed: {}", t),
            StsSdkError::ResponseError { err, raw: _ } => println!("Response error: {}", err),
            StsSdkError::ServiceError { err, raw: _ } => println!("Service error: {}", err),
            StsSdkError::TimeoutError(t) => println!("Timeout error: {}", t),
        },
    }

    let s3client = SssClient::new(&config);
    let list_result = s3client
        .list_objects_v2()
        .bucket(args.bucket.clone().unwrap())
        .send()
        .await;

    match list_result {
        Result::Ok(objects) => {
            println!(
                "Succeeded in listing the bucket {} as {}",
                args.bucket.clone().unwrap_or_default(),
                args.arn.clone()
            )
        }
        Result::Err(e) => match e {
            SssSdkError::ConstructionFailure(t) => println!("Construction failed: {}", t),
            SssSdkError::DispatchFailure(t) => println!("Dispatch failed: {}", t),
            SssSdkError::ResponseError { err, raw: _ } => println!("Response error: {}", err),
            SssSdkError::ServiceError { err, raw: _ } => println!("Service error: {}", err),
            SssSdkError::TimeoutError(t) => println!("Timeout error: {}", t),
        },
    }

    Ok(())
}
