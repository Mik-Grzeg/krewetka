resource "azurerm_virtual_network" "vnet" {
  name                = "${var.infra_name}-vnet"
  address_space       = ["10.240.0.0/16"]
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
}
resource "azurerm_subnet" "akssubnet" {
  name                 = "aks-subnet"
  virtual_network_name = azurerm_virtual_network.vnet.name
  resource_group_name  = azurerm_resource_group.rg.name
  address_prefixes     = ["10.240.0.0/22"]
}
resource "azurerm_public_ip" "lb" {
  name                = "lb-aks-ip"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  allocation_method   = "Static"
  domain_name_label = "${var.infra_name}"
  sku = "Standard"
}

resource "azurerm_network_security_group" "nsg" {
	name = "inbound_security_rules"
	location = azurerm_resource_group.rg.location
	resource_group_name = azurerm_resource_group.rg.name

	security_rule {
		name = "allow_us"
		priority = 100
		direction = "Inbound"
		access = "Allow"
		protocol = "Tcp"
		source_port_range = "*"
		destination_port_ranges = ["80","443","9094"]

		# in case of a need to limit source addresses to specific ip ranges
		# comment the line below
		source_address_prefix = "*"

		# also uncomment the line below and insert the ip range
		# source_address_prefixes = ["<your ip address>"]

		destination_address_prefix = "*"
	}
}

resource "azurerm_subnet_network_security_group_association" "nsg_subnet_binding" {
	subnet_id = azurerm_subnet.akssubnet.id
	network_security_group_id = azurerm_network_security_group.nsg.id
}
