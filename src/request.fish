#!/usr/bin/env fish

argparse --ignore-unknown H= d= -- $argv
or return

set -l args

if set -q _flag_H
    set -a args -H $_flag_H
else
    set -a args -H "Content-Type: application/json"
end

if set -q _flag_d
    set -a args -d $_flag_d
else
    set -a args -d '{
  "msgtype": "text",
  "text": {
    "content": "hello world"
  }
}'
end

if not string match -rq -- '^[^-]' $argv
    set -a args 'http://127.0.0.1:8000'
end

curl $args $argv
