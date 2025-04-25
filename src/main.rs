use chrono::{DateTime, Utc};
use log::{debug, error, info};
use nixv::nix_commands::nix_build::nix_build_process;
use nixv::nix_commands::nix_build_flake::*;
use nixv::nix_commands::nix_develop_flake::nix_develop_flake_process;
use nixv::nix_commands::nix_shell::nix_shell_process;
use nixv::nix_logs::helpers::log_;
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::ffi::OsStr;
use std::fs::{self, create_dir_all, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use sysinfo::*;

// Globals using atomics instead of lazy_static
static MONITOR_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Debug, Clone)]
struct KubernetesInfo {
    is_kubernetes: bool,
    pod_name: Option<String>,
    pod_namespace: Option<String>,
    node_name: Option<String>,
    container_name: Option<String>,
    container_id: Option<String>,
    container_limits: ContainerResources,
    container_requests: ContainerResources,
}

#[derive(Serialize, Debug, Clone)]
struct ContainerResources {
    cpu_cores: Option<f32>,
    memory_bytes: Option<u64>,
    memory_mb: Option<f32>,
}

#[derive(Serialize, Debug, Clone)]
struct CPUStats {
    usage_percent: f32,
    steal_percent: f32,
    iowait_percent: f32,
    load_avg_1min: Option<f64>,
    load_avg_5min: Option<f64>,
    load_avg_15min: Option<f64>,
    physical_cores: Option<usize>,
    virtual_cores: usize,
    frequency_mhz: Option<u64>,
    throttled: bool,
    cfs_throttled_periods: Option<u64>,
    cfs_throttled_time: Option<u64>,
}

#[derive(Serialize, Debug, Clone)]
struct ProcessStats {
    pid: u32,
    name: String,
    cpu_usage: f32,
    adjusted_cpu_usage: f32, // Adjusted for container limits
    memory_usage_bytes: u64,
    memory_usage_mb: f32,
    memory_usage_percent: f32,
    command_line: Option<String>,
    run_time_seconds: Option<u64>,
    priority: Option<i32>,
    threads: Option<usize>,
}

#[derive(Serialize, Debug, Clone)]
struct SystemStats {
    total_memory_bytes: u64,
    total_memory_mb: f32,
    used_memory_bytes: u64,
    used_memory_mb: f32,
    memory_usage_percent: f32,
    total_swap_bytes: u64,
    total_swap_mb: f32,
    used_swap_bytes: u64,
    used_swap_mb: f32,
    swap_usage_percent: f32,
    cpu: CPUStats,
    kubernetes: KubernetesInfo,
}

#[derive(Serialize, Debug)]
struct MonitoringOutput {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    timestamp: DateTime<Utc>,
    agent_id: String,
    node_id: String,
    pod_name: Option<String>,
    target_pid: u32,
    main_process: Option<ProcessStats>,
    child_processes: HashMap<u32, ProcessStats>,
    total_cpu_usage: f32,
    adjusted_total_cpu_usage: f32, // Adjusted for container limits
    total_memory_usage_bytes: u64,
    total_memory_usage_mb: f32,
    total_memory_usage_percent: f32,
    process_count: usize,
    system_stats: SystemStats,
}

// Generate a unique ID for this run
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}-{:x}", std::process::id(), timestamp)
}

