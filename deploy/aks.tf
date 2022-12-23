data "azurerm_subscription" "current" {
  subscription_id = var.sub_id
}

resource "azurerm_role_assignment" "aks-sp" {
	scope = data.azurerm_subscription.current.id
	role_definition_name = "Network Contributor"
	principal_id = azurerm_kubernetes_cluster.aks.identity[0].principal_id
}

resource "azurerm_kubernetes_cluster" "aks" {
	name = "${var.infra_name}-aks"
	location = azurerm_resource_group.rg.location
	resource_group_name = azurerm_resource_group.rg.name
	dns_prefix = "${var.infra_name}aks"

	default_node_pool {
		name = "default"
    node_count = 1
    vm_size    = "Standard_D2_v2"
		vnet_subnet_id = azurerm_subnet.akssubnet.id
	}

  identity {
    type = "SystemAssigned"
  }
}

