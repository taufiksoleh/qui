use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Namespace, Pod, Service};
use kube::{
    api::{Api, DeleteParams, ListParams, LogParams},
    Client,
};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use vt100::Parser;

#[derive(Debug, Clone, Deserialize)]
struct KubeConfig {
    #[serde(rename = "current-context")]
    current_context: String,
    contexts: Vec<ContextEntry>,
    clusters: Vec<ClusterEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ContextEntry {
    name: String,
    context: ContextDetail,
}

#[derive(Debug, Clone, Deserialize)]
struct ContextDetail {
    cluster: String,
    #[serde(default)]
    namespace: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ClusterEntry {
    name: String,
    cluster: ClusterDetail,
}

#[derive(Debug, Clone, Deserialize)]
struct ClusterDetail {
    server: String,
}

#[derive(Debug, Clone)]
pub struct ContextInfo {
    pub name: String,
    pub cluster: String,
    pub server: String,
    pub namespace: String,
    pub is_current: bool,
}

#[derive(Clone)]
pub struct KubeClient {
    client: Client,
}

impl KubeClient {
    pub async fn new() -> Result<Self> {
        let client = Client::try_default().await?;
        Ok(Self { client })
    }

    fn get_kubeconfig_path() -> PathBuf {
        if let Ok(path) = std::env::var("KUBECONFIG") {
            PathBuf::from(path)
        } else {
            let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.push(".kube");
            home.push("config");
            home
        }
    }

    pub fn list_contexts() -> Result<Vec<ContextInfo>> {
        let config_path = Self::get_kubeconfig_path();
        let config_content = fs::read_to_string(&config_path)?;
        let kubeconfig: KubeConfig = serde_yaml::from_str(&config_content)?;

        let current_context = kubeconfig.current_context.clone();

        let mut contexts = Vec::new();
        for ctx in kubeconfig.contexts {
            let server = kubeconfig
                .clusters
                .iter()
                .find(|c| c.name == ctx.context.cluster)
                .map(|c| c.cluster.server.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            contexts.push(ContextInfo {
                name: ctx.name.clone(),
                cluster: ctx.context.cluster,
                server,
                namespace: if ctx.context.namespace.is_empty() {
                    "default".to_string()
                } else {
                    ctx.context.namespace
                },
                is_current: ctx.name == current_context,
            });
        }

        Ok(contexts)
    }

    pub fn get_current_context() -> Result<String> {
        let config_path = Self::get_kubeconfig_path();
        let config_content = fs::read_to_string(&config_path)?;
        let kubeconfig: KubeConfig = serde_yaml::from_str(&config_content)?;
        Ok(kubeconfig.current_context)
    }

    pub fn switch_context(context_name: &str) -> Result<()> {
        let output = Command::new("kubectl")
            .arg("config")
            .arg("use-context")
            .arg(context_name)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to switch context: {}", error_msg);
        }

        Ok(())
    }
}

pub struct TerminalSession {
    parser: Parser,
    writer: Box<dyn Write + Send>,
    #[allow(dead_code)]
    child: Box<dyn portable_pty::Child + Send + Sync>,
    rx: Receiver<Vec<u8>>,
    _reader_thread: Option<thread::JoinHandle<()>>,
}

impl TerminalSession {
    pub fn new(namespace: &str, pod_name: &str) -> Result<Self> {
        let pty_system = NativePtySystem::default();

        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new("kubectl");
        cmd.arg("exec");
        cmd.arg("-it");
        cmd.arg("-n");
        cmd.arg(namespace);
        cmd.arg(pod_name);
        cmd.arg("--");
        cmd.arg("/bin/sh");

        let child = pair.slave.spawn_command(cmd)?;

        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        // Create a channel for reading PTY output in a background thread
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();

        // Spawn a thread to read from the PTY
        let reader_thread = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            parser: Parser::new(24, 80, 1000),
            writer,
            child,
            rx,
            _reader_thread: Some(reader_thread),
        })
    }

    pub fn send_input(&mut self, event: &crate::events::InputEvent) -> Result<()> {
        let mut buf = Vec::new();

        match event.key_code() {
            KeyCode::Char(c) => {
                if event.modifiers().contains(KeyModifiers::CONTROL) {
                    // Handle Ctrl+C, Ctrl+D, etc.
                    let ctrl_char = match c {
                        'c' => Some(3u8),  // Ctrl+C
                        'd' => Some(4u8),  // Ctrl+D
                        'z' => Some(26u8), // Ctrl+Z
                        'l' => Some(12u8), // Ctrl+L
                        _ => None,
                    };
                    if let Some(ctrl_byte) = ctrl_char {
                        buf.push(ctrl_byte);
                    }
                } else {
                    buf.extend_from_slice(c.to_string().as_bytes());
                }
            }
            KeyCode::Enter => buf.extend_from_slice(b"\r"),
            KeyCode::Backspace => buf.push(127),
            KeyCode::Tab => buf.push(b'\t'),
            KeyCode::Up => buf.extend_from_slice(b"\x1b[A"),
            KeyCode::Down => buf.extend_from_slice(b"\x1b[B"),
            KeyCode::Right => buf.extend_from_slice(b"\x1b[C"),
            KeyCode::Left => buf.extend_from_slice(b"\x1b[D"),
            _ => {}
        }

        if !buf.is_empty() {
            self.writer.write_all(&buf)?;
            self.writer.flush()?;
        }

        // Process any pending output from the channel
        self.process_output();

        Ok(())
    }

    fn process_output(&mut self) {
        // Process all available data from the channel without blocking
        while let Ok(data) = self.rx.try_recv() {
            self.parser.process(&data);
        }
    }

    pub fn get_screen(&mut self) -> Vec<String> {
        // Process any pending output
        self.process_output();

        let screen = self.parser.screen();

        // Get the entire screen contents as a string and split by lines
        let contents = screen.contents();
        contents.lines().map(|s| s.to_string()).collect()
    }

    pub fn close(&mut self) -> Result<()> {
        // Send Ctrl+D to close the shell gracefully
        self.writer.write_all(&[4])?;
        self.writer.flush()?;
        Ok(())
    }
}

impl KubeClient {
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
