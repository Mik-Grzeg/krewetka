output "infraname" {
	value = var.infra_name
}

output "chpassword" {
	sensitive = true
	value = random_password.ch_password.result
}

output "aksipaddress" {
	value = azurerm_kubernetes_cluster.aks.fqdn
}

output "publicip" {
	value = azurerm_public_ip.lb.ip_address
}

output "public_fqdn" {
	value = azurerm_public_ip.lb.fqdn
}

output "kubeconfig" {
	sensitive = true
	value = azurerm_kubernetes_cluster.aks.kube_config_raw
}


