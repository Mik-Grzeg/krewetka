# Configure the Azure provider
terraform {
  required_providers {
    azurerm = {
      version = ">= 3.31.0"
    }
  }

  required_version = ">= 1.1.0"
}

provider "azurerm" {
  subscription_id = var.sub_id
  features {}
}

resource "azurerm_resource_group" "rg" {
  name     = var.infra_name
  location = var.region
}

