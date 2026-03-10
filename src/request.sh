#!/usr/bin/env bash

# 解析命令行参数，忽略未知选项
HEAD="Content-Type: application/json"
DATA='{
  "msgtype": "text",
  "text": {
    "content": "hello world"
  }
}'

# 存储非选项参数（URL 和其他选项）
other_args=()

# 解析命令行参数
while [[ $# -gt 0 ]]; do
  case $1 in
    -H)
      if [[ $# -gt 1 ]]; then
        HEAD="$2"
        shift 2
      else
        echo "Error: -H requires an argument"
        exit 1
      fi
      ;;
    -d)
      if [[ $# -gt 1 ]]; then
        DATA="$2"
        shift 2
      else
        echo "Error: -d requires an argument"
        exit 1
      fi
      ;;
    *)
      # 收集其他参数
      other_args+=($1)
      shift
      ;;
  esac
done

# 构建 curl 命令
curl_cmd=('curl' '-H' "$HEAD" '-d' "$DATA")

# 添加其他参数
curl_cmd+=(${other_args[@]})

# 检查是否有 URL（非选项参数）
has_url=false
for arg in "${other_args[@]}"; do
  if [[ ! $arg =~ ^- ]]; then
    has_url=true
    break
  fi
done

# 如果没有指定 URL，添加默认值
if [[ $has_url == false ]]; then
  curl_cmd+=('http://127.0.0.1:8000')
fi

# 执行 curl 命令
"${curl_cmd[@]}"
   
