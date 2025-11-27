use anyhow::Result;
use crossterm::event::KeyCode;

use crate::events::InputEvent;
use crate::kube_client::{DeploymentInfo, KubeClient, PodInfo, ServiceInfo};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Pods,
    Deployments,
    Services,
    Logs,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Namespace,
    Scale,
}

pub struct App {
    pub client: KubeClient,
    pub current_view: View,
    pub namespaces: Vec<String>,
    pub current_namespace: String,
    pub namespace_index: usize,
    pub pods: Vec<PodInfo>,
    pub pod_index: usize,
    pub deployments: Vec<DeploymentInfo>,
    pub deployment_index: usize,
    pub services: Vec<ServiceInfo>,
    pub service_index: usize,
    pub logs: String,
    pub error_message: Option<String>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub status_message: String,
}

impl App {
    pub async fn new() -> Result<Self> {
        let client = KubeClient::new().await?;
        let namespaces = client.list_namespaces().await?;
        let current_namespace = namespaces
            .first()
            .cloned()
            .unwrap_or_else(|| "default".to_string());

        let mut app = Self {
            client,
            current_view: View::Pods,
            namespaces,
            current_namespace: current_namespace.clone(),
            namespace_index: 0,
            pods: vec![],
            pod_index: 0,
            deployments: vec![],
            deployment_index: 0,
            services: vec![],
            service_index: 0,
            logs: String::new(),
            error_message: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            status_message: String::new(),
        };

        app.refresh_current_view().await?;
        Ok(app)
    }

