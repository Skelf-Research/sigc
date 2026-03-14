# Exit Codes Reference

sigc exit codes and their meanings.

## Success Codes

### 0 - Success

Command completed successfully.

```bash
sigc run strategy.sig
echo $?  # 0
```

## Error Codes

### 1 - General Error

Unspecified error.

```bash
sigc run nonexistent.sig
# Error: File not found
echo $?  # 1
```

### 2 - Parse Error

Syntax error in sigc file.

```bash
sigc run bad_syntax.sig
# Error E001: Unexpected token
echo $?  # 2
```

### 3 - Type Error

Type mismatch or shape error.

```bash
sigc run type_error.sig
# Error E100: Type mismatch
echo $?  # 3
```

### 4 - Data Error

Problem loading or processing data.

```bash
sigc run strategy.sig
# Error E200: File not found
echo $?  # 4
```

### 5 - Runtime Error

Error during execution.

```bash
sigc run strategy.sig
# Error E300: Division by zero
echo $?  # 5
```

### 6 - Configuration Error

Invalid configuration.

```bash
sigc run strategy.sig --config bad.yaml
# Error E400: Invalid YAML
echo $?  # 6
```

### 7 - Broker Error

Problem with broker connection or orders.

```bash
sigc daemon start --live
# Error E600: Connection failed
echo $?  # 7
```

### 8 - Safety Error

Safety system triggered.

```bash
# Circuit breaker activated
echo $?  # 8
```

## Daemon Exit Codes

### 10 - Daemon Already Running

```bash
sigc daemon start
# Error: Daemon already running
echo $?  # 10
```

### 11 - Daemon Not Running

```bash
sigc daemon stop
# Error: Daemon not running
echo $?  # 11
```

### 12 - Daemon Failed to Start

```bash
sigc daemon start
# Error: Failed to start daemon
echo $?  # 12
```

## Signal Codes

### 130 - Interrupted (SIGINT)

User pressed Ctrl+C.

```bash
sigc run strategy.sig
# ^C
echo $?  # 130
```

### 137 - Killed (SIGKILL)

Process was forcefully killed.

### 143 - Terminated (SIGTERM)

Process received termination signal.

## Using Exit Codes

### In Shell Scripts

```bash
#!/bin/bash

sigc run strategy.sig
exit_code=$?

case $exit_code in
  0) echo "Success" ;;
  2) echo "Parse error - check syntax" ;;
  4) echo "Data error - check data files" ;;
  7) echo "Broker error - check connection" ;;
  *) echo "Error code: $exit_code" ;;
esac
```

### In CI/CD

```yaml
# GitHub Actions
- name: Run Backtest
  run: |
    sigc run strategy.sig
  continue-on-error: false
```

### With Monitoring

```bash
# Send alert on non-zero exit
sigc daemon start || send_alert "Daemon failed to start: $?"
```

## Exit Code Categories

| Range | Category |
|-------|----------|
| 0 | Success |
| 1-9 | General errors |
| 10-19 | Daemon errors |
| 20-29 | Broker errors |
| 30-39 | Safety errors |
| 128+ | Signal-based exits |

## Debugging

Get detailed error information:

```bash
SIGC_LOG_LEVEL=debug sigc run strategy.sig
```

Or check logs:

```bash
cat /var/log/sigc/strategy.log
```

## See Also

- [Error Messages](error-messages.md)
- [CLI Reference](cli.md)
