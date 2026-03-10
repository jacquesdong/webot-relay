use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonResponse {
    pub errcode: i32,
    pub errmsg: String,
}

impl JsonResponse {
    pub fn success() -> Self {
        Self {
            errcode: 0,
            errmsg: "ok".to_string(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            errcode: 1,
            errmsg: format!("Error relaying request: {}", message),
        }
    }

    pub fn method_not_allowed() -> Self {
        Self {
            errcode: 1,
            errmsg: "Method Not Allowed".to_string(),
        }
    }

    pub fn url_not_configured() -> Self {
        Self {
            errcode: 1,
            errmsg: "URL not configured".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let response = JsonResponse::success();
        assert_eq!(response.errcode, 0);
        assert_eq!(response.errmsg, "ok");
    }

    #[test]
    fn test_error_response() {
        let response = JsonResponse::error("Connection refused");
        assert_eq!(response.errcode, 1);
        assert!(response.errmsg.contains("Connection refused"));
    }

    #[test]
    fn test_method_not_allowed() {
        let response = JsonResponse::method_not_allowed();
        assert_eq!(response.errcode, 1);
        assert_eq!(response.errmsg, "Method Not Allowed");
    }

    #[test]
    fn test_url_not_configured() {
        let response = JsonResponse::url_not_configured();
        assert_eq!(response.errcode, 1);
        assert_eq!(response.errmsg, "URL not configured");
    }
}
