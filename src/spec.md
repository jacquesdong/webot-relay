# Python HTTP Relay Server - 技术规范与Rust重构指南

## 1. 功能描述

### 1.1 核心功能概述

本程序是一个HTTP中继服务器，接收POST请求并将其转发到指定的目标URL。当未配置目标URL时，程序返回预设的成功响应。

### 1.2 功能特性

- 接收HTTP POST请求
- 将请求头和请求体转发到目标URL
- 支持通过命令行参数或环境变量配置目标URL
- 支持详细日志输出模式
- 返回JSON格式的响应

### 1.3 Python版本要求与兼容性

| 项目 | 要求 |
|------|------|
| **最低Python版本** | Python 3.6 |
| **推荐Python版本** | Python 3.8+ |
| **依赖标准库** | `http.server`, `urllib.request`, `argparse`, `os`, `sys`, `typing` |
| **支持操作系统** | Linux, macOS, Windows |

#### Python 3.6 兼容性说明

- ✅ 使用 `typing.Tuple` 类型注解
- ✅ 使用 `format()` 字符串格式化
- ✅ 使用 `dict.items()` 遍历字典
- ✅ 使用 `urllib.request.urlopen`
- ⚠️ 不支持 f-strings (Python 3.6+ 支持)
- ⚠️ 不支持 `http.server.HTTPServer` 的某些新特性

#### Python 3.10+ 兼容性说明

- ✅ 完全兼容
- ⚠️ `http.server` 模块有轻微API变化，但向后兼容

---

## 2. 接口定义

### 2.1 命令行接口

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `-b`, `--bind` | 字符串 | 否 | `:8000` | 监听地址，格式为 `:端口` 或 `主机:端口` |
| `--url` | 字符串 | 否 | - | 转发目标URL |
| `-v`, `--verbose` | 标志 | 否 | false | 启用详细日志输出 |

#### 使用示例

```bash
# 基本使用（默认监听所有网卡8000端口）
python main.py

# 指定端口
python main.py --bind 8080
python main.py --bind :8080

# 指定主机和端口
python main.py --bind 127.0.0.1:9000

# 转发到目标服务器
python main.py --bind :8000 --url http://localhost:8001

# 启用详细日志
python main.py --bind :8000 -v --url http://example.com

# 使用环境变量配置URL
export WEBOT_URL=http://localhost:8001
python main.py --bind :8000
```

### 2.2 环境变量

| 变量名 | 类型 | 说明 |
|--------|------|------|
| `WEBOT_URL` | 字符串 | 转发目标URL（优先级低于命令行参数） |

### 2.3 HTTP接口

#### 请求
- **方法**: POST
- **请求头**: 透传所有请求头
- **请求体**: 透传所有请求体数据

#### 响应

**成功响应** (未配置URL时):
```json
{
  "errcode": 0,
  "errmsg": "ok"
}
```

**中继响应** (配置URL时):
- 透传目标URL返回的所有响应头和状态码

**错误响应** (中继失败时):
```json
{
  "errcode": 1,
  "errmsg": "Error relaying request: <错误信息>"
}
```

#### 错误码定义

| errcode | 说明 |
|---------|------|
| 0 | 成功 (未配置URL时返回) |
| 1 | 转发请求失败 |

> 注意: 无效端口号、无效绑定地址等情况程序直接退出，不返回JSON响应。

#### 非POST请求处理

对于非POST的HTTP请求，返回:
```json
{
  "errcode": 1,
  "errmsg": "Method Not Allowed"
}
```
状态码: 405

---

## 3. 数据结构

### 3.1 配置结构

```rust
struct Config {
    bind: String,           // 绑定地址 (e.g., ":8000" or "localhost:8000")
    url: Option<String>,    // 转发目标URL
    verbose: bool,          // 详细日志模式
}
```

### 3.2 地址解析结果

```rust
struct Address {
    host: String,  // 主机名或空字符串
    port: u16,     // 端口号
}
```

### 3.3 HTTP响应结构

```rust
struct JsonResponse {
    errcode: i32,   // 错误码: 0成功, 1失败
    errmsg: String, // 错误信息
}
```

---

## 4. 算法逻辑

### 4.1 主流程

```
┌─────────────────────────────────────────────────────────┐
│                      程序启动                            │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│           解析命令行参数 (-b, --bind, --url, -v)        │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│        解析地址字符串 (host, port)                       │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│    获取目标URL (命令行参数 > 环境变量 > 空)              │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│              创建HTTP服务器并监听                        │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│                   等待POST请求                           │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
            ┌─────────┴─────────┐
            │   处理POST请求    │
            └─────────┬─────────┘
                      │
           ┌──────────┴──────────┐
           │ verbose 模式?       │
           └──────────┬──────────┘
                │           │
               是          否
                │           │
                ▼           ▼
        打印请求信息    不打印
                │           │
                └───────────┘
                      │
                      ▼
            ┌─────────┴─────────┐
            │   URL 已配置?     │
            └─────────┬─────────┘
                      │
           ┌──────────┴──────────┐
           │                     │
          是                    否
           │                     │
           ▼                     ▼
    转发请求到目标URL    返回成功响应
           │                     │
           └──────────┬──────────┘
                      │
                      ▼
            ┌─────────┴─────────┐
            │   返回响应给客户端 │
            └───────────────────┘
```

### 4.2 地址解析算法

```
parse_bind_address(bind_addr: &str) -> (String, u16)

输入: ":8000" 或 "localhost:8000" 或 "0.0.0.0:8000"

IF bind_addr 为空 THEN
    RETURN ("", 8000)
END IF

IF bind_addr 以 ':' 开头 THEN
    RETURN ("", bind_addr[1:].parse::<u16>())
END IF

parts = bind_addr.rsplit(':', 1)
IF parts.len() == 2 THEN
    host = parts[0]
    TRY
        port = parts[1].parse::<u16>()
        RETURN (host, port)
    CATCH
        RETURN (bind_addr, 8000)
    END TRY
END IF

RETURN (bind_addr, 8000)
```

### 4.3 请求处理算法

```
handle_post_request(request):

1. 读取 Content-Length 请求头
2. 读取请求体数据 (post_data)
3. 获取所有请求头 (request_headers)

IF verbose 模式 THEN
    打印所有请求头到 stderr
    打印请求体到 stderr (尝试UTF-8解码，失败则打印二进制长度)
END IF

IF url 已配置 THEN
    TRY
        创建转发请求 (POST method, 原始headers, post_data)
        发送请求到目标URL
        获取响应状态码和响应体
        
        如果启用了 verbose 模式:
            记录响应状态码
            记录所有响应头
            记录响应体（如果是文本则显示内容，否则显示二进制数据大小）
        
        发送响应状态码给客户端
        透传所有响应头给客户端
        发送响应体给客户端
    CATCH 异常
        发送 500 状态码
        发送响应头 Server: webot-relay
        发送 JSON 错误响应 {"errcode": 1, "errmsg": "<错误信息>"}
    END TRY
ELSE
    发送 200 状态码
    发送响应头 Server: webot-relay
    发送 JSON 成功响应 {"errcode": 1, "errmsg": "URL not configured"}
END IF

### 4.3 地址解析规则

- 如果 bind_addr 为空，返回 ('', 8000)
- 如果 bind_addr 包含 ":"，按最后一个 ":" 分割，前半部分为 host，后半部分为 port
- 如果 bind_addr 不包含 ":"，尝试将其解析为端口号，如果成功则返回 ('', port)，否则返回 (bind_addr, 8000)

### 4.4 服务器启动日志

- 格式："Server running on {host}:{port} [relay]"（如果配置了 URL）
- 格式："Server running on {host}:{port} [dumb]"（如果未配置 URL）
```

---

## 5. 关键实现细节

### 5.1 HTTP服务器实现

- 使用标准库 `std::net::TcpListener` 监听TCP连接
- 使用 `http` crate 解析和构建HTTP请求/响应
- 或使用 `actix-web` / `axum` 框架简化HTTP处理

### 5.2 请求转发实现

```rust
// 伪代码
let client = reqwest::Client::new();
let response = client
    .request(method, &target_url)
    .headers(original_headers)
    .body(post_data)
    .send()
    .await?;

// 透传响应
response.status()  // HTTP状态码
response.headers() // 响应头
response.bytes()   // 响应体
```

### 5.3 错误处理

| 错误场景 | 处理方式 | 响应状态码 | 响应内容 |
|----------|----------|------------|----------|
| 目标URL连接失败 | 返回错误信息 | 500 | `{"errcode": 1, "errmsg": "..."}` |
| 目标URL超时 | 返回错误信息 | 500 | `{"errcode": 1, "errmsg": "..."}` |
| 请求体解析失败 | 返回错误信息 | 500 | `{"errcode": 1, "errmsg": "..."}` |
| 无效的地址格式 | 打印错误并退出 | N/A | N/A |
| 端口号无效 | 打印错误并退出 | N/A | N/A |

---

## 6. 边界条件

### 6.1 输入验证

| 条件 | 处理 |
|------|------|
| 地址为空 | 使用默认 `:8000` |
| 端口号为0 | 使用默认端口8000 |
| 端口号超过65535 | 返回错误 |
| URL格式无效 | 记录但继续运行（运行时失败） |

### 6.2 请求处理

| 条件 | 处理 |
|------|------|
| 无 Content-Length 头 | 读取0字节数据 |
| 请求体为空 | 转发空请求体 |
| 二进制请求体 | 透传原始字节 |
| 请求头包含中文 | 透传原始编码 |

### 6.3 响应处理

| 条件 | 处理 |
|------|------|
| 目标URL返回非200 | 透传原始状态码 |
| 目标URL无响应 | 超时返回500错误 |
| 目标URL响应过大 | 透传完整响应 |

---

## 7. Rust实现要求

