use anyhow::Result;
use aws_sdk_ec2::types::Filter;
use aws_types::sdk_config::SdkConfig;
use log::{debug, error, info, trace};

use crate::cli::App;

// TODO: error handling & reporting - i.e. no crash

pub async fn run(config: SdkConfig, app: &App, accounts: Vec<String>) -> Result<()> {
  trace!("aws env: {:?}", &config);

  // ignore non-existing role
  let sts = aws_sdk_sts::Client::new(&config);
  match sts.get_caller_identity().send().await {
    Ok(acc) => {
      info!(
        "working on account: {}",
        acc.account().expect("failed to extract account id")
      );
    }
    Err(e) => {
      debug!("ignore failed assume role: {:?}", e);
      return Ok(());
    }
  }

  // TODO: provide domain and/or tier based filtering
  let principals: Vec<String> = accounts
    .iter()
    .map(|acct_id| format!("arn:aws:iam::{}:root", acct_id))
    .collect();

  let ec2 = aws_sdk_ec2::Client::new(&config);

  // VPC Endpoint Services
  let mut vpces_stream = ec2
    .describe_vpc_endpoint_service_configurations()
    .filters(Filter::builder().name("tag-key").values(&app.endpoint_key).build())
    .into_paginator()
    .items()
    .send();

  while let Some(Ok(svc)) = vpces_stream.next().await {
    let svc_id = svc.service_id.unwrap();

    match ec2
      .modify_vpc_endpoint_service_permissions()
      .service_id(&svc_id)
      .set_add_allowed_principals(Some(principals.clone()))
      .send()
      .await
    {
      Ok(_) => {
        info!("updated principals for {}", &svc_id)
      }
      Err(e) => {
        error!("failed to update principals for {}: {:?}", &svc_id, e)
      }
    }
  }

  Ok(())
}
