# mocker
A simple mock REST client for everyone

## Main Objective 
Run a mock server by using .json/.yaml files with configurations.

## Features
- [x] Support for .json files 
- [ ] Support for headers (guard)
- [ ] Support for all http methods
- [ ] Support for url query parameters (guard)
- [ ] Support for body parameters (guard)
- [ ] Add configuration using command line
- [X] Network logger
- [ ] File Watcher
- [ ] Support swagger & postman collection
- [ ] Support socket
- [ ] Support for .yaml files 
- [ ] Binary for running the tool 
- [ ] Distribution using homebrew

## Usage
Clone the repository 
use `cargo run` command

### use flags 
--s for path to look for config files. Default is the root of the project "."
--p for port to run the mock server on. default is 8080

### JSON configuration
- "name" : An optional name to this configuration. Only used for logging.
- "method" : Defines what method should this url work on. ex: if this value is "GET" then only a GET request will be processed, rest will be replied with "Method not Implemented".
- "methods" : Same as "method" except you can use multiple method to map this URL to.  
- "request_headers" : Use a dictionary/map to to return a response only when the request contains these headers(keys for now).
- "response_headers" : Use a dictionary/map to get these values in headers of incoming http response.
- "response_code" : The response code to set set for response.
- "response_content_type": Set the content_type for incoming http response.
- "response" : Use the JSON or raw string you want to be returned in the incoming http response
- "response_delay_ms": Add a delay to the response in milli seconds.
