# i18n Typed API 设计方案

## 设计目标

提供类型安全的 i18n API，实现：
1. **编译时类型检查**：参数类型和数量在编译时验证
2. **直观的函数调用**：直接调用函数，无需记忆枚举名
3. **IDE 自动补全**：函数名直接提供智能提示
4. **零运行时开销**：无参数翻译返回 `&'static str`
5. **清晰的命名空间**：使用模块路径组织翻译函数

## API 设计

### 1. 无参数翻译

**翻译文件：**
```yaml
welcome:
  description:
    en: "Webot Relay Server"
    zh-CN: "企业微信通知转发服务"

messages:
  service_installed:
    en: "Service installed successfully"
    zh-CN: "服务安装成功"
```

**生成的 API：**
```rust
pub mod welcome {
    /// Get translation for "welcome.description"
    /// 
    /// English: "Webot Relay Server"
    /// Chinese: "企业微信通知转发服务"
    pub fn description() -> &'static str {
        t!("welcome.description")
    }
}

pub mod messages {
    /// Get translation for "messages.service_installed"
    /// 
    /// English: "Service installed successfully"
    /// Chinese: "服务安装成功"
    pub fn service_installed() -> &'static str {
        t!("messages.service_installed")
    }
}
```

**使用方式：**
```rust
println!("{}", i18n::welcome::description());
println!("{}", i18n::messages::service_installed());
```

### 2. 带参数翻译

**翻译文件：**
```yaml
errors:
  parsing_address:
    en: "Error parsing address: %{error}"
    zh-CN: "地址解析错误: %{error}"
  invalid_ip:
    en: "Invalid IP address: %{host}, %{error}"
    zh-CN: "无效的 IP 地址: %{host}, %{error}"

messages:
  server_running_relay:
    en: "Server running on %{address} [relay]"
    zh-CN: "服务器运行在 %{address} [relay]"
```

**生成的 API：**
```rust
pub mod errors {
    /// Get translation for "errors.parsing_address"
    /// 
    /// English: "Error parsing address: %{error}"
    /// Chinese: "地址解析错误: %{error}"
    /// 
    /// # Parameters
    /// - `error`: Error description
    pub fn parsing_address(error: &str) -> String {
        t!("errors.parsing_address", error = error).to_string()
    }

    /// Get translation for "errors.invalid_ip"
    /// 
    /// English: "Invalid IP address: %{host}, %{error}"
    /// Chinese: "无效的 IP 地址: %{host}, %{error}"
    /// 
    /// # Parameters
    /// - `host`: Host address
    /// - `error`: Error description
    pub fn invalid_ip(host: &str, error: &str) -> String {
        t!("errors.invalid_ip", host = host, error = error).to_string()
    }
}

pub mod messages {
    /// Get translation for "messages.server_running_relay"
    /// 
    /// English: "Server running on %{address} [relay]"
    /// Chinese: "服务器运行在 %{address} [relay]"
    /// 
    /// # Parameters
    /// - `address`: Server address
    pub fn server_running_relay(address: &str) -> String {
        t!("messages.server_running_relay", address = address).to_string()
    }
}
```

**使用方式：**
```rust
eprintln!("{}", i18n::errors::parsing_address("invalid format"));
eprintln!("{}", i18n::errors::invalid_ip("localhost", "connection refused"));
println!("{}", i18n::messages::server_running_relay("127.0.0.1:8000"));
```

## 实现方案

### 1. build.rs 修改

**解析翻译文件：**
```rust
fn parse_translations(yaml: &serde_yaml::Value) -> Vec<TranslationEntry> {
    let mut entries = Vec::new();
    extract_translations(yaml, "", &mut entries);
    entries
}

struct TranslationEntry {
    key: String,           // "errors.parsing_address"
    module_path: String,   // "errors"
    function_name: String, // "parsing_address"
    params: Vec<String>,   // ["error"]
    en_text: String,       // "Error parsing address: %{error}"
    zh_text: String,       // "地址解析错误: %{error}"
}
```

