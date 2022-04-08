# BlockVisor API

This is the backend for the BlockVisor infrastructure platform.

## API Documentation

API Documentation should be updated and maintained at https://blockjoy.stoplight.io

## Configuration

Configuration can be set via `toml` config file in the `config` directory. The default mode is `development` but that can be overridden by setting `APP_ENV` to something like `production` or `staging`.

### Configuration Order

Config is processed as follows. Each subsequent file or env var overrides any configuration settings of the previous:

1. `config/Default.toml`
2. `config/development.toml` or `config/production.toml` or `config/staging.toml` depending on the `APP_ENV` environment variable setting. Defaults to `config/development.toml`.
3. `config/Local.toml`
4. Individual config parameters can be overridden via env vars with the prefix `BLOCLVISOR`, e.g. (`BLOCKVISOR_DATABASE_URL`).
