## Provisiong infrastructure on Azure Cloud
Creating empty infrastructure is automated with terraform, which is necessary for that. In case it is not installed head to [terraform installation guide](https://developer.hashicorp.com/terraform/tutorials/aws-get-started/install-cli). 

Additionally, already created azure subscription is required. Passing subscription ID is needed to create the infrastructure.

## Required environment variables
```bash
export TF_VAR_infra_name=<name>
export TF_VAR_region=<region>
export TF_VAR_sub_id=<subscription-id>
```
whereas
* \<name\> - how the infrastructure and resource group will be named
* \<region\> - azure region, in a adequate code e.g. `norwayeast`
* \<subscription-id\> - azure subscription ID

## Prerequisites
Log into `az cli`

## Instructions
1. Initialize terraform modules
```bash
terraform init
```
2. Create infrastructure
```bash
terraform apply
```
It requires manual input of `yes` after checking resources that are planned to be creates.
3. Ensure that the previous command exited successfully


## Deleting infrastructure
There are two approaches to do that:
* It can be done via Azure Portal by deleting adequate resource groups.
* It can be done with terraform
```bash
terraform destroy
```
It requires manual input of `yes` after checking resources that are planned to be destroyed.