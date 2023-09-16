# moker
A Simple Mock REST Client for Everyone

## Main Objective 
Moker allows you to easily run a mock server using JSON or YAML files.

## Features
- [x] Supports .json files 
- [x] Supports headers (guard)
- [x] Supports all HTTP methods
- [x] Supports URL query parameters
- [ ] Supports body parameters (guard)
- [x] Configurable through the command line
- [x] Network logger
- [x] File watcher
- [ ] Supports Swagger & Postman collection
- [ ] Supports socket
- [x] Supports .yaml files 
- [ ] Distribution via binary 
- [ ] Distribution via Homebrew

## Usage
1. Clone the repository 
2. Use the `cargo run` command

To run the sample JSON files, use:

```bash
cargo run -- -s ./example/json -p 8000
```

### Flags 
- `-s`: Path to look for config files. Default is the root of the project (".")
- `-p`: Port to run the mock server on. Default is 8080

### JSON Configuration for Route
- `"name"`: An optional name for this configuration, used for logging purposes.
- `"method"`: Defines the allowed HTTP method/s for this URL. If set, only requests with specified methods will be processed; others will receive a "method not implemented" response. Can be a list of methods (e.g., `["get", "post"]`). Ignore if you want to allow any HTTP method.
- `"headers"`: Define a dictionary/map of headers. The response will be sent only if the request contains these headers.

### Response Configuration
Within the JSON body, use the `"response"` parameter to specify the following values:
- `"headers"`: Define a dictionary/map of headers to be included in the HTTP response.
- `"status_code"`: Set the HTTP response code.
- `"body"`: Provide the JSON or raw string you want to be returned in the HTTP response.
- `"delay_ms"`: Add a delay to the response in milliseconds.
