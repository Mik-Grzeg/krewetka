# Krewetka-platform
Helm chart responsible for deployment of crucial services, network configurations, config maps, etc. It is an umbrella chart, which means it a single chart with multiple subcharts, each of which function as a building block. 

## Subcharts
* kafka
* clickhouse
* ingress-nginx
* zookeeper
* kube-prometheus-stack
* krewetka - helm chart with reporting, reporting-ui, processor and classifier deployments

## Instructions
1. Export public IP address of the Azure Load Balancer provisioned in `../deploy` terraform part
```bash
export PUBLIC_IP=$(terraform -chdir=../deploy output --raw publicip)
```
2. Download dependecy charts with command below, it should create few `.tgz` archives in `./charts` directory
```bash
helm dependency update
```

3. Install helm chart
It requires passing public IP address of the Azure Load Balancer created in `../deploy` terraform part.
```bash
helm install <release-name> . -f values.yaml --namespace krewetka --create-namespace --set ingress-nginx.controller.service.loadBalancerIP=$PUBLIC_IP --set kafka.externalAccess.service.loadBalancerIPs={$PUBLIC_IP}
```

4. Inspect k8s cluster, check whether all pods and services are running(in a green state).
