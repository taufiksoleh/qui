use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use std::sync::{Arc, Mutex};

use crate::events::InputEvent;
use crate::kube_client::{ContextInfo, DeploymentInfo, KubeClient, PodInfo, ServiceInfo, TerminalSession};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Pods,
    Deployments,
    Services,
    Logs,
    Clusters,
    Namespaces,
    Help,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Scale,
    TerminalChoice,
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
    pub terminal_session: Option<Arc<Mutex<TerminalSession>>>,
    pub terminal_pod_name: Option<String>,
    pub terminal_scroll: usize,
    pub terminal_choice_selection: usize,
}

impl App {
    pub async fn new() -> Result<Self> {
        // Try to get contexts first (this works even without a connection)
        let contexts = KubeClient::list_contexts().unwrap_or_default();
        let current_context = KubeClient::get_current_context().unwrap_or_default();

        // Check if we have any contexts configured
        if contexts.is_empty() {
            anyhow::bail!("No Kubernetes contexts found. Please configure kubectl first.");
        }

        // Check if current context is set
        if current_context.is_empty() {
            anyhow::bail!("No current context set. Please run 'kubectl config use-context <context-name>' or use kubectx.");
        }

        // Try to create client and connect
        let (client, namespaces, initial_view, error_message) = match KubeClient::new().await {
            Ok(client) => {
                // Try to list namespaces to verify connection
                match client.list_namespaces().await {
                    Ok(namespaces) => {
                        if namespaces.is_empty() {
                            (client, vec!["default".to_string()], View::Pods, None)
                        } else {
                            (client, namespaces, View::Pods, None)
                        }
                    }
                    Err(e) => {
                        // Connection failed, start on Clusters view
                        let error_msg = format!(
                            "Failed to connect to cluster '{}': {}. Please switch to a valid context (Press 4 for Clusters view).",
                            current_context, e
                        );
                        (client, vec!["default".to_string()], View::Clusters, Some(error_msg))
                    }
                }
            }
            Err(e) => {
                // Client creation failed, this is usually a config issue
                anyhow::bail!(
                    "Failed to initialize Kubernetes client: {}. Please check your kubeconfig and ensure a valid context is selected.",
                    e
                );
            }
        };

        let current_namespace = namespaces
            .first()
            .cloned()
            .unwrap_or_else(|| "default".to_string());

        let mut app = Self {
            client,
            current_view: initial_view,
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
            error_message,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            status_message: String::new(),
            terminal_session: None,
            terminal_pod_name: None,
            terminal_scroll: 0,
            terminal_choice_selection: 0,
        };

        // Only try to refresh if we don't have an error
        if app.error_message.is_none() {
            let _ = app.refresh_current_view().await;
        }

        Ok(app)
    }