**生成模块结构：**
```rust
fn generate_i18n_modules(entries: &[TranslationEntry]) -> String {
    // 按模块分组
    let mut modules: HashMap<String, Vec<&TranslationEntry>> = HashMap::new();
    for entry in entries {
        modules
            .entry(entry.module_path.clone())
            .or_insert_with(Vec::new)
            .push(entry);
    }
    
    // 生成模块代码
    let mut code = String::new();
    for (module_name, entries) in modules {
        code.push_str(&format!("pub mod {} {{\n", module_name));
        for entry in entries {
            code.push_str(&generate_function(entry));
        }
        code.push_str("}\n\n");
    }
    
    code
}

fn generate_function(entry: &TranslationEntry) -> String {
    let mut code = String::new();
    
    // 函数文档
    code.push_str(&format!("    /// Get translation for \"{}\"\n", entry.key));
    code.push_str("    /// \n");
    code.push_str(&format!("    /// English: \"{}\"\n", entry.en_text));
    code.push_str(&format!("    /// Chinese: \"{}\"\n", entry.zh_text));
    
    if !entry.params.is_empty() {
        code.push_str("    /// \n");
        code.push_str("    /// # Parameters\n");
        for param in &entry.params {
            code.push_str(&format!("    /// - `{}`: Parameter\n", param));
        }
    }
    
    // 函数签名
    if entry.params.is_empty() {
        // 无参数函数
        code.push_str(&format!("    pub fn {}() -> &'static str {{\n", entry.function_name));
        code.push_str(&format!("        t!(\"{}\")\n", entry.key));
        code.push_str("    }\n\n");
    } else {
        // 带参数函数
        let params: Vec<String> = entry.params.iter()
            .map(|p| format!("{}: &str", p))
            .collect();
        let args: Vec<String> = entry.params.iter()
            .map(|p| format!("{} = {}", p, p))
            .collect();
        
        code.push_str(&format!("    pub fn {}({}) -> String {{\n", 
            entry.function_name, params.join(", ")));
        code.push_str(&format!("        t!(\"{}\", {}).to_string()\n", 
            entry.key, args.join(", ")));
        code.push_str("    }\n\n");
    }
    
    code
}
```

### 2. 生成的 i18n.rs 结构

```rust
// Auto-generated by build.rs - DO NOT EDIT
// This file is generated from locales/app.yml

use rust_i18n::t;

pub mod welcome {
    pub fn description() -> &'static str {
        t!("welcome.description")
    }

    pub fn long_description() -> &'static str {
        t!("welcome.long_description")
    }
}

pub mod commands {
    pub mod service {
        pub fn description() -> &'static str {
            t!("commands.service.description")
        }

        pub fn install_description() -> &'static str {
            t!("commands.service.install.description")
        }

        pub fn start_description() -> &'static str {
            t!("commands.service.start.description")
        }

        pub fn stop_description() -> &'static str {
            t!("commands.service.stop.description")
        }

        pub fn status_description() -> &'static str {
            t!("commands.service.status.description")
        }

        pub fn uninstall_description() -> &'static str {
            t!("commands.service.uninstall.description")
        }
    }

    pub mod run {
        pub fn description() -> &'static str {
            t!("commands.run.description")
        }

        pub fn bind_description() -> &'static str {
            t!("commands.run.bind.description")
        }

        pub fn url_description() -> &'static str {
            t!("commands.run.url.description")
        }

        pub fn verbose_description() -> &'static str {
            t!("commands.run.verbose.description")
        }
    }
}

pub mod options {
    pub mod version {
        pub fn description() -> &'static str {
            t!("options.version.description")
        }
    }
}

pub mod errors {
    pub fn parsing_address(error: &str) -> String {
        t!("errors.parsing_address", error = error).to_string()
    }

    pub fn invalid_ip(host: &str, error: &str) -> String {
        t!("errors.invalid_ip", host = host, error = error).to_string()
    }

    pub fn binding_address(error: &str) -> String {
        t!("errors.binding_address", error = error).to_string()
    }

    pub fn starting_server(error: &str) -> String {
        t!("errors.starting_server", error = error).to_string()
    }

    pub fn installing_service(error: &str) -> String {
        t!("errors.installing_service", error = error).to_string()
    }

    pub fn starting_service(error: &str) -> String {
        t!("errors.starting_service", error = error).to_string()
    }

    pub fn stopping_service(error: &str) -> String {
        t!("errors.stopping_service", error = error).to_string()
    }

    pub fn getting_status(error: &str) -> String {
        t!("errors.getting_status", error = error).to_string()
    }

    pub fn uninstalling_service(error: &str) -> String {
        t!("errors.uninstalling_service", error = error).to_string()
    }
}

pub mod messages {
    pub fn service_installed() -> &'static str {
        t!("messages.service_installed")
    }

    pub fn service_started() -> &'static str {
        t!("messages.service_started")
    }

    pub fn service_stopped() -> &'static str {
        t!("messages.service_stopped")
    }

    pub fn service_uninstalled() -> &'static str {
        t!("messages.service_uninstalled")
    }

    pub fn server_running_relay(address: &str) -> String {
        t!("messages.server_running_relay", address = address).to_string()
    }

    pub fn server_running_dumb(address: &str) -> String {
        t!("messages.server_running_dumb", address = address).to_string()
    }
}
```

