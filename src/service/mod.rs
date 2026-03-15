use std::error::Error;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

/// 服务管理器 trait，定义了服务管理的基本操作
pub trait ServiceManager {
    /// 安装服务
    fn install(&self, args: &str) -> Result<(), Box<dyn Error>>;
    /// 启动服务
    fn start(&self) -> Result<(), Box<dyn Error>>;
    /// 停止服务
    fn stop(&self) -> Result<(), Box<dyn Error>>;
    /// 查看服务状态
    fn status(&self) -> Result<String, Box<dyn Error>>;
    /// 卸载服务
    fn uninstall(&self) -> Result<(), Box<dyn Error>>;
}

/// 获取当前平台的服务管理器
pub fn get_service_manager() -> Box<dyn ServiceManager> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxServiceManager)
    }

    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsServiceManager)
    }

    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSServiceManager)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        panic!("Unsupported operating system")
    }
}

/// 服务管理错误
#[derive(Debug)]
pub struct ServiceError {
    pub message: String,
}

impl Error for ServiceError {}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ServiceError {
    pub fn new(message: &str) -> Self {
        ServiceError {
            message: message.to_string(),
        }
    }
}
