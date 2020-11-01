# Database 2 client project
  Built using Elm for GUI creation and Rust as computation and IO manager, grouped together as Key-Api pairs.

## Api's
    
  An Api is the processing of a message into a response, both
defined by a previous protocol. Such api may include File 
management, TCP/UDP Socket manipulation, etc.

## Client/Server communication.
  
  Using Elm's built in ports, one is able to communicate with Rust's process over requests (very similar to JsonRPC protocol). 

  The request structure is as follow:

    {
        "api_name" [String]: The name of the api to invoke,
        "payload" [Object][^1]: The message that the Api must process
    }

  The response structure must be:

    {
        "api_name" [String]: The name of the api that processed the request that this response responds to.
        "error" [Boolean]: This response represents an error.
        "body" [Object]: The message processing's result
    }
    
[^1]: Must be in JSON notation

## Asynchronous dispatch 
    
  Every request is processed and dispatched in an asynchronous
manner, using tokio's multithreded runtime that is built on
top of "Tasks". Each request is processed in one of these tasks in a concurrent manner.

  Each processing follows theese steps:
  
  - The message is received and validated.
  - If the message is valid, it's `api_name` field will be
  extracted, otherwise, the process dies here and no response
  is issued.
  - With the `api_name` extracted from the request, it will be
  matched against the registered api's keys, if none is found,
  the process dies here and no response is issued.
  - The api dispatcher is build around the registered 
  middlewares, linked in a consecutive chain call, ending with
  the call to the found api