    pub async fn handle_event(&mut self, event: InputEvent) -> Result<bool> {
        // Handle terminal view with special input handling
        if self.current_view == View::Terminal {
            return self.handle_terminal_mode(event).await;
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(event).await,
            InputMode::Scale => self.handle_scale_mode(event).await,
            InputMode::TerminalChoice => self.handle_terminal_choice_mode(event).await,
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
            KeyCode::Enter => match self.current_view {
                View::Clusters => self.switch_to_selected_context().await?,
                View::Namespaces => self.switch_to_selected_namespace().await?,
                _ => {}
            },
            KeyCode::Esc => {
                if self.current_view == View::Help {
                    self.current_view = View::Pods;
                } else if self.current_view == View::Logs {
                    self.logs_follow = false;
                    self.current_view = View::Pods;
                } else if self.current_view == View::Terminal {
                    self.close_terminal();
                    self.current_view = View::Pods;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection_down();
            }
            KeyCode::Left => {
                self.navigate_tab_left().await?;
            }
            KeyCode::Right => {
                self.navigate_tab_right().await?;
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

    async fn handle_terminal_choice_mode(&mut self, event: InputEvent) -> Result<bool> {
        match event.key_code() {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.status_message.clear();
            }
            KeyCode::Char('1') => {
                // User chose embedded terminal
                self.input_mode = InputMode::Normal;
                self.open_embedded_terminal().await?;
            }
            KeyCode::Char('2') => {
                // User chose native terminal tab
                self.input_mode = InputMode::Normal;
                self.open_native_terminal().await?;
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                if self.terminal_choice_selection == 0 {
                    self.open_embedded_terminal().await?;
                } else {
                    self.open_native_terminal().await?;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.terminal_choice_selection > 0 {
                    self.terminal_choice_selection -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.terminal_choice_selection < 1 {
                    self.terminal_choice_selection += 1;
                }
            }
            _ => {}
        }
        Ok(true)
    }

    async fn open_embedded_terminal(&mut self) -> Result<()> {
        if let Some(pod) = self.pods.get(self.pod_index) {
            self.status_message = format!("Connecting to pod: {}...", pod.name);

            let namespace = self.current_namespace.clone();
            let pod_name = pod.name.clone();

            // Spawn terminal creation in a blocking task to avoid blocking the UI
            // Try bash first (better for Ruby/Rails), fall back to sh if it fails
            let result = tokio::task::spawn_blocking(move || {
                // Try bash first
                match TerminalSession::new_with_shell(&namespace, &pod_name, Some("/bin/bash")) {
                    Ok(session) => Ok(session),
                    Err(_) => {
                        // Fall back to sh
                        TerminalSession::new_with_shell(&namespace, &pod_name, Some("/bin/sh"))
                    }
                }
            }).await;

            match result {
                Ok(Ok(session)) => {
                    self.terminal_session = Some(Arc::new(Mutex::new(session)));
                    self.terminal_pod_name = Some(pod.name.clone());
                    self.current_view = View::Terminal;
                    self.status_message = format!("Connected to pod: {} | Press Esc to exit", pod.name);
                }
                Ok(Err(e)) => {
                    self.error_message = Some(format!("Failed to exec into pod: {}. Make sure kubectl is installed and the pod has /bin/bash or /bin/sh", e));
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to spawn terminal task: {}", e));
                }
            }
        }
        Ok(())
    }

    async fn open_native_terminal(&mut self) -> Result<()> {
        if let Some(pod) = self.pods.get(self.pod_index) {
            let namespace = self.current_namespace.clone();
            let pod_name = pod.name.clone();

            // Open a new terminal tab
            match KubeClient::open_pod_terminal(&namespace, &pod_name) {
                Ok(_) => {
                    self.status_message = format!(
                        "Opened terminal tab for pod: {} | You can now run 'irb', 'rails c', or any interactive command",
                        pod.name
                    );
                }
                Err(e) => {
                    self.error_message = Some(format!(
                        "Failed to open terminal tab: {}. Falling back to manual command...", e
                    ));
                    // Show the manual command as a fallback
                    self.status_message = format!(
                        "Run this command in your terminal: kubectl exec -it -n {} {} -- /bin/bash",
                        namespace, pod_name
                    );
                }
            }
        }
        Ok(())
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
            View::Help | View::Terminal => {}
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
            View::Help | View::Terminal => {}
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
            View::Logs | View::Help | View::Terminal => {}
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
            // Clear any previous errors
            self.error_message = None;
            self.status_message = format!("Switching to context: {}...", context.name);

            match KubeClient::switch_context(&context.name) {
                Ok(_) => {
                    self.current_context = context.name.clone();

                    // Reinitialize client with new context
                    match KubeClient::new().await {
                        Ok(new_client) => {
                            self.client = new_client;

                            // Try to verify connection by listing namespaces
                            match self.client.list_namespaces().await {
                                Ok(namespaces) => {
                                    self.namespaces = namespaces;
                                    self.current_namespace = if !context.namespace.is_empty() {
                                        context.namespace.clone()
                                    } else {
                                        self.namespaces
                                            .first()
                                            .cloned()
                                            .unwrap_or_else(|| "default".to_string())
                                    };

                                    // Success! Clear any errors and show success message
                                    self.error_message = None;
                                    self.status_message = format!(
                                        "Successfully connected to context: {} (namespace: {})",
                                        context.name, self.current_namespace
                                    );

                                    // Switch to Pods view and refresh
                                    self.current_view = View::Pods;
                                    self.refresh_current_view().await?;
                                }
                                Err(e) => {
                                    self.error_message = Some(format!(
                                        "Switched to '{}' but failed to connect: {}. The cluster may be down or unreachable.",
                                        context.name, e
                                    ));
                                    self.namespaces = vec!["default".to_string()];
                                    self.current_namespace = "default".to_string();
                                }
                            }

                            // Refresh context list to update current indicator
                            self.refresh_current_view().await?;
                        }
                        Err(e) => {
                            self.error_message = Some(format!(
                                "Switched to '{}' but failed to initialize client: {}. Check your kubeconfig.",
                                context.name, e
                            ));
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
        if self.pods.get(self.pod_index).is_some() {
            // Show terminal choice menu
            self.input_mode = InputMode::TerminalChoice;
            self.terminal_choice_selection = 0;
            self.status_message = "Choose terminal type: [1] Embedded Terminal  [2] Native Terminal Tab  [Esc] Cancel".to_string();
        }
        Ok(())
    }

    async fn handle_terminal_mode(&mut self, event: InputEvent) -> Result<bool> {
        // Handle Ctrl+D to exit terminal
        if let KeyCode::Char('d') = event.key_code() {
            if event.modifiers().contains(KeyModifiers::CONTROL) {
                self.close_terminal();
                self.current_view = View::Pods;
                return Ok(true);
            }
        }

        // Handle Esc to exit terminal
        if let KeyCode::Esc = event.key_code() {
            self.close_terminal();
            self.current_view = View::Pods;
            return Ok(true);
        }

        // Handle Page Up/Down for scrolling (don't send to terminal)
        match event.key_code() {
            KeyCode::PageUp => {
                if self.terminal_scroll > 0 {
                    self.terminal_scroll = self.terminal_scroll.saturating_sub(10);
                }
                return Ok(true);
            }
            KeyCode::PageDown => {
                self.terminal_scroll = self.terminal_scroll.saturating_add(10);
                return Ok(true);
            }
            _ => {}
        }

        // Forward all other input to the terminal
        if let Some(session) = &self.terminal_session {
            if let Ok(mut session) = session.lock() {
                session.send_input(&event)?;
            }
        }

        // Reset scroll when user types
        self.terminal_scroll = 0;

        Ok(true)
    }

    fn close_terminal(&mut self) {
        if let Some(session) = &self.terminal_session {
            if let Ok(mut session) = session.lock() {
                let _ = session.close();
            }
        }
        self.terminal_session = None;
        self.terminal_pod_name = None;
        self.terminal_scroll = 0;
    }

    pub fn get_terminal_screen(&self) -> Option<Vec<String>> {
        if let Some(session) = &self.terminal_session {
            if let Ok(mut session) = session.lock() {
                return Some(session.get_screen());
            }
        }
        None
    }

    pub fn refresh_terminal(&mut self) {
        // This is called periodically to ensure terminal output is displayed
        // The actual work is done in get_terminal_screen()
    }

    async fn navigate_tab_left(&mut self) -> Result<()> {
        let tabs = [
            View::Pods,
            View::Deployments,
            View::Services,
            View::Clusters,
            View::Namespaces,
            View::Help,
        ];

        if let Some(current_index) = tabs.iter().position(|&v| v == self.current_view) {
            let new_index = if current_index == 0 {
                tabs.len() - 1
            } else {
                current_index - 1
            };
            self.current_view = tabs[new_index];
            self.refresh_current_view().await?;
        }

        Ok(())
    }

    async fn navigate_tab_right(&mut self) -> Result<()> {
        let tabs = [
            View::Pods,
            View::Deployments,
            View::Services,
            View::Clusters,
            View::Namespaces,
            View::Help,
        ];

        if let Some(current_index) = tabs.iter().position(|&v| v == self.current_view) {
            let new_index = if current_index == tabs.len() - 1 {
                0
            } else {
                current_index + 1
            };
            self.current_view = tabs[new_index];
            self.refresh_current_view().await?;
        }

        Ok(())
    }

    pub fn get_help_text(&self) -> Vec<(&str, &str)> {
        let mut help = vec![
            ("q", "Quit"),
            ("←/→", "Switch Tab"),
            ("1-5", "Jump to Tab"),
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
