use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use crate::service::{ServiceError, ServiceManager};

/// Linux 服务管理器，使用 systemd 管理服务
pub struct LinuxServiceManager;

impl ServiceManager for LinuxServiceManager {
    fn install(&self, args: &str) -> Result<(), Box<dyn Error>> {
        // 构建 systemd 服务文件路径
        let service_file_path = "/etc/systemd/system/webot-relay.service";

        // 获取当前可执行文件的路径
        let current_exe = std::env::current_exe()?;
        let exe_path = current_exe.to_str().unwrap();

        // 构建服务文件内容
        let service_content = format!(
            r#"
[Unit]
Description=Webot Relay Server
After=network.target

[Service]
Type=simple
ExecStart={} run {}
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#,
            exe_path, args
        );

        // 写入服务文件
        let mut file = File::create(service_file_path)?;
        file.write_all(service_content.as_bytes())?;

        // 重新加载 systemd 配置
        let output = Command::new("systemctl").arg("daemon-reload").output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new(
                "Failed to reload systemd daemon",
            )));
        }

        // 启用服务
        let output = Command::new("systemctl")
            .arg("enable")
            .arg("webot-relay")
            .output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to enable service")));
        }

        Ok(())
    }

    fn start(&self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("systemctl")
            .arg("start")
            .arg("webot-relay")
            .output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to start service")));
        }
        Ok(())
    }

    fn stop(&self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("systemctl")
            .arg("stop")
            .arg("webot-relay")
            .output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to stop service")));
        }
        Ok(())
    }

    fn status(&self) -> Result<String, Box<dyn Error>> {
        let output = Command::new("systemctl")
            .arg("status")
            .arg("webot-relay")
            .output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to get service status")));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn uninstall(&self) -> Result<(), Box<dyn Error>> {
        // 停止服务
        let _ = Command::new("systemctl")
            .arg("stop")
            .arg("webot-relay")
            .output();

        // 禁用服务
        let _ = Command::new("systemctl")
            .arg("disable")
            .arg("webot-relay")
            .output();

        // 删除服务文件
        let _ = std::fs::remove_file("/etc/systemd/system/webot-relay.service");

        // 重新加载 systemd 配置
        let _ = Command::new("systemctl").arg("daemon-reload").output();

        Ok(())
    }
}
