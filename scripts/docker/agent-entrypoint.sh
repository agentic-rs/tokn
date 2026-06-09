#!/bin/sh
set -eu

agent="${TOKN_AGENT:-codex}"
mode="${TOKN_MODE:-api-route}"
api_url="${TOKN_GATEWAY_API_URL:-http://gateway:4141}"
proxy_url="${TOKN_GATEWAY_PROXY_URL:-http://gateway:4142}"
ca_dir="${TOKN_AGENT_CA_DIR:-/tmp/tokn-router/ca}"
ca_cert="$ca_dir/ca.crt"
ca_bundle="$ca_dir/ca-bundle.crt"

case "$agent" in
  codex)
    agent_cmd="${TOKN_CODEX_CMD:-codex}"
    ;;
  opencode)
    agent_cmd="${TOKN_OPENCODE_CMD:-opencode}"
    ;;
  pi)
    agent_cmd="${TOKN_PI_CMD:-pi}"
    ;;
  *)
    echo "tokn-agent: unsupported TOKN_AGENT '$agent' (expected codex, opencode, or pi)" >&2
    exit 64
    ;;
esac

fetch_ca() {
  mkdir -p "$ca_dir"
  if ! curl -fsSL "$api_url/-/lan/ca.crt" -o "$ca_cert"; then
    echo "tokn-agent: failed to download CA from $api_url/-/lan/ca.crt" >&2
    echo "tokn-agent: ensure the gateway is running and accessible; ca_dir=$ca_dir" >&2
    exit 1
  fi
  if [ ! -s "$ca_cert" ]; then
    echo "tokn-agent: downloaded CA is missing or empty: $ca_cert" >&2
    echo "tokn-agent: ensure the gateway CA endpoint is serving a certificate" >&2
    exit 1
  fi
  if [ -f /etc/ssl/certs/ca-certificates.crt ]; then
    cat /etc/ssl/certs/ca-certificates.crt "$ca_cert" > "$ca_bundle"
  else
    cp "$ca_cert" "$ca_bundle"
  fi
  export NODE_EXTRA_CA_CERTS="$ca_cert"
  export SSL_CERT_FILE="$ca_bundle"
  export REQUESTS_CA_BUNDLE="$ca_bundle"
  export CURL_CA_BUNDLE="$ca_bundle"
  export GIT_SSL_CAINFO="$ca_bundle"
}

proxy_with_credentials() {
  proxy_user="$1"
  case "$proxy_url" in
    http://*)
      proxy_scheme="http"
      proxy_host="${proxy_url#http://}"
      ;;
    https://*)
      proxy_scheme="https"
      proxy_host="${proxy_url#https://}"
      ;;
    *://*)
      echo "tokn-agent: unsupported proxy URL scheme in TOKN_GATEWAY_PROXY_URL=$proxy_url" >&2
      echo "tokn-agent: expected http://, https://, or a scheme-less host:port" >&2
      exit 1
      ;;
    *)
      proxy_scheme="http"
      proxy_host="$proxy_url"
      ;;
  esac
  if [ -z "$proxy_host" ]; then
    echo "tokn-agent: TOKN_GATEWAY_PROXY_URL resolved to an empty proxy host" >&2
    exit 1
  fi
  printf '%s://%s:x@%s' "$proxy_scheme" "$proxy_user" "$proxy_host"
}

case "$mode" in
  api-route)
    export OPENAI_BASE_URL="$api_url/v1"
    export OPENAI_API_KEY="${OPENAI_API_KEY:-tokn-local}"
    export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-tokn-local}"
    ;;
  api-passthrough)
    export OPENAI_BASE_URL="$api_url/passthrough/v1"
    export OPENAI_API_KEY="${OPENAI_API_KEY:-tokn-local}"
    export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-tokn-local}"
    ;;
  proxy-switch)
    fetch_ca
    proxy_with_auth="$(proxy_with_credentials switch)"
    export HTTPS_PROXY="${HTTPS_PROXY:-$proxy_with_auth}"
    export HTTP_PROXY="${HTTP_PROXY:-$proxy_with_auth}"
    export ALL_PROXY="${ALL_PROXY:-$HTTPS_PROXY}"
    export NO_PROXY="${NO_PROXY:-gateway,localhost,127.0.0.1,::1}"
    ;;
  proxy-passthrough)
    fetch_ca
    proxy_with_auth="$(proxy_with_credentials passthrough)"
    export HTTPS_PROXY="${HTTPS_PROXY:-$proxy_with_auth}"
    export HTTP_PROXY="${HTTP_PROXY:-$proxy_with_auth}"
    export ALL_PROXY="${ALL_PROXY:-$HTTPS_PROXY}"
    export NO_PROXY="${NO_PROXY:-gateway,localhost,127.0.0.1,::1}"
    ;;
  *)
    echo "tokn-agent: unsupported TOKN_MODE '$mode' (expected api-route, proxy-switch, api-passthrough, or proxy-passthrough)" >&2
    exit 64
    ;;
esac

echo "tokn-agent: agent=$agent mode=$mode command=$agent_cmd" >&2
exec "$agent_cmd" "$@"