## 模块路径规则

### 转换规则

| 翻译键 | 模块路径 | 函数名 |
|--------|----------|--------|
| `welcome.description` | `welcome::description()` | `i18n::welcome::description()` |
| `commands.service.install.description` | `commands::service::install_description()` | `i18n::args::commands::service::install_description()` |
| `errors.parsing_address` | `errors::parsing_address(error: &str)` | `i18n::errors::parsing_address("error")` |
| `messages.server_running_relay` | `messages::server_running_relay(address: &str)` | `i18n::messages::server_running_relay("addr")` |

### 模块层次结构

```
i18n
├── welcome
│   ├── description()
│   └── long_description()
├── commands
│   ├── service
│   │   ├── description()
│   │   ├── install_description()
│   │   ├── start_description()
│   │   ├── stop_description()
│   │   ├── status_description()
│   │   └── uninstall_description()
│   └── run
│       ├── description()
│       ├── bind_description()
│       ├── url_description()
│       └── verbose_description()
├── options
│   └── version
│       └── description()
├── errors
│   ├── parsing_address(error: &str)
│   ├── invalid_ip(host: &str, error: &str)
│   ├── binding_address(error: &str)
│   └── ...
└── messages
    ├── service_installed()
    ├── service_started()
    ├── server_running_relay(address: &str)
    └── ...
```

## 参数类型推断

### 默认类型

所有参数默认为 `&str` 类型，因为：
1. 翻译文本是字符串
2. 大多数参数是字符串类型
3. 其他类型可以通过 `.to_string()` 转换

### 未来扩展

可以支持更多类型：
```rust
// 未来可能支持
pub mod server {
    pub fn stats(connections: i32, uptime: f64) -> String {
        t!("server.stats", connections = connections, uptime = uptime).to_string()
    }
}
```

## 使用示例

### 在 main.rs 中使用

**旧代码：**
```rust
use rust_i18n::t;

eprintln!("{}", t!("errors.parsing_address", error = err));
println!("{}", t!("messages.server_running_relay", address = socket_addr));
```

**新代码：**
```rust
// 不需要导入，使用完整路径
eprintln!("{}", i18n::errors::parsing_address(&err));
println!("{}", i18n::messages::server_running_relay(&socket_addr));
```

### 在 args.rs 中使用

**旧代码：**
```rust
use rust_i18n::t;

#[command(about = t!("welcome.description").to_string())]
```

**新代码：**
```rust
// 不需要导入，使用完整路径
#[command(about = i18n::welcome::description().to_string())]
```

### 在错误处理中使用

```rust
fn handle_error(err: &str) {
    eprintln!("{}", i18n::errors::parsing_address(err));
}

fn show_server_info(address: &str) {
    println!("{}", i18n::messages::server_running_relay(address));
}

fn show_service_status() {
    println!("{}", i18n::messages::service_installed());
}
```

### 在命令行参数中使用

```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "webot-relay",
    about = i18n::welcome::description().to_string()
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = i18n::args::commands::service::description().to_string())]
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },

    #[command(about = i18n::args::commands::run::description().to_string())]
    Run {
        #[arg(help = i18n::args::commands::run::bind_description().to_string())]
        bind: String,
    },
}
```

## 优势分析

### 1. 清晰的命名空间

**避免命名冲突：**
```rust
// ✅ 多个模块可以有同名函数
i18n::welcome::description()
i18n::args::commands::description()
i18n::errors::description()

// ❌ 扁平函数名需要避免冲突
welcome_description()
commands_description()
errors_description()
```

### 2. 函数名更短

**对比：**
```rust
// 扁平函数名
i18n::args::commands_service_install_description()

// 模块路径
i18n::args::commands::service::install_description()
```

### 3. 类型安全

**编译时检查参数：**
```rust
// ✅ 正确
i18n::errors::parsing_address("invalid format");

// ❌ 编译错误：参数数量不匹配
i18n::errors::parsing_address();

// ❌ 编译错误：参数类型不匹配
i18n::errors::parsing_address(123);
```

### 4. IDE 支持

**自动补全：**
- 输入 `i18n::` 会提示所有模块
- 输入 `i18n::errors::` 会提示该模块下的所有函数
- 按模块分组，更易查找

