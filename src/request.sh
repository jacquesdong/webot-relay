#!/usr/bin/env bash

curl 'http://127.0.0.1:8000' \
   -H 'Content-Type: application/json' \
   -d '{
    "msgtype": "text",
    "text": {
        "content": "hello world"
    }
}'
