use anyhow::Result;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Namespace, Pod, Service};
use kube::{
    api::{Api, DeleteParams, ListParams, LogParams},
    Client,
};

#[derive(Clone)]
pub struct KubeClient {
    client: Client,
}

impl KubeClient {
    pub async fn new() -> Result<Self> {
        let client = Client::try_default().await?;
        Ok(Self { client })
    }

    pub async fn list_namespaces(&self) -> Result<Vec<String>> {
        let api: Api<Namespace> = Api::all(self.client.clone());
        let namespaces = api.list(&ListParams::default()).await?;

        Ok(namespaces
            .items
            .iter()
            .filter_map(|ns| ns.metadata.name.clone())
            .collect())
    }

    pub async fn list_pods(&self, namespace: &str) -> Result<Vec<PodInfo>> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let pods = api.list(&ListParams::default()).await?;

        Ok(pods.items.iter().map(PodInfo::from_pod).collect())
    }

    pub async fn delete_pod(&self, namespace: &str, name: &str) -> Result<()> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }

    pub async fn get_pod_logs(&self, namespace: &str, name: &str) -> Result<String> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let log_params = LogParams {
            tail_lines: Some(100),
            ..Default::default()
        };

        let logs = api.logs(name, &log_params).await?;
        Ok(logs)
    }

    pub async fn list_deployments(&self, namespace: &str) -> Result<Vec<DeploymentInfo>> {
        let api: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        let deployments = api.list(&ListParams::default()).await?;

        Ok(deployments
            .items
            .iter()
            .map(DeploymentInfo::from_deployment)
            .collect())
    }

    pub async fn delete_deployment(&self, namespace: &str, name: &str) -> Result<()> {
        let api: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }

    pub async fn scale_deployment(&self, namespace: &str, name: &str, replicas: i32) -> Result<()> {
        let api: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        let mut deployment = api.get(name).await?;

        if let Some(spec) = &mut deployment.spec {
            spec.replicas = Some(replicas);
        }

        api.replace(name, &Default::default(), &deployment).await?;
        Ok(())
    }

    pub async fn list_services(&self, namespace: &str) -> Result<Vec<ServiceInfo>> {
        let api: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        let services = api.list(&ListParams::default()).await?;

        Ok(services
            .items
            .iter()
            .map(ServiceInfo::from_service)
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct PodInfo {
    pub name: String,
    pub _namespace: String,
    pub status: String,
    pub ready: String,
    pub restarts: i32,
    pub age: String,
}

impl PodInfo {
    fn from_pod(pod: &Pod) -> Self {
        let name = pod.metadata.name.clone().unwrap_or_default();
        let namespace = pod.metadata.namespace.clone().unwrap_or_default();

        let status = pod
            .status
            .as_ref()
            .and_then(|s| s.phase.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let (ready_count, total_count) = pod
            .status
            .as_ref()
            .and_then(|s| s.container_statuses.as_ref())
            .map(|cs| {
                let ready = cs.iter().filter(|c| c.ready).count();
                (ready, cs.len())
            })
            .unwrap_or((0, 0));

        let ready = format!("{}/{}", ready_count, total_count);

        let restarts = pod
            .status
            .as_ref()
            .and_then(|s| s.container_statuses.as_ref())
            .map(|cs| cs.iter().map(|c| c.restart_count).sum())
            .unwrap_or(0);

        let age = pod
            .metadata
            .creation_timestamp
            .as_ref()
            .map(|t| format_age(&t.0))
            .unwrap_or_else(|| "Unknown".to_string());

        Self {
            name,
            _namespace: namespace,
            status,
            ready,
            restarts,
            age,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeploymentInfo {
    pub name: String,
    pub _namespace: String,
    pub ready: String,
    pub up_to_date: i32,
    pub available: i32,
    pub age: String,
}

impl DeploymentInfo {
    fn from_deployment(dep: &Deployment) -> Self {
        let name = dep.metadata.name.clone().unwrap_or_default();
        let namespace = dep.metadata.namespace.clone().unwrap_or_default();

        let desired = dep.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0);
        let ready = dep
            .status
            .as_ref()
            .and_then(|s| s.ready_replicas)
            .unwrap_or(0);
        let ready_str = format!("{}/{}", ready, desired);

        let up_to_date = dep
            .status
            .as_ref()
            .and_then(|s| s.updated_replicas)
            .unwrap_or(0);
        let available = dep
            .status
            .as_ref()
            .and_then(|s| s.available_replicas)
            .unwrap_or(0);

        let age = dep
            .metadata
            .creation_timestamp
            .as_ref()
            .map(|t| format_age(&t.0))
            .unwrap_or_else(|| "Unknown".to_string());

        Self {
            name,
            _namespace: namespace,
            ready: ready_str,
            up_to_date,
            available,
            age,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub _namespace: String,
    pub service_type: String,
    pub cluster_ip: String,
    pub ports: String,
    pub age: String,
}

impl ServiceInfo {
    fn from_service(svc: &Service) -> Self {
        let name = svc.metadata.name.clone().unwrap_or_default();
        let namespace = svc.metadata.namespace.clone().unwrap_or_default();

        let service_type = svc
            .spec
            .as_ref()
            .and_then(|s| s.type_.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let cluster_ip = svc
            .spec
            .as_ref()
            .and_then(|s| s.cluster_ip.clone())
            .unwrap_or_else(|| "None".to_string());

        let ports = svc
            .spec
            .as_ref()
            .and_then(|s| s.ports.as_ref())
            .map(|ports| {
                ports
                    .iter()
                    .map(|p| {
                        format!(
                            "{}/{}",
                            p.port,
                            p.protocol.as_ref().unwrap_or(&"TCP".to_string())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_else(|| "None".to_string());

        let age = svc
            .metadata
            .creation_timestamp
            .as_ref()
            .map(|t| format_age(&t.0))
            .unwrap_or_else(|| "Unknown".to_string());

        Self {
            name,
            _namespace: namespace,
            service_type,
            cluster_ip,
            ports,
            age,
        }
    }
}

fn format_age(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(*timestamp);

    let days = duration.num_days();
    let hours = duration.num_hours();
    let minutes = duration.num_minutes();

    if days > 0 {
        format!("{}d", days)
    } else if hours > 0 {
        format!("{}h", hours)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", duration.num_seconds())
    }
}