/// Detect if we're running in a Kubernetes environment and gather pod/container information
fn detect_kubernetes() -> KubernetesInfo {
    let mut k8s_info = KubernetesInfo {
        is_kubernetes: false,
        pod_name: None,
        pod_namespace: None,
        node_name: None,
        container_name: None,
        container_id: None,
        container_limits: ContainerResources {
            cpu_cores: None,
            memory_bytes: None,
            memory_mb: None,
        },
        container_requests: ContainerResources {
            cpu_cores: None,
            memory_bytes: None,
            memory_mb: None,
        },
    };

    // Check for common Kubernetes environment markers
    if Path::new("/var/run/secrets/kubernetes.io").exists() {
        k8s_info.is_kubernetes = true;
    }

    // Check for Kubernetes specific environment variables
    if let Ok(pod_name) = env::var("KUBERNETES_POD_NAME") {
        k8s_info.is_kubernetes = true;
        k8s_info.pod_name = Some(pod_name);
    } else {
        // Try to get hostname as pod name
        if let Ok(output) = Command::new("hostname").output() {
            if output.status.success() {
                if let Ok(name) = String::from_utf8(output.stdout) {
                    k8s_info.pod_name = Some(name.trim().to_string());
                }
            }
        }
    }

    if let Ok(namespace) = env::var("KUBERNETES_POD_NAMESPACE") {
        k8s_info.pod_namespace = Some(namespace);
    } else {
        // Try to get namespace from the service account mount
        if let Ok(namespace) = fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/namespace") {
            k8s_info.pod_namespace = Some(namespace.trim().to_string());
        }
    }

    if let Ok(node_name) = env::var("KUBERNETES_NODE_NAME") {
        k8s_info.node_name = Some(node_name);
    }

    // Get container ID from cgroup
    if let Ok(cgroup_content) = fs::read_to_string("/proc/self/cgroup") {
        for line in cgroup_content.lines() {
            if line.contains("docker") || line.contains("containerd") {
                if let Some(id_part) = line.split('/').last() {
                    k8s_info.container_id = Some(id_part.trim().to_string());
                    break;
                }
            }
        }
    }

    // Get container name from env or hostname
    if let Ok(container_name) = env::var("KUBERNETES_CONTAINER_NAME") {
        k8s_info.container_name = Some(container_name);
    } else {
        k8s_info.container_name = Some("jnlp".to_string()); // Default for Jenkins agents
    }

    // Get resource limits and requests from cgroups or env vars
    // CPU limits
    if let Ok(cpu_limit) = read_cpu_limit() {
        k8s_info.container_limits.cpu_cores = Some(cpu_limit);
    } else if let Ok(limit_str) = env::var("CONTAINER_CPU_LIMIT") {
        if let Ok(limit) = limit_str.parse::<f32>() {
            k8s_info.container_limits.cpu_cores = Some(limit);
        }
    }

    // Memory limits
    if let Ok(memory_limit) = read_memory_limit() {
        k8s_info.container_limits.memory_bytes = Some(memory_limit);
        k8s_info.container_limits.memory_mb = Some(memory_limit as f32 / (1024.0 * 1024.0));
    } else if let Ok(limit_str) = env::var("CONTAINER_MEMORY_LIMIT_BYTES") {
        if let Ok(limit) = limit_str.parse::<u64>() {
            k8s_info.container_limits.memory_bytes = Some(limit);
            k8s_info.container_limits.memory_mb = Some(limit as f32 / (1024.0 * 1024.0));
        }
    }

    // CPU requests (try environment variables)
    if let Ok(request_str) = env::var("CONTAINER_CPU_REQUEST") {
        if let Ok(request) = request_str.parse::<f32>() {
            k8s_info.container_requests.cpu_cores = Some(request);
        }
    } else {
        // Jenkins pod spec shows 12 CPU request
        k8s_info.container_requests.cpu_cores = Some(12.0);
    }

    // Memory requests (try environment variables)
    if let Ok(request_str) = env::var("CONTAINER_MEMORY_REQUEST_BYTES") {
        if let Ok(request) = request_str.parse::<u64>() {
            k8s_info.container_requests.memory_bytes = Some(request);
            k8s_info.container_requests.memory_mb = Some(request as f32 / (1024.0 * 1024.0));
        }
    } else {
        // Jenkins pod spec shows 16Gi memory request
        let memory_gi: u64 = 16 * 1024 * 1024 * 1024;
        k8s_info.container_requests.memory_bytes = Some(memory_gi);
        k8s_info.container_requests.memory_mb = Some(memory_gi as f32 / (1024.0 * 1024.0));
    }

    k8s_info
}

/// Read CPU limit from cgroups
fn read_cpu_limit() -> io::Result<f32> {
    // Try CGroups v2 first
    if let Ok(quota) = fs::read_to_string("/sys/fs/cgroup/cpu.max") {
        let parts: Vec<&str> = quota.trim().split_whitespace().collect();
        if parts.len() >= 2 && parts[0] != "max" {
            if let (Ok(quota), Ok(period)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                if period > 0 {
                    return Ok(quota as f32 / period as f32);
                }
            }
        }
    }

    // Fall back to CGroups v1
    let quota_path = "/sys/fs/cgroup/cpu/cpu.cfs_quota_us";
    let period_path = "/sys/fs/cgroup/cpu/cpu.cfs_period_us";

    if Path::new(quota_path).exists() && Path::new(period_path).exists() {
        let quota = fs::read_to_string(quota_path)?.trim().parse::<i64>().unwrap_or(-1);
        if quota > 0 {
            let period = fs::read_to_string(period_path)?.trim().parse::<u64>().unwrap_or(100000);
            return Ok(quota as f32 / period as f32);
        }
    }

    // Also try the container-specific cgroup paths
    let quota_path = "/sys/fs/cgroup/cpu,cpuacct/cpu.cfs_quota_us";
    let period_path = "/sys/fs/cgroup/cpu,cpuacct/cpu.cfs_period_us";

    if Path::new(quota_path).exists() && Path::new(period_path).exists() {
        let quota = fs::read_to_string(quota_path)?.trim().parse::<i64>().unwrap_or(-1);
        if quota > 0 {
            let period = fs::read_to_string(period_path)?.trim().parse::<u64>().unwrap_or(100000);
            return Ok(quota as f32 / period as f32);
        }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "CPU limit not found"))
}

