use anyhow::Result;
use crossterm::event::KeyCode;

use crate::events::InputEvent;
use crate::kube_client::{ContextInfo, DeploymentInfo, KubeClient, PodInfo, ServiceInfo};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Pods,
    Deployments,
    Services,
    Logs,
    Clusters,
    Namespaces,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Scale,
}

pub struct App {
    pub client: KubeClient,
    pub current_view: View,
    pub namespaces: Vec<String>,
    pub current_namespace: String,
    pub namespace_index: usize,
    pub contexts: Vec<ContextInfo>,
    pub context_index: usize,
    pub current_context: String,
    pub pods: Vec<PodInfo>,
    pub pod_index: usize,
    pub deployments: Vec<DeploymentInfo>,
    pub deployment_index: usize,
    pub services: Vec<ServiceInfo>,
    pub service_index: usize,
    pub logs: String,
    pub logs_scroll: usize,
    pub logs_follow: bool,
    pub logs_pod_name: Option<String>,
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

        let contexts = KubeClient::list_contexts().unwrap_or_default();
        let current_context = KubeClient::get_current_context().unwrap_or_default();

        let mut app = Self {
            client,
            current_view: View::Pods,
            namespaces,
            current_namespace: current_namespace.clone(),
            namespace_index: 0,
            contexts,
            context_index: 0,
            current_context,
            pods: vec![],
            pod_index: 0,
            deployments: vec![],
            deployment_index: 0,
            services: vec![],
            service_index: 0,
            logs: String::new(),
            logs_scroll: 0,
            logs_follow: false,
            logs_pod_name: None,
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
            KeyCode::Char('4') => {
                self.current_view = View::Clusters;
                self.refresh_current_view().await?;
            }
            KeyCode::Char('5') | KeyCode::Char('n') => {
                self.current_view = View::Namespaces;
                self.refresh_current_view().await?;
            }
            KeyCode::Char('?') | KeyCode::Char('h') => {
                self.current_view = View::Help;
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
            KeyCode::Char('f') => {
                if self.current_view == View::Logs {
                    self.toggle_log_follow();
                }
            }
            KeyCode::Char('e') => {
                if self.current_view == View::Pods {
                    self.exec_into_pod().await?;
                }
            }
            KeyCode::Char('s') => {
                if self.current_view == View::Deployments {
                    self.input_mode = InputMode::Scale;
                    self.input_buffer.clear();
                }
            }
            KeyCode::Enter => {
                match self.current_view {
                    View::Clusters => self.switch_to_selected_context().await?,
                    View::Namespaces => self.switch_to_selected_namespace().await?,
                    _ => {}
                }
            }
            KeyCode::Esc => {
                if self.current_view == View::Help {
                    self.current_view = View::Pods;
                } else if self.current_view == View::Logs {
                    self.logs_follow = false;
                    self.current_view = View::Pods;
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
            View::Clusters => {
                if self.context_index > 0 {
                    self.context_index -= 1;
                }
            }
            View::Namespaces => {
                if self.namespace_index > 0 {
                    self.namespace_index -= 1;
                }
            }
            View::Logs => {
                if self.logs_scroll > 0 {
                    self.logs_scroll -= 1;
                    self.logs_follow = false; // Disable follow when manually scrolling
                }
            }
            View::Help => {}
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
            View::Clusters => {
                if self.context_index < self.contexts.len().saturating_sub(1) {
                    self.context_index += 1;
                }
            }
            View::Namespaces => {
                if self.namespace_index < self.namespaces.len().saturating_sub(1) {
                    self.namespace_index += 1;
                }
            }
            View::Logs => {
                let log_lines = self.logs.lines().count();
                if self.logs_scroll < log_lines.saturating_sub(1) {
                    self.logs_scroll += 1;
                    self.logs_follow = false; // Disable follow when manually scrolling
                }
            }
            View::Help => {}
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
            View::Clusters => match KubeClient::list_contexts() {
                Ok(contexts) => {
                    self.contexts = contexts;
                    if self.context_index >= self.contexts.len() {
                        self.context_index = self.contexts.len().saturating_sub(1);
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to list contexts: {}", e));
                }
            },
            View::Namespaces => {
                // Namespaces are already loaded, just ensure index is valid
                if self.namespace_index >= self.namespaces.len() {
                    self.namespace_index = self.namespaces.len().saturating_sub(1);
                }
            }
            View::Logs | View::Help => {}
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
                    self.logs_scroll = 0; // Reset scroll position
                    self.logs_pod_name = Some(pod.name.clone()); // Store pod name for follow mode
                    self.current_view = View::Logs;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to get logs: {}", e));
                }
            }
        }
        Ok(())
    }

    fn toggle_log_follow(&mut self) {
        self.logs_follow = !self.logs_follow;
        if self.logs_follow {
            // Scroll to bottom when enabling follow mode
            let log_lines = self.logs.lines().count();
            self.logs_scroll = log_lines.saturating_sub(1);
            self.status_message = "Log follow mode enabled (press 'f' to disable)".to_string();
        } else {
            self.status_message = "Log follow mode disabled".to_string();
        }
    }

    pub async fn refresh_logs(&mut self) -> Result<()> {
        if self.logs_follow && self.current_view == View::Logs {
            if let Some(pod_name) = &self.logs_pod_name.clone() {
                match self
                    .client
                    .get_pod_logs(&self.current_namespace, pod_name)
                    .await
                {
                    Ok(logs) => {
                        self.logs = logs;
                        // Auto-scroll to bottom in follow mode
                        let log_lines = self.logs.lines().count();
                        self.logs_scroll = log_lines.saturating_sub(1);
                    }
                    Err(_) => {
                        // Silently ignore errors in background refresh
                    }
                }
            }
        }
        Ok(())
    }

    async fn switch_to_selected_context(&mut self) -> Result<()> {
        if let Some(context) = self.contexts.get(self.context_index) {
            match KubeClient::switch_context(&context.name) {
                Ok(_) => {
                    self.status_message = format!("Switched to context: {}", context.name);
                    self.current_context = context.name.clone();

                    // Reinitialize client with new context
                    match KubeClient::new().await {
                        Ok(new_client) => {
                            self.client = new_client;

                            // Refresh namespaces and set to the context's default namespace
                            match self.client.list_namespaces().await {
                                Ok(namespaces) => {
                                    self.namespaces = namespaces;
                                    self.current_namespace = if !context.namespace.is_empty() {
                                        context.namespace.clone()
                                    } else {
                                        "default".to_string()
                                    };
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("Failed to list namespaces: {}", e));
                                }
                            }

                            // Refresh context list to update current indicator
                            self.refresh_current_view().await?;
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to initialize new client: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to switch context: {}", e));
                }
            }
        }
        Ok(())
    }

    async fn switch_to_selected_namespace(&mut self) -> Result<()> {
        if let Some(namespace) = self.namespaces.get(self.namespace_index) {
            self.current_namespace = namespace.clone();
            self.status_message = format!("Switched to namespace: {}", namespace);
            self.current_view = View::Pods;
            self.refresh_current_view().await?;
        }
        Ok(())
    }

    async fn exec_into_pod(&mut self) -> Result<()> {
        if let Some(pod) = self.pods.get(self.pod_index) {
            match KubeClient::exec_into_pod(&self.current_namespace, &pod.name) {
                Ok(_) => {
                    self.status_message = format!("Exited shell for pod: {}", pod.name);
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to exec into pod: {}", e));
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
            ("4", "Clusters"),
            ("5/n", "Namespaces"),
            ("?/h", "Help"),
            ("r", "Refresh"),
            ("↑/k", "Up"),
            ("↓/j", "Down"),
        ];

        match self.current_view {
            View::Pods => {
                help.push(("l", "Logs"));
                help.push(("e", "Exec"));
                help.push(("d", "Delete"));
            }
            View::Deployments => {
                help.push(("s", "Scale"));
                help.push(("d", "Delete"));
            }
            View::Clusters => {
                help.push(("Enter", "Switch"));
            }
            View::Namespaces => {
                help.push(("Enter", "Switch"));
            }
            View::Logs => {
                help.push(("↑/↓", "Scroll"));
                help.push(("f", "Follow"));
                help.push(("Esc", "Back"));
            }
            View::Help => {
                help.push(("Esc", "Close"));
            }
            _ => {}
        }

        help
    }
}
