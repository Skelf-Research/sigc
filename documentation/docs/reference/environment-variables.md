# Environment Variables Reference

Environment variables for configuring sigc.

## Core Variables

### SIGC_HOME

sigc home directory for configuration and cache.

```bash
export SIGC_HOME="/home/user/.sigc"
```

Default: `~/.sigc`

### SIGC_CONFIG

Path to default configuration file.

```bash
export SIGC_CONFIG="/path/to/config.yaml"
```

### SIGC_LOG_LEVEL

Logging verbosity.

```bash
export SIGC_LOG_LEVEL="debug"  # debug, info, warn, error
```

Default: `info`

## Data Variables

### SIGC_DATA_DIR

Default directory for data files.

```bash
export SIGC_DATA_DIR="/data/sigc"
```

### AWS Credentials

For S3 data access:

```bash
export AWS_ACCESS_KEY_ID="your_access_key"
export AWS_SECRET_ACCESS_KEY="your_secret_key"
export AWS_REGION="us-east-1"
```

### Database

For PostgreSQL connections:

```bash
export SIGC_DB_HOST="localhost"
export SIGC_DB_PORT="5432"
export SIGC_DB_USER="sigc"
export SIGC_DB_PASSWORD="password"
export SIGC_DB_NAME="trading"
```

## Broker Variables

### Alpaca

```bash
export ALPACA_API_KEY="your_api_key"
export ALPACA_API_SECRET="your_api_secret"
export ALPACA_BASE_URL="https://api.alpaca.markets"
```

For paper trading:
```bash
export ALPACA_BASE_URL="https://paper-api.alpaca.markets"
```

## Alerting Variables

### Slack

```bash
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/..."
```

### PagerDuty

```bash
export PAGERDUTY_SERVICE_KEY="your_service_key"
```

### Email

```bash
export SMTP_HOST="smtp.gmail.com"
export SMTP_PORT="587"
export SMTP_USER="alerts@example.com"
export SMTP_PASSWORD="app_password"
```

## Performance Variables

### SIGC_WORKERS

Number of parallel workers.

```bash
export SIGC_WORKERS="8"
```

Default: Number of CPU cores

### SIGC_MAX_MEMORY

Maximum memory usage in GB.

```bash
export SIGC_MAX_MEMORY="16"
```

### RUST_LOG

Detailed Rust logging for debugging.

```bash
export RUST_LOG="sigc=debug,sig_runtime=trace"
```

## Daemon Variables

### SIGC_PID_FILE

PID file location for daemon.

```bash
export SIGC_PID_FILE="/var/run/sigc/strategy.pid"
```

### SIGC_LOG_FILE

Log file location for daemon.

```bash
export SIGC_LOG_FILE="/var/log/sigc/strategy.log"
```

## Development Variables

### SIGC_DEV_MODE

Enable development mode with extra checks.

```bash
export SIGC_DEV_MODE="1"
```

### SIGC_PROFILE

Enable profiling.

```bash
export SIGC_PROFILE="1"
```

## Using in Configuration

Reference environment variables in config files:

```yaml
broker:
  credentials:
    api_key: ${ALPACA_API_KEY}
    api_secret: ${ALPACA_API_SECRET}
```

## Loading from File

Create a `.env` file:

```bash
# .env
ALPACA_API_KEY=your_key
ALPACA_API_SECRET=your_secret
SIGC_LOG_LEVEL=debug
```

Load before running:
```bash
source .env
sigc run strategy.sig
```

## Precedence

1. CLI arguments
2. Environment variables
3. Configuration file values
4. Default values

## Security Notes

- Never commit secrets to version control
- Use secret managers in production
- Rotate credentials regularly
- Use separate credentials for paper vs live

## See Also

- [Configuration Reference](configuration.md)
- [CLI Reference](cli.md)