/// Read memory limit from cgroups
fn read_memory_limit() -> io::Result<u64> {
    // Try CGroups v2 first
    if let Ok(limit) = fs::read_to_string("/sys/fs/cgroup/memory.max") {
        let limit = limit.trim();
        if limit != "max" {
            if let Ok(limit) = limit.parse::<u64>() {
                return Ok(limit);
            }
        }
    }

    // Fall back to CGroups v1
    let limit_path = "/sys/fs/cgroup/memory/memory.limit_in_bytes";
    if Path::new(limit_path).exists() {
        let limit = fs::read_to_string(limit_path)?.trim().parse::<u64>().unwrap_or(u64::MAX);
        if limit < u64::MAX {
            return Ok(limit);
        }
    }

    // Also try the container-specific cgroup paths
    let limit_path = "/sys/fs/cgroup/memory/memory.limit_in_bytes";
    if Path::new(limit_path).exists() {
        let limit = fs::read_to_string(limit_path)?.trim().parse::<u64>().unwrap_or(u64::MAX);
        if limit < u64::MAX {
            return Ok(limit);
        }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "Memory limit not found"))
}

/// Read CPU throttling stats from cgroups
fn read_cpu_throttle_stats() -> (Option<u64>, Option<u64>) {
    let mut throttled_periods = None;
    let mut throttled_time = None;

    // Try CGroups v2
    if let Ok(stats) = fs::read_to_string("/sys/fs/cgroup/cpu.stat") {
        for line in stats.lines() {
            if line.starts_with("nr_throttled ") {
                if let Some(val_str) = line.split_whitespace().nth(1) {
                    throttled_periods = val_str.parse::<u64>().ok();
                }
            } else if line.starts_with("throttled_usec ") {
                if let Some(val_str) = line.split_whitespace().nth(1) {
                    throttled_time = val_str.parse::<u64>().ok();
                }
            }
        }
    }

    // Fall back to CGroups v1
    if throttled_periods.is_none() || throttled_time.is_none() {
        let periods_path = "/sys/fs/cgroup/cpu/cpu.stat";
        if Path::new(periods_path).exists() {
            if let Ok(stats) = fs::read_to_string(periods_path) {
                for line in stats.lines() {
                    if line.starts_with("nr_throttled ") {
                        if let Some(val_str) = line.split_whitespace().nth(1) {
                            throttled_periods = val_str.parse::<u64>().ok();
                        }
                    } else if line.starts_with("throttled_time ") {
                        if let Some(val_str) = line.split_whitespace().nth(1) {
                            throttled_time = val_str.parse::<u64>().ok();
                        }
                    }
                }
            }
        }
    }

    (throttled_periods, throttled_time)
}

/// Get CPU steal time percentage from /proc/stat
fn get_cpu_steal_percent() -> f32 {
    if let Ok(content) = fs::read_to_string("/proc/stat") {
        if let Some(cpu_line) = content.lines().next() {
            let parts: Vec<&str> = cpu_line.split_whitespace().collect();
            if parts.len() >= 9 {
                // CPU steal time is typically the 9th value
                if let Ok(steal) = parts[8].parse::<u64>() {
                    // Calculate total CPU time
                    let mut total: u64 = 0;
                    for i in 1..parts.len() {
                        if let Ok(val) = parts[i].parse::<u64>() {
                            total += val;
                        }
                    }
                    if total > 0 {
                        return (steal as f32 * 100.0) / total as f32;
                    }
                }
            }
        }
    }
    0.0
}

/// Get CPU IO wait time percentage from /proc/stat
fn get_cpu_iowait_percent() -> f32 {
    if let Ok(content) = fs::read_to_string("/proc/stat") {
        if let Some(cpu_line) = content.lines().next() {
            let parts: Vec<&str> = cpu_line.split_whitespace().collect();
            if parts.len() >= 6 {
                // IO wait is typically the 6th value
                if let Ok(iowait) = parts[5].parse::<u64>() {
                    // Calculate total CPU time
                    let mut total: u64 = 0;
                    for i in 1..parts.len() {
                        if let Ok(val) = parts[i].parse::<u64>() {
                            total += val;
                        }
                    }
                    if total > 0 {
                        return (iowait as f32 * 100.0) / total as f32;
                    }
                }
            }
        }
    }
    0.0
}

/// Get physical CPU cores count
fn get_physical_cpu_cores() -> Option<usize> {
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        let mut physical_ids = HashSet::new();
        
        for line in content.lines() {
            if line.starts_with("physical id") {
                if let Some(id) = line.split(':').nth(1) {
                    physical_ids.insert(id.trim());
                }
            }
        }
        
        if !physical_ids.is_empty() {
            return Some(physical_ids.len());
        }
    }
    None
}

