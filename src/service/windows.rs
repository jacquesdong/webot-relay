use std::error::Error;
use std::process::Command;

use crate::service::{ServiceError, ServiceManager};

/// Windows 服务管理器，使用 sc 命令管理服务
pub struct WindowsServiceManager;

impl ServiceManager for WindowsServiceManager {
    fn install(&self, args: &str) -> Result<(), Box<dyn Error>> {
        // 构建服务安装命令
        let output = Command::new("sc")
            .arg("create")
            .arg("webot-relay")
            .arg("binPath=")
            .arg(format!(
                "\"{}\" run {}",
                std::env::current_exe()?.to_str().unwrap(),
                args
            ))
            .arg("start=")
            .arg("auto")
            .output()?;

        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to install service")));
        }

        Ok(())
    }

    fn start(&self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("sc")
            .arg("start")
            .arg("webot-relay")
            .output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to start service")));
        }
        Ok(())
    }

    fn stop(&self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("sc").arg("stop").arg("webot-relay").output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to stop service")));
        }
        Ok(())
    }

    fn status(&self) -> Result<String, Box<dyn Error>> {
        let output = Command::new("sc")
            .arg("query")
            .arg("webot-relay")
            .output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to get service status")));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn uninstall(&self) -> Result<(), Box<dyn Error>> {
        // 停止服务
        let _ = Command::new("sc").arg("stop").arg("webot-relay").output();

        // 删除服务
        let output = Command::new("sc")
            .arg("delete")
            .arg("webot-relay")
            .output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to uninstall service")));
        }

        Ok(())
    }
}