**参数提示：**
- IDE 会显示函数签名和参数名称
- 鼠标悬停会显示翻译文本

### 5. 重构友好

**修改翻译键：**
- 修改函数名后，编译器会提示所有调用处
- 重构工具可以自动更新所有调用

**添加参数：**
- 添加新参数后，编译器会提示所有调用处需要更新

### 6. 性能优势

**无参数翻译：**
- 返回 `&'static str`，零运行时开销
- 编译时确定字符串地址

**带参数翻译：**
- 只在需要时进行字符串拼接
- 避免不必要的内存分配

## 对比分析

### 与扁平函数名方案对比

| 特性 | 扁平函数名 | 模块路径 |
|------|-----------|----------|
| 命名空间 | ❌ 混乱 | ✅ 清晰 |
| 函数名长度 | ❌ 可能很长 | ✅ 更短 |
| 命名冲突 | ❌ 需要避免 | ✅ 自然避免 |
| IDE 支持 | ✅ 良好 | ✅ 更好 |
| 代码组织 | ❌ 扁平 | ✅ 层次化 |
| 导入方式 | ⚠️ 需要 use * | ✅ 完整路径 |

### 与枚举方案对比

| 特性 | 枚举方案 | 模块路径方案 |
|------|----------|--------------|
| API 直观性 | ⚠️ 需要调用 `.tr()` | ✅ 直接调用函数 |
| 参数类型安全 | ❌ 无参数检查 | ✅ 编译时检查 |
| IDE 支持 | ⚠️ 需要记住枚举名 | ✅ 函数名直接提示 |
| 性能 | ✅ 零开销 | ✅ 无参数零开销 |
| 重构友好 | ✅ 编译时检查 | ✅ 编译时检查 |
| 文档友好 | ⚠️ 需要额外注释 | ✅ 函数文档直接显示 |
| 命名空间 | ❌ 无 | ✅ 清晰 |

### 与字符串字面量对比

| 特性 | 字符串字面量 | 模块路径方案 |
|------|--------------|--------------|
| 拼写错误检测 | ❌ 运行时 | ✅ 编译时 |
| 参数检查 | ❌ 无 | ✅ 编译时 |
| IDE 支持 | ❌ 无 | ✅ 有 |
| 重构友好 | ❌ 无 | ✅ 有 |
| 命名空间 | ❌ 无 | ✅ 清晰 |

## 实现步骤

### 阶段1：修改 build.rs

1. 解析翻译文件，提取参数信息
2. 按模块分组翻译键
3. 生成模块结构和函数签名
4. 生成文档注释

### 阶段2：更新使用代码

1. 替换 `t!("key")` 为 `i18n::module::function()`
2. 替换 `t!("key", param = value)` 为 `i18n::module::function(value)`
3. 更新 args.rs 中的使用
4. 更新 main.rs 中的使用

### 阶段3：测试验证

1. 测试无参数翻译
2. 测试带参数翻译
3. 测试多语言切换
4. 测试错误情况

### 阶段4：文档和示例

1. 更新文档
2. 添加使用示例
3. 添加迁移指南

## 迁移指南

### 从旧方案迁移

**步骤1：更新 build.rs**
```bash
# 修改 build.rs 以生成模块结构的函数 API
```

**步骤2：更新代码调用**
```rust
// 旧代码
use rust_i18n::t;
let text = t!("welcome.description").to_string();
let error = t!("errors.parsing_address", error = err).to_string();

// 新代码
// 不需要导入，使用完整路径
let text = i18n::welcome::description().to_string();
let error = i18n::errors::parsing_address(&err);
```

**步骤3：更新 args.rs**
```rust
// 旧代码
use rust_i18n::t;
#[command(about = t!("welcome.description").to_string())]

// 新代码
// 不需要导入，使用完整路径
#[command(about = i18n::welcome::description().to_string())]
```

## 总结

这个模块路径的 typed API 设计方案提供了：

✅ **类型安全**：编译时检查参数类型和数量  
✅ **直观 API**：直接调用函数，无需记忆枚举名  
✅ **IDE 支持**：自动补全和参数提示  
✅ **零运行时开销**：无参数翻译返回 `&'static str`  
✅ **重构友好**：修改函数签名时编译器提示所有调用处  
✅ **文档友好**：函数文档直接显示翻译文本  
✅ **清晰命名空间**：模块路径组织，避免命名冲突  
✅ **函数名更短**：模块路径比扁平函数名更简洁  
✅ **完整路径使用**：不使用 `use *`，代码更清晰  

这是一个安全、高效、易用、组织良好的 i18n 解决方案！