    pub async fn handle_event(&mut self, event: InputEvent) -> Result<bool> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(event).await,
            InputMode::Namespace => self.handle_namespace_mode(event).await,
            InputMode::Scale => self.handle_scale_mode(event).await,
        }
    }

    async fn handle_normal_mode(&mut self, event: InputEvent) -> Result<bool> {
        match event.key_code() {
            KeyCode::Char('q') => return Ok(false),
            KeyCode::Char('1') => {
                self.current_view = View::Pods;
                self.refresh_current_view().await?;
            }
            KeyCode::Char('2') => {
                self.current_view = View::Deployments;
                self.refresh_current_view().await?;
            }
            KeyCode::Char('3') => {
                self.current_view = View::Services;
                self.refresh_current_view().await?;
            }
            KeyCode::Char('n') => {
                self.input_mode = InputMode::Namespace;
                self.input_buffer.clear();
            }
            KeyCode::Char('r') => {
                self.refresh_current_view().await?;
            }
            KeyCode::Char('d') => {
                self.delete_current_item().await?;
            }
            KeyCode::Char('l') => {
                if self.current_view == View::Pods {
                    self.view_pod_logs().await?;
                }
            }
            KeyCode::Char('s') => {
                if self.current_view == View::Deployments {
                    self.input_mode = InputMode::Scale;
                    self.input_buffer.clear();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection_down();
            }
            _ => {}
        }
        Ok(true)
    }

    async fn handle_namespace_mode(&mut self, event: InputEvent) -> Result<bool> {
        match event.key_code() {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Enter => {
                if !self.input_buffer.is_empty() {
                    self.current_namespace = self.input_buffer.clone();
                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                    self.refresh_current_view().await?;
                }
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Up => {
                if self.namespace_index > 0 {
                    self.namespace_index -= 1;
                    if let Some(ns) = self.namespaces.get(self.namespace_index) {
                        self.input_buffer = ns.clone();
                    }
                }
            }
            KeyCode::Down => {
                if self.namespace_index < self.namespaces.len().saturating_sub(1) {
                    self.namespace_index += 1;
                    if let Some(ns) = self.namespaces.get(self.namespace_index) {
                        self.input_buffer = ns.clone();
                    }
                }
            }
            _ => {}
        }
        Ok(true)
    }

    async fn handle_scale_mode(&mut self, event: InputEvent) -> Result<bool> {
        match event.key_code() {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Enter => {
                if let Ok(replicas) = self.input_buffer.parse::<i32>() {
                    if let Some(deployment) = self.deployments.get(self.deployment_index) {
                        match self
                            .client
                            .scale_deployment(&self.current_namespace, &deployment.name, replicas)
                            .await
                        {
                            Ok(_) => {
                                self.status_message =
                                    format!("Scaled {} to {} replicas", deployment.name, replicas);
                                self.refresh_current_view().await?;
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Failed to scale: {}", e));
                            }
                        }
                    }
                }
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            _ => {}
        }
        Ok(true)
    }

    fn move_selection_up(&mut self) {
        match self.current_view {
            View::Pods => {
                if self.pod_index > 0 {
                    self.pod_index -= 1;
                }
            }
            View::Deployments => {
                if self.deployment_index > 0 {
                    self.deployment_index -= 1;
                }
            }
            View::Services => {
                if self.service_index > 0 {
                    self.service_index -= 1;
                }
            }
            View::Logs => {}
        }
    }

    fn move_selection_down(&mut self) {
        match self.current_view {
            View::Pods => {
                if self.pod_index < self.pods.len().saturating_sub(1) {
                    self.pod_index += 1;
                }
            }
            View::Deployments => {
                if self.deployment_index < self.deployments.len().saturating_sub(1) {
                    self.deployment_index += 1;
                }
            }
            View::Services => {
                if self.service_index < self.services.len().saturating_sub(1) {
                    self.service_index += 1;
                }
            }
            View::Logs => {}
        }
    }

    async fn refresh_current_view(&mut self) -> Result<()> {
        self.error_message = None;
        match self.current_view {
            View::Pods => match self.client.list_pods(&self.current_namespace).await {
                Ok(pods) => {
                    self.pods = pods;
                    if self.pod_index >= self.pods.len() {
                        self.pod_index = self.pods.len().saturating_sub(1);
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to list pods: {}", e));
                }
            },
            View::Deployments => {
                match self.client.list_deployments(&self.current_namespace).await {
                    Ok(deployments) => {
                        self.deployments = deployments;
                        if self.deployment_index >= self.deployments.len() {
                            self.deployment_index = self.deployments.len().saturating_sub(1);
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to list deployments: {}", e));
                    }
                }
            }
            View::Services => match self.client.list_services(&self.current_namespace).await {
                Ok(services) => {
                    self.services = services;
                    if self.service_index >= self.services.len() {
                        self.service_index = self.services.len().saturating_sub(1);
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to list services: {}", e));
                }
            },
            View::Logs => {}
        }
        Ok(())
    }

    async fn delete_current_item(&mut self) -> Result<()> {
        match self.current_view {
            View::Pods => {
                if let Some(pod) = self.pods.get(self.pod_index) {
                    match self
                        .client
                        .delete_pod(&self.current_namespace, &pod.name)
                        .await
                    {
                        Ok(_) => {
                            self.status_message = format!("Deleted pod {}", pod.name);
                            self.refresh_current_view().await?;
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to delete pod: {}", e));
                        }
                    }
                }
            }
            View::Deployments => {
                if let Some(deployment) = self.deployments.get(self.deployment_index) {
                    match self
                        .client
                        .delete_deployment(&self.current_namespace, &deployment.name)
                        .await
                    {
                        Ok(_) => {
                            self.status_message = format!("Deleted deployment {}", deployment.name);
                            self.refresh_current_view().await?;
                        }
                        Err(e) => {
                            self.error_message =
                                Some(format!("Failed to delete deployment: {}", e));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn view_pod_logs(&mut self) -> Result<()> {
        if let Some(pod) = self.pods.get(self.pod_index) {
            match self
                .client
                .get_pod_logs(&self.current_namespace, &pod.name)
                .await
            {
                Ok(logs) => {
                    self.logs = logs;
                    self.current_view = View::Logs;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to get logs: {}", e));
                }
            }
        }
        Ok(())
    }

    pub fn get_help_text(&self) -> Vec<(&str, &str)> {
        let mut help = vec![
            ("q", "Quit"),
            ("1", "Pods"),
            ("2", "Deployments"),
            ("3", "Services"),
            ("n", "Change Namespace"),
            ("r", "Refresh"),
            ("↑/k", "Up"),
            ("↓/j", "Down"),
        ];

        match self.current_view {
            View::Pods => {
                help.push(("l", "View Logs"));
                help.push(("d", "Delete Pod"));
            }
            View::Deployments => {
                help.push(("s", "Scale"));
                help.push(("d", "Delete"));
            }
            View::Logs => {
                help.push(("Esc", "Back"));
            }
            _ => {}
        }

        help
    }
}
