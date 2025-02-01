# IP Request Counter

A simple HTTP server that tracks and displays real-time IP address statistics for incoming requests.

## Features

- Real-time IP address request counting
- Periodic statistics display (every second)
- RESTful endpoint `/ping`
- Graceful shutdown support
- Configurable logging levels

## Paths are tracked

- /ping

## Configuration

Environment variables (can be set via `.env` file):
- `HOST`: Server host (default: "0.0.0.0")
- `PORT`: Server port (default: "8081")
- `LOG_LEVEL`: Logging level (default: "INFO")

## Usage

1. Clone the repository
2. Set up environment variables (optional)
3. Run the server:
```bash
cargo run
```
or

1. Download [executable file](https://github.com/TOwInOK/tomoru-test/releases) for your system
  - **note** only for macos users!
  - ```sh
    xattr -rd com.apple.quarantine name_of_file
    ./name_of_file
    ```
2. launch `./name_of_file`

The server will display IP statistics every second, showing the number of requests from each IP address sorted by frequency.

## Endpoints

- `GET /ping`: Returns "pong" (health check endpoint)

## Showcase
![showcase](.content/showcase.webp)

## License

[MIT](Readme.md)
