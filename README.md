# Archive Tool

A command line tool for archiving websites for NYCU Web Archive.

## Installation

You can download the latest release from the [release page](https://github.com/GymSquad/backup-script/releases/).

## Usage

```
Usage: archive-tool [CONFIG]

Arguments:
  [CONFIG]  Custom config file location [default: archive.toml]

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Configuration

The configuration file is written in [TOML](https://toml.io/) format.

See [archive.toml](archive.toml) for an example.

### Section: `database`

The `database` section is used to configure the database connection.

| Key   | Type     | Description                  |
| ----- | -------- | ---------------------------- |
| `url` | `String` | The database connection URL. |

### Section: `runner`

The `runner` section is used to configure the runner.

| Key           | Type      | Description                                                                                                                   |
| ------------- | --------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `num-threads` | `Integer` | Optional. The number of threads to use. (Default: no limit)                                                                   |
| `num-workers` | `Integer` | Optional. The number of workers (archiving process) that can be spawned at the same time. (Default: Number of physical cores) |
| `num-urls`    | `Integer` | Optional. The number of websites to archive. Useful for testing purposes. (Default: Number of websites)                       |

### Section: `archive`

The `archive` section is used to configure the archiving process.

| Key          | Type       | Description                                     |
| ------------ | ---------- | ----------------------------------------------- |
| `output-dir` | `String`   | The output directory for the archived websites. |
| `command`    | `String[]` | The command to run for archiving.               |
