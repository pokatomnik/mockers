#!/usr/bin/env bash
set -euo pipefail

readonly MOCKS_DIR="/mocks"
readonly HOST="0.0.0.0"
readonly PORT="8080"

error() {
  echo "error: $1" >&2
  exit 64
}

require_value() {
  local flag="$1"
  local value="${2-}"

  if [[ -z "$value" || "$value" == -* ]]; then
    error "option '$flag' requires a value"
  fi
}

validate_bool() {
  local flag="$1"
  local value="$2"

  case "$value" in
    true|false) ;;
    *)
      error "option '$flag' must be 'true' or 'false'"
      ;;
  esac
}

validate_log_level() {
  local flag="$1"
  local value="$2"

  case "$value" in
    info|debug|trace) ;;
    *)
      error "option '$flag' must be one of: info, debug, trace"
      ;;
  esac
}

validate_preflight() {
  local flag="$1"
  local value="$2"

  case "$value" in
    mirror|permissive) ;;
    *)
      error "option '$flag' must be one of: mirror, permissive"
      ;;
  esac
}

validate_uint() {
  local flag="$1"
  local value="$2"

  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    error "option '$flag' must be a non-negative integer"
  fi
}

allowed_args=()

while (($#)); do
  case "$1" in
    -c|--cors)
      if (($# >= 2)) && [[ "${2}" != -* ]]; then
        validate_bool "$1" "$2"
        allowed_args+=("--cors" "$2")
        shift 2
      else
        allowed_args+=("--cors")
        shift
      fi
      ;;
    --cors=*)
      value="${1#*=}"
      validate_bool "--cors" "$value"
      allowed_args+=("--cors=$value")
      shift
      ;;
    -c=*)
      value="${1#*=}"
      validate_bool "-c" "$value"
      allowed_args+=("--cors=$value")
      shift
      ;;

    -d|--delay-ms)
      require_value "$1" "${2-}"
      allowed_args+=("--delay-ms" "$2")
      shift 2
      ;;
    --delay-ms=*)
      value="${1#*=}"
      require_value "--delay-ms" "$value"
      allowed_args+=("--delay-ms=$value")
      shift
      ;;
    -d=*)
      value="${1#*=}"
      require_value "-d" "$value"
      allowed_args+=("--delay-ms=$value")
      shift
      ;;

    -o|--origin)
      require_value "$1" "${2-}"
      allowed_args+=("--origin" "$2")
      shift 2
      ;;
    --origin=*)
      value="${1#*=}"
      require_value "--origin" "$value"
      allowed_args+=("--origin=$value")
      shift
      ;;
    -o=*)
      value="${1#*=}"
      require_value "-o" "$value"
      allowed_args+=("--origin=$value")
      shift
      ;;

    -a|--admin-base-url)
      require_value "$1" "${2-}"
      allowed_args+=("--admin-base-url" "$2")
      shift 2
      ;;
    --admin-base-url=*)
      value="${1#*=}"
      require_value "--admin-base-url" "$value"
      allowed_args+=("--admin-base-url=$value")
      shift
      ;;
    -a=*)
      value="${1#*=}"
      require_value "-a" "$value"
      allowed_args+=("--admin-base-url=$value")
      shift
      ;;

    -l|--log-request)
      require_value "$1" "${2-}"
      validate_log_level "$1" "$2"
      allowed_args+=("--log-request" "$2")
      shift 2
      ;;
    --log-request=*)
      value="${1#*=}"
      validate_log_level "--log-request" "$value"
      allowed_args+=("--log-request=$value")
      shift
      ;;
    -l=*)
      value="${1#*=}"
      validate_log_level "-l" "$value"
      allowed_args+=("--log-request=$value")
      shift
      ;;

    -v|--verbosity)
      require_value "$1" "${2-}"
      validate_log_level "$1" "$2"
      allowed_args+=("--verbosity" "$2")
      shift 2
      ;;
    --verbosity=*)
      value="${1#*=}"
      validate_log_level "--verbosity" "$value"
      allowed_args+=("--verbosity=$value")
      shift
      ;;
    -v=*)
      value="${1#*=}"
      validate_log_level "-v" "$value"
      allowed_args+=("--verbosity=$value")
      shift
      ;;

    --preflight)
      require_value "$1" "${2-}"
      validate_preflight "$1" "$2"
      allowed_args+=("--preflight" "$2")
      shift 2
      ;;
    --preflight=*)
      value="${1#*=}"
      validate_preflight "--preflight" "$value"
      allowed_args+=("--preflight=$value")
      shift
      ;;

    --proxy-body-max-bytes)
      require_value "$1" "${2-}"
      validate_uint "$1" "$2"
      allowed_args+=("--proxy-body-max-bytes" "$2")
      shift 2
      ;;
    --proxy-body-max-bytes=*)
      value="${1#*=}"
      validate_uint "--proxy-body-max-bytes" "$value"
      allowed_args+=("--proxy-body-max-bytes=$value")
      shift
      ;;

    serve)
      error "subcommand 'serve' must not be passed explicitly: container already runs 'mockers serve'"
      ;;

    --host|-p|--port|-m|--mocks|-h|--help)
      error "option '$1' is not allowed in the container interface"
      ;;

    --host=*|--port=*|--mocks=*)
      error "option '${1%%=*}' is not allowed in the container interface"
      ;;

    -*|--*)
      error "unsupported option '$1'"
      ;;

    *)
      error "positional argument '$1' is not allowed: container only accepts whitelisted serve options"
      ;;
  esac
done

exec /usr/local/bin/mockers \
  serve \
  --host "$HOST" \
  --port "$PORT" \
  --mocks "$MOCKS_DIR" \
  "${allowed_args[@]}"