/// Check if CPU is being throttled based on cgroup metrics
fn is_cpu_throttled(k8s_info: &KubernetesInfo) -> bool {
    // First, check cgroup throttling metrics
    let (throttled_periods, _) = read_cpu_throttle_stats();
    if let Some(periods) = throttled_periods {
        if periods > 0 {
            return true;
        }
    }
    
    // If we have CPU limits and we're using more than 90% of them, consider it potential throttling
    if let Some(cpu_limit) = k8s_info.container_limits.cpu_cores {
        let system = System::new_all();
        // Calculate overall CPU usage - this is approximate
        let global_cpu_usage = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() 
            / system.cpus().len() as f32;
        let virtual_cores = system.cpus().len() as f32;
        
        // Calculate how much CPU we're using compared to our limit
        let relative_usage = global_cpu_usage * virtual_cores / 100.0;
        if relative_usage > cpu_limit * 0.9 {
            return true;
        }
    }
    
    false
}

/// Get system CPU frequency in MHz
fn get_cpu_frequency() -> Option<u64> {
    if let Ok(content) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq") {
        if let Ok(freq_khz) = content.trim().parse::<u64>() {
            return Some(freq_khz / 1000); // Convert KHz to MHz
        }
    }
    None
}

/// Adjust CPU usage based on container limits and virtualization
fn adjust_cpu_usage(
    raw_usage: f32, 
    steal_percent: f32,
    k8s_info: &KubernetesInfo,
    system_cpu_count: usize
) -> f32 {
    // Adjust for CPU stealing/throttling first
    let mut adjusted_usage = raw_usage;
    
    if steal_percent > 0.0 {
        // If steal time is significant, adjust the reported CPU usage
        let available_cpu = 100.0 - steal_percent;
        if available_cpu > 0.0 {
            adjusted_usage = (raw_usage / available_cpu) * 100.0;
        }
    }
    
    // Adjust based on container CPU limits if available
    if let Some(cpu_limit) = k8s_info.container_limits.cpu_cores {
        // If we have a limit that's less than the total available CPUs, normalize the usage
        if cpu_limit < system_cpu_count as f32 {
            // Scale the usage to be relative to the actual limit
            // This converts from "% of all CPUs" to "% of allocated CPUs"
            adjusted_usage = (adjusted_usage * system_cpu_count as f32) / (cpu_limit * 100.0);
        }
    }
    
    adjusted_usage
}

/// Find all descendant processes recursively
fn find_all_descendant_pids(start_pid: Pid, sys: &System) -> HashSet<Pid> {
    let mut descendants = HashSet::new();
    let mut queue = VecDeque::new();

    // First level children
    for (pid, process) in sys.processes() {
        if process.parent() == Some(start_pid) {
            if descendants.insert(*pid) {
                queue.push_back(*pid);
            }
        }
    }

    // Process recursive descendants
    while let Some(current_pid) = queue.pop_front() {
        for (pid, process) in sys.processes() {
            if process.parent() == Some(current_pid) {
                if descendants.insert(*pid) {
                    queue.push_back(*pid);
                }
            }
        }
    }
    descendants
}

/// Get the base path for monitoring output
fn get_monitoring_base_path() -> PathBuf {
    // Use environment variable if set, otherwise default
    if let Ok(path) = env::var("NIXV_MONITOR_PATH") {
        PathBuf::from(path)
    } else if Path::new("/tmp/nix-monitoring").exists() || create_dir_all("/tmp/nix-monitoring").is_ok() {
        // Use /tmp for Kubernetes pods since it's usually writable
        PathBuf::from("/tmp/nix-monitoring")
    } else {
        // Fall back to current directory
        PathBuf::from("./monitoring")
    }
}

/// Get a node identifier that's stable across runs
fn get_node_id() -> String {
    // In Kubernetes, try to get the node name first
    if let Ok(node_name) = env::var("KUBERNETES_NODE_NAME") {
        return format!("k8s-node-{}", node_name);
    }
    
    // Try to get it from pod spec
    if let Ok(content) = fs::read_to_string("/etc/podinfo/nodeName") {
        return format!("k8s-node-{}", content.trim());
    }
    
    // Fall back to hostname command
    if let Ok(output) = Command::new("hostname").output() {
        if output.status.success() {
            if let Ok(hostname) = String::from_utf8(output.stdout) {
                return hostname.trim().to_string();
            }
        }
    }
    
    // Last resort - generate an ID based on PID and time
    format!("node-{}", generate_id())
}

