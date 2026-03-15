use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use crate::service::{ServiceError, ServiceManager};

/// macOS 服务管理器，使用 launchd 管理服务
pub struct MacOSServiceManager;

impl ServiceManager for MacOSServiceManager {
    fn install(&self, args: &str) -> Result<(), Box<dyn Error>> {
        // 构建 launchd 配置文件路径
        let plist_path = "~/Library/LaunchAgents/com.webot.relay.plist";
        let plist_path = shellexpand::tilde(plist_path).into_owned();

        // 构建配置文件内容
        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.webot.relay</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>run</string>
        {}
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>~/Library/Logs/webot-relay.log</string>
    <key>StandardErrorPath</key>
    <string>~/Library/Logs/webot-relay.error.log</string>
</dict>
</plist>"#,
            std::env::current_exe()?.to_str().unwrap(),
            args.split_whitespace()
                .map(|arg| format!("        <string>{}</string>", arg))
                .collect::<String>()
        );

        // 写入配置文件
        let mut file = File::create(plist_path.clone())?;
        file.write_all(plist_content.as_bytes())?;

        // 加载服务
        let output = Command::new("launchctl")
            .arg("load")
            .arg(plist_path)
            .output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to load service")));
        }

        Ok(())
    }

    fn start(&self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("launchctl")
            .arg("start")
            .arg("com.webot.relay")
            .output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to start service")));
        }
        Ok(())
    }

    fn stop(&self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("launchctl")
            .arg("stop")
            .arg("com.webot.relay")
            .output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to stop service")));
        }
        Ok(())
    }

    fn status(&self) -> Result<String, Box<dyn Error>> {
        let output = Command::new("launchctl")
            .arg("list")
            .arg("com.webot.relay")
            .output()?;
        if !output.status.success() {
            return Err(Box::new(ServiceError::new("Failed to get service status")));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn uninstall(&self) -> Result<(), Box<dyn Error>> {
        // 停止服务
        let _ = Command::new("launchctl")
            .arg("stop")
            .arg("com.webot.relay")
            .output();

        // 卸载服务
        let plist_path = "~/Library/LaunchAgents/com.webot.relay.plist";
        let plist_path = shellexpand::tilde(plist_path).into_owned();
        let _ = Command::new("launchctl")
            .arg("unload")
            .arg(plist_path.clone())
            .output();

        // 删除配置文件
        let _ = std::fs::remove_file(plist_path);

        Ok(())
    }
}