### 7.1 依赖项

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }      # 异步运行时
axum = "0.7"                                         # HTTP框架
reqwest = { version = "0.11", features = ["json"] } # HTTP客户端
serde = { version = "1", features = ["derive"] }    # 序列化
serde_json = "1"                                     # JSON处理
clap = { version = "4", features = ["derive"] }      # 命令行解析
```

### 7.2 代码结构建议

```
src/
├── main.rs           # 入口点，命令行解析和服务器启动
├── config.rs         # 配置结构体定义
├── handler.rs        # HTTP请求处理器
├── relay.rs          # 请求转发逻辑
└── response.rs       # JSON响应构建
```

### 7.3 性能要求

- 支持高并发连接（使用异步框架）
- 请求处理延迟 < 100ms（不含网络转发）
- 内存占用 < 10MB（空闲状态）

### 7.4 兼容性要求

- 兼容Python版本的所有功能特性
- 保持相同的命令行参数和行为
- 保持相同的响应格式
- 支持Linux/macOS/Windows

---

## 8. 测试用例

### 8.1 功能测试 (Python)

| 测试场景 | 预期行为 |
|----------|----------|
| `python main.py` (无参数) | 启动在8000端口，返回成功响应 |
| `python main.py --bind :8000` | 启动在8000端口 |
| `python main.py --bind :8080 --url http://example.com` | 转发请求到example.com |
| `WEBOT_URL=http://test.com python main.py` | 使用环境变量的URL |
| `python main.py --bind :8000 -v` | 打印详细日志 |
| 发送POST请求带JSON body | 正确转发/打印 |

### 8.2 功能测试 (Rust)

| 测试场景 | 预期行为 |
|----------|----------|
| `cargo run -- --bind :8000` | 启动在8000端口 |
| `cargo run -- --bind :8080 --url http://example.com` | 转发请求到example.com |
| `WEBOT_URL=http://test.com cargo run -- --bind :8000` | 使用环境变量的URL |
| `cargo run -- --bind :8000 -v` | 打印详细日志 |
| 发送POST请求带JSON body | 正确转发/打印 |

#### Rust 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bind_address() {
        // 测试空地址
        let (host, port) = parse_bind_address("");
        assert_eq!(host, "");
        assert_eq!(port, 8000);

        // 测试仅端口
        let (host, port) = parse_bind_address(":8080");
        assert_eq!(host, "");
        assert_eq!(port, 8080);

        // 测试主机:端口
        let (host, port) = parse_bind_address("127.0.0.1:9000");
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 9000);

        // 测试无效端口回退
        let (host, port) = parse_bind_address("invalid");
        assert_eq!(host, "invalid");
        assert_eq!(port, 8000);
    }

    #[test]
    fn test_validate_port() {
        // 有效端口
        assert!(validate_port(8080).is_ok());

        // 无效端口: 0
        assert!(validate_port(0).is_err());

        // 无效端口: 超过65535
        assert!(validate_port(65536).is_err());

        // 边界值
        assert!(validate_port(1).is_ok());
        assert!(validate_port(65535).is_ok());
    }

    #[test]
    fn test_json_response_success() {
        let response = JsonResponse::success();
        assert_eq!(response.errcode, 0);
        assert_eq!(response.errmsg, "ok");
    }

    #[test]
    fn test_json_response_error() {
        let response = JsonResponse::error("Connection refused");
        assert_eq!(response.errcode, 1);
        assert!(response.errmsg.contains("Connection refused"));
    }
}
```

#### Rust 集成测试

```rust
// tests/integration_test.rs

use std::process::Command;
use std::net::TcpStream;
use std::io::{Write, Read};

#[test]
fn test_server_starts() {
    // 启动服务器
    let mut child = Command::new("cargo")
        .args(&["run", "--", "--bind", ":18030"])
        .spawn()
        .expect("Failed to start server");

    // 等待服务器启动
    std::thread::sleep(std::time::Duration::from_secs(1));

    // 发送POST请求
    let mut stream = TcpStream::connect("127.0.0.1:18030").unwrap();
    stream.write_all(b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 13\r\n\r\n{\"test\": \"ok\"}").unwrap();

    // 读取响应
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    // 验证响应
    assert!(response.contains("200 OK"));
    assert!(response.contains("errcode"));

    // 清理
    child.kill().unwrap();
}

#[test]
fn test_invalid_port() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--bind", ":99999"])
        .output()
        .expect("Failed to execute");

    // 应该失败并显示错误信息
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Port must be between"));
}
```

### 8.3 错误场景测试

| 测试场景 | 预期行为 |
|----------|----------|
| 目标URL不可达 | 返回500错误 |
| 发送非POST请求 | 返回405 Method Not Allowed |
| 无效端口号 (--bind :99999) | 程序报错退出 |
| 无效绑定地址 | 程序报错退出 |

---

## 9. 附录

### 9.1 原Python代码关键变量

| 变量名 | 类型 | 说明 |
|--------|------|------|
| `url` | `Option<String>` | 转发目标URL |
| `verbose` | `bool` | 详细日志标志 |
| `post_data` | `bytes` | POST请求体 |
| `request_headers` | `dict` | 请求头字典 |

### 9.2 JSON响应格式

```rust
// 成功响应
JsonResponse { errcode: 0, errmsg: "ok" }

// 错误响应
JsonResponse { errcode: 1, errmsg: "Error relaying request: <error>" }
```
