# Intro

A tiny [controller](https://kubernetes.io/docs/concepts/architecture/controller/) that manages Allowed Principal entries in AWS VPC Endpoint Services, focusing on adding accounts in an Organization into the Allowed Principals list.

The name is also a pun on [our company name](https://controlant.com/).

# Usage

## Concept

- Run the controller with desired tag names
- Apply tags to AWS VPC Endpoint Services
- The controller will manage Allowed Principals based on AWS Accounts discovered from the root-account defined via the `--root-role`.
- The controller does not remove principals at this time.

## Detail

- Use `--help` to see documentation for cli arguments
- `--endpoint-key` can be used to specify different tag key
- Configure tags for AWS resources:
  - `[key]` is used to define the service being managed

## Example

- Run `ctrl-vpces --endpoint-key endpoint.controlant.com/managed`
- The controller will maintain following principals for the tagged Endpoint Services
  - All accounts under the organization

## Assume Role and AWS Account Discovery

The controller is designed with managing multiple AWS accounts in mind:

- Without any related arguments, the controller will use the [default credential provider chain](https://docs.rs/aws-config/latest/aws_config/default_provider/credentials/struct.DefaultCredentialsChain.html), and run in the configured AWS account
- Using the `--assume-role` arguments (repeatable), the controller will try to assume all the specified roles and perform changes in those accounts, these roles need to have sufficient permission, which is listed in the next section (and obviously need to allow the controller to assume such roles)
- Alternatively, the controller can also be instructed to discover all accounts in an organization, and try to assume role based on the same path and name. The `--sub-role` must include the path and name of the role (e.g. `/controllers/vpces-controller`), this option is also mutually excluse with `--assume-role`
  - By default, the controller uses the current credential to discover accounts, which requires it to be run within the root account. This is not always desirable, so the `--root-role` allows the controller to assume a role in the root account to perform discovery, which enables it to be run in other places. The "root role" needs to allow `organizations:ListAccounts` in its IAM policy

### "sub account" IAM policy (in Terraform)

Note: change the `ENDPOINT_KEY` below:

```hcl
data "aws_iam_policy_document" "perms" {
  statement {
    actions = [
      "ec2:DescribeVpcEndpointServiceConfigurations",
      "ec2:ModifyVpcEndpointServicePermissions"
    ]
    effect    = "Allow"
    resources = ["*"]

    condition {
      test     = "Null"
      variable = "aws:ResourceTag/ENDPOINT_KEY"
      values   = ["false"]
    }
  }
}
```

# Roadmap

- TODO: add metrics
- TODO: use distributed tracing
- TODO: tests
- MAYBE: add tag to define filter on principals (accounts) to add
- MAYBE: add Name tag (account-name) to principal entries for clarity in UI

# Debug

- `RUST_LOG=trace` will enable all logs including those from AWS SDK
- `RUST_LOG=warn,ctrl_vpces=debug` will enable debug+ logs from the controller (these include all read api calls and no action decisions) as well as warn+ logs from other crates
- info logs include all update decisions and write api call results