/// Spawn a thread to monitor a process and its children
fn spawn_pid_monitor(target_pid_u32: u32, interval_ms: u64) -> thread::JoinHandle<()> {
    let monitor_interval = Duration::from_millis(interval_ms);
    let target_pid = Pid::from_u32(target_pid_u32);
    
    // Get node ID and agent ID
    let node_id = get_node_id();
    let agent_id = generate_id();
    
    // Detect Kubernetes environment
    let k8s_info = detect_kubernetes();
    let pod_name = k8s_info.pod_name.clone();
    
    // Create monitoring directory if it doesn't exist
    let base_path = get_monitoring_base_path();
    if let Err(e) = create_dir_all(&base_path) {
        error!(
            "[Monitor PID {}] Failed to create monitoring directory {}: {}",
            target_pid_u32,
            base_path.display(),
            e
        );
    }
    
    // Determine log file path
    let pod_suffix = pod_name.as_ref().map_or(String::new(), |p| format!("_{}", p));
    let file_name = format!("nix_{}_agent{}{}_pid_{}.jsonl", node_id, agent_id, pod_suffix, target_pid_u32);
    let log_file_path = base_path.join(file_name);

    debug!(
        "[Monitor PID {}] Determined log file path: {}",
        target_pid_u32,
        log_file_path.display()
    );

    // Mark monitoring as active
    MONITOR_ACTIVE.store(true, Ordering::SeqCst);

    thread::spawn(move || {
        info!(
            "[Monitor PID {}] Thread started with interval {}ms on node {}, pod {}",
            target_pid_u32, 
            interval_ms, 
            node_id,
            pod_name.as_deref().unwrap_or("unknown")
        );
        
        debug!(
            "[Monitor PID {}] Initializing sysinfo::System...",
            target_pid_u32
        );
        let mut sys = System::new_all();
        debug!(
            "[Monitor PID {}] sysinfo::System initialized.",
            target_pid_u32
        );

        // Log Kubernetes environment details
        if k8s_info.is_kubernetes {
            info!(
                "[Monitor PID {}] Kubernetes environment detected: pod={}, namespace={}, container={}, cpu_limit={:?}, memory_limit={:?}MB", 
                target_pid_u32, 
                k8s_info.pod_name.as_deref().unwrap_or("unknown"),
                k8s_info.pod_namespace.as_deref().unwrap_or("unknown"),
                k8s_info.container_name.as_deref().unwrap_or("unknown"),
                k8s_info.container_limits.cpu_cores,
                k8s_info.container_limits.memory_mb
            );
        }

        info!(
            "[Monitor PID {}] Attempting to open/create log file: {}",
            target_pid_u32,
            log_file_path.display()
        );
        let mut log_file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
        {
            Ok(file) => {
                info!(
                    "[Monitor PID {}] Successfully opened/created log file.",
                    target_pid_u32
                );
                Some(file)
            }
            Err(e) => {
                error!(
                    "[Monitor PID {}] Failed to open log file {}: {}. Monitor thread exiting.",
                    target_pid_u32,
                    log_file_path.display(),
                    e
                );
                None
            }
        };

        let mut loop_count = 0;
        while let Some(ref mut file) = log_file {
            loop_count += 1;
            if loop_count % 20 == 0 {
                debug!(
                    "[Monitor PID {}] Loop iteration {} (interval: {}ms)",
                    target_pid_u32, loop_count, interval_ms
                );
            }

            let timestamp = Utc::now();
            sys.refresh_all();

            // Get system-wide statistics
            let steal_percent = get_cpu_steal_percent();
            let iowait_percent = get_cpu_iowait_percent();
            let cpu_frequency = get_cpu_frequency();
            let physical_cores = get_physical_cpu_cores();
            let is_throttled = is_cpu_throttled(&k8s_info);
            let (throttled_periods, throttled_time) = read_cpu_throttle_stats();
            
            // For load averages, use default values since we can't easily get them
            let load_avg_1min = None; 
            let load_avg_5min = None;
            let load_avg_15min = None;
            
            // Create system stats
            let system_stats = SystemStats {
                total_memory_bytes: sys.total_memory(),
                total_memory_mb: sys.total_memory() as f32 / (1024.0 * 1024.0),
                used_memory_bytes: sys.used_memory(),
                used_memory_mb: sys.used_memory() as f32 / (1024.0 * 1024.0),
                memory_usage_percent: if sys.total_memory() > 0 {
                    (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0
                } else {
                    0.0
                },
                total_swap_bytes: sys.total_swap(),
                total_swap_mb: sys.total_swap() as f32 / (1024.0 * 1024.0),
                used_swap_bytes: sys.used_swap(),
                used_swap_mb: sys.used_swap() as f32 / (1024.0 * 1024.0),
                swap_usage_percent: if sys.total_swap() > 0 {
                    (sys.used_swap() as f32 / sys.total_swap() as f32) * 100.0
                } else {
                    0.0
                },
                cpu: CPUStats {
                    usage_percent: sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() 
                                  / sys.cpus().len() as f32,
                    steal_percent,
                    iowait_percent,
                    load_avg_1min,
                    load_avg_5min,
                    load_avg_15min,
                    physical_cores,
                    virtual_cores: sys.cpus().len(),
                    frequency_mhz: cpu_frequency,
                    throttled: is_throttled,
                    cfs_throttled_periods: throttled_periods,
                    cfs_throttled_time: throttled_time,
                },
                kubernetes: k8s_info.clone(),
            };

            let mut main_process_stats: Option<ProcessStats> = None;
            let mut child_process_stats: HashMap<u32, ProcessStats> = HashMap::new();
            let mut process_found = false;
            let mut total_cpu_usage = 0.0;
            let mut adjusted_total_cpu_usage = 0.0;
            let mut total_memory_usage_bytes = 0;

            // Collect main process stats
            if let Some(process) = sys.process(target_pid) {
                process_found = true;
                let mem_bytes = process.memory();
                let mem_percent = if sys.total_memory() > 0 {
                    (mem_bytes as f32 / sys.total_memory() as f32) * 100.0
                } else {
                    0.0
                };
                
                let raw_cpu_usage = process.cpu_usage();
                let adjusted_cpu = adjust_cpu_usage(
                    raw_cpu_usage, 
                    steal_percent, 
                    &k8s_info, 
                    sys.cpus().len()
                );
                
                // Get process name as string
                let proc_name = process.name().to_string_lossy().to_string();
                
                // Get command line as string if available
                let cmd_line = match process.cmd().len() {
                    0 => None,
                    _ => Some(process.cmd().join(OsStr::new(" ")).to_str().unwrap().to_owned())
                };
                
                // Get thread count estimate
                let thread_count = if let Some(tasks) = process.tasks() {
                    Some(tasks.len())
                } else {
                    None
                };
                
                let stats = ProcessStats {
                    pid: process.pid().as_u32(),
                    name: proc_name,
                    cpu_usage: raw_cpu_usage,
                    adjusted_cpu_usage: adjusted_cpu,
                    memory_usage_bytes: mem_bytes,
                    memory_usage_mb: mem_bytes as f32 / (1024.0 * 1024.0),
                    memory_usage_percent: mem_percent,
                    command_line: cmd_line,
                    run_time_seconds: None, // Not available in current sysinfo
                    priority: None, // Not available in current sysinfo
                    threads: thread_count,
                };
                
                total_cpu_usage += stats.cpu_usage;
                adjusted_total_cpu_usage += stats.adjusted_cpu_usage;
                total_memory_usage_bytes += stats.memory_usage_bytes;
                
                main_process_stats = Some(stats);
            } else {
                debug!(
                    "[Monitor PID {}] Target process not found this cycle.",
                    target_pid_u32
                );
            }

            // Collect child process stats recursively
            let descendant_pids = find_all_descendant_pids(target_pid, &sys);

            for child_pid in descendant_pids {
                if let Some(process) = sys.process(child_pid) {
                    let mem_bytes = process.memory();
                    let mem_percent = if sys.total_memory() > 0 {
                        (mem_bytes as f32 / sys.total_memory() as f32) * 100.0
                    } else {
                        0.0
                    };
                    
                    let raw_cpu_usage = process.cpu_usage();
                    let adjusted_cpu = adjust_cpu_usage(
                        raw_cpu_usage, 
                        steal_percent, 
                        &k8s_info, 
                        sys.cpus().len()
                    );
                    
                    // Get process name as string
                    let proc_name = process.name().to_string_lossy().to_string();
                    
                    // Get command line as string if available
                    let cmd_line = match process.cmd().len() {
                        0 => None,
                        _ => Some(process.cmd().join(OsStr::new(" ")).to_str().unwrap().to_owned())
                    };
                    
                    // Get thread count estimate
                    let thread_count = if let Some(tasks) = process.tasks() {
                        Some(tasks.len())
                    } else {
                        None
                    };
                    
                    let stats = ProcessStats {
                        pid: process.pid().as_u32(),
                        name: proc_name,
                        cpu_usage: raw_cpu_usage,
                        adjusted_cpu_usage: adjusted_cpu,
                        memory_usage_bytes: mem_bytes,
                        memory_usage_mb: mem_bytes as f32 / (1024.0 * 1024.0),
                        memory_usage_percent: mem_percent,
                        command_line: cmd_line,
                        run_time_seconds: None, // Not available in current sysinfo
                        priority: None, // Not available in current sysinfo
                        threads: thread_count,
                    };
                    
                    total_cpu_usage += stats.cpu_usage;
                    adjusted_total_cpu_usage += stats.adjusted_cpu_usage;
                    total_memory_usage_bytes += stats.memory_usage_bytes;
                    
                    child_process_stats.insert(stats.pid, stats);
                }
            }
            
            if loop_count % 20 == 0 && !child_process_stats.is_empty() {
                debug!(
                    "[Monitor PID {}] Found {} child processes. Total CPU: {:.2}% (adjusted: {:.2}%), Total Memory: {:.2} MB",
                    target_pid_u32,
                    child_process_stats.len(),
                    total_cpu_usage,
                    adjusted_total_cpu_usage,
                    total_memory_usage_bytes as f32 / (1024.0 * 1024.0)
                );
            }

            // Calculate process count and memory percentage
            let process_count = child_process_stats.len() + if main_process_stats.is_some() { 1 } else { 0 };
            let total_memory_usage_mb = total_memory_usage_bytes as f32 / (1024.0 * 1024.0);
            let total_memory_usage_percent = if sys.total_memory() > 0 {
                (total_memory_usage_bytes as f32 / sys.total_memory() as f32) * 100.0
            } else {
                0.0
            };
            
            // Create the complete monitoring output
            let main_process_clone = main_process_stats.clone();
            let output = MonitoringOutput {
                timestamp,
                agent_id: agent_id.clone(),
                node_id: node_id.clone(),
                pod_name: k8s_info.pod_name.clone(),
                target_pid: target_pid_u32,
                main_process: main_process_clone,
                child_processes: child_process_stats.clone(),
                total_cpu_usage,
                adjusted_total_cpu_usage,
                total_memory_usage_bytes,
                total_memory_usage_mb,
                total_memory_usage_percent,
                process_count,
                system_stats: system_stats,
            };

            // Write output to log file
            match serde_json::to_string(&output) {
                Ok(mut json_string) => {
                    json_string.push('\n');
                    if let Err(e) = file.write_all(json_string.as_bytes()) {
                        error!(
                            "[Monitor PID {}] Failed to write to log file: {}. Stopping monitor.",
                            target_pid_u32, e
                        );
                        break;
                    }
                }
                Err(e) => {
                    error!(
                        "[Monitor PID {}] Error serializing data to JSON: {}",
                        target_pid_u32, e
                    );
                }
            }

            // Check if target process is still running
            if !process_found {
                sys.refresh_all();
                if sys.process(target_pid).is_none() {
                    info!("[Monitor PID {}] Target process not found. Stopping monitor.", target_pid_u32);
                    break;
                }
            }

            thread::sleep(monitor_interval);
        }

        // Update monitoring status
        MONITOR_ACTIVE.store(false, Ordering::SeqCst);

        if log_file.is_none() && loop_count == 0 {
            error!(
                "[Monitor PID {}] Exited before monitoring loop due to file open failure.",
                target_pid_u32
            );
        } else {
            info!(
                "[Monitor PID {}] Monitor thread finished after {} loop iterations.",
                target_pid_u32, loop_count
            );
        }
    })
}

/// Start monitoring the process implicitly
fn ensure_monitoring_started(interval_ms: u64) -> Option<thread::JoinHandle<()>> {
    // Check if monitoring is already active
    if MONITOR_ACTIVE.load(Ordering::SeqCst) {
        debug!("Process monitoring already active");
        return None;
    }
    
    // Get current process PID
    match std::process::id() {
        pid => {
            info!("Starting implicit process monitoring for PID {}", pid);
            Some(spawn_pid_monitor(pid, interval_ms))
        }
    }
}

/// Creates a wrapper that runs a command with monitoring
fn run_with_implicit_monitoring<F, T>(
    cmd_func: F,
    args: Vec<String>,
    interval_ms: u64,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnOnce(Vec<String>) -> Result<T, Box<dyn std::error::Error>>,
{
    // Start monitoring for this process and all children
    let _monitor_handle = ensure_monitoring_started(interval_ms);
    
    // Run the actual command
    let result = cmd_func(args);
    
    // Return the command result
    result
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get monitoring interval from environment or use default
    let pid_monitor_interval_ms: u64 = env::var("PID_MONITOR_INTERVAL_MS")
        .map_err(|e| {
            eprintln!(
                "Warning: Invalid PID_MONITOR_INTERVAL_MS, using default 500ms. Error: {}",
                e
            );
            e
        })
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(500);
    
    // Set up logging
    let args: Vec<String> = env::args().collect();
    let mut log_level_map = HashMap::new();
    log_level_map.insert("error", log::LevelFilter::Error);
    log_level_map.insert("warn", log::LevelFilter::Warn);
    log_level_map.insert("info", log::LevelFilter::Info);
    log_level_map.insert("debug", log::LevelFilter::Debug);
    log_level_map.insert("trace", log::LevelFilter::Trace);
    let log_level = match env::var("RUST_LOG") {
        Ok(v) => log_level_map
            .get(v.as_str())
            .copied()
            .unwrap_or(log::LevelFilter::Info),
        Err(_) => log::LevelFilter::Info,
    };
    env_logger::builder()
        .filter_level(log_level)
        .format(|_buf, record| -> Result<(), std::io::Error> {
            Ok({
                log_(record);
            })
        })
        .init();

    // Always start monitoring for the main process
    let _monitor_handle = ensure_monitoring_started(pid_monitor_interval_ms);
    
    // Process command line arguments
    let default = &String::from("");
    match args.split_first() {
        Some((x, xs)) => {
            let command = x.split('/').last().unwrap_or(default);
            match command {
                "nixv" => {
                    let (subcommand, xargs) = xs.split_first().unwrap_or((default, &[]));
                    match subcommand.as_str() {
                        "develop" => {
                            // Run develop with implicit monitoring
                            run_with_implicit_monitoring(
                                |args| -> Result<(), Box<dyn std::error::Error>> {
                                    let _ = nix_develop_flake_process(args)?;
                                    let shell = "/bin/bash";
                                    let nix_develop_command = format!("nix develop --command {}", shell);
                                    Command::new("nix-shell")
                                        .arg("--command")
                                        .arg(&nix_develop_command)
                                        .stdin(Stdio::inherit())
                                        .stdout(Stdio::inherit())
                                        .stderr(Stdio::inherit())
                                        .status()
                                        .expect("Failed to execute 'nix develop'");
                                    Ok(())
                                },
                                xargs.to_vec().to_owned(),
                                pid_monitor_interval_ms,
                            )?;
                        }
                        "build" => {
                            // Run build with implicit monitoring
                            run_with_implicit_monitoring(
                                |args| -> Result<(), Box<dyn std::error::Error>> {
                                    nix_build_flake_process(args)?;
                                    Ok(())
                                },
                                xargs.to_vec().to_owned(),
                                pid_monitor_interval_ms,
                            )?;
                        }
                        "no-monitor" => {
                            // Special case for running without monitoring if needed
                            let (sub_subcommand, sub_xargs) = xargs.split_first().unwrap_or((default, &[]));
                            match sub_subcommand.as_str() {
                                "build" => {
                                    let _ = nix_build_flake_process(sub_xargs.to_vec().to_owned())?;
                                }
                                "develop" => {
                                    let _ = nix_develop_flake_process(sub_xargs.to_vec().to_owned())?;
                                    let shell = "/bin/bash";
                                    let nix_develop_command = format!("nix develop --command {}", shell);
                                    Command::new("nix-shell")
                                        .arg("--command")
                                        .arg(&nix_develop_command)
                                        .stdin(Stdio::inherit())
                                        .stdout(Stdio::inherit())
                                        .stderr(Stdio::inherit())
                                        .status()
                                        .expect("Failed to execute 'nix develop'");
                                }
                                _ => print_help(),
                            }
                        }
                        _ => print_help(),
                    };
                }
                "nixv-build" => {
                    // Run nixv-build with implicit monitoring
                    run_with_implicit_monitoring(
                        |args| -> Result<(), Box<dyn std::error::Error>> {
                            nix_build_process(args)?;
                            Ok(())
                        },
                        xs.to_vec().to_owned(),
                        pid_monitor_interval_ms,
                    )?;
                }
                "nixv-shell" => {
                    // Run nixv-shell with implicit monitoring
                    run_with_implicit_monitoring(
                        |args| -> Result<(), Box<dyn std::error::Error>> {
                            let _ = nix_shell_process(args)?;
                            let shell = "/bin/bash";
                            let nix_develop_command = format!("nix-shell --command {}", shell);
                            Command::new("nix-shell")
                                .arg("--command")
                                .arg(&nix_develop_command)
                                .stdin(Stdio::inherit())
                                .stdout(Stdio::inherit())
                                .stderr(Stdio::inherit())
                                .status()
                                .expect("Failed to execute 'nix-shell'");
                            Ok(())
                        },
                        xs.to_vec().to_owned(),
                        pid_monitor_interval_ms,
                    )?;
                }
                _ => print_help(),
            }
        }
        None => print_help(),
    }

    Ok(())
}

fn print_help() {
    // Use a command to get hostname
    let pod_info = if let Ok(output) = Command::new("hostname").output() {
        if output.status.success() {
            if let Ok(name) = String::from_utf8(output.stdout) {
                format!(" on pod {}", name.trim())
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    let pid_str = std::process::id().to_string();
        
    println!(
        "Nix build tool with CPU/Memory monitoring{}\n\
         Supported commands: [nixv develop, nixv build, nixv-build, nixv-shell]\n\
         (CPU & memory monitoring is automatically enabled for all commands)\n\
         Special commands: [nixv no-monitor build, nixv no-monitor develop] (runs without monitoring)\n\
         log-level can be set by ENV: RUST_LOG -> [error, warn, info, debug, trace]\n\
         PID monitor interval (ms) set by ENV: PID_MONITOR_INTERVAL_MS (default: 500)\n\
         Monitor output directory can be set by ENV: NIXV_MONITOR_PATH (default: /tmp/nix-monitoring)\n\
         Monitor output includes CPU usage, memory usage, and container/pod information\n\
         Current PID: {}",
        pod_info, pid_str
    );
}