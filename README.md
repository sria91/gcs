# GCS - Ground Control Station with command-line and REST interfaces.

## Setting up the development environment container
* Install Docker for your operating system from [here](https://www.docker.com/get-started).
* Install VS Code editor from [here](https://code.visualstudio.com/Download) and launch it.
* Install the `Remote - Containers` VS Code extension from [here](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers).
* Open this folder in VS Code.
* When prompted, click on `Reopen in Container`.
* It will take some time to download and build the Docker container.
* Once it's done you will be able to use the editor.

## Building this software
In order to build this software run: `cargo build --release`.

## Testing this software

### Testing the REST API
* Install Node.js from [here](https://nodejs.org/en/download/).
* Open a terminal and install Newman: `npm install -g newman`.
* Start the REST API server in one terminal: `cargo run --release --bin gcs_api`.
* In another terminal run:
    * Windows: `newman run .\tests\gcs.postman_collection.json`
    * Linux: `newman run ./tests/gcs.postman_collection.json`

## Running this software

### Running the CLI
`gcs_cli` allows the user to specify the input either from a text file or standard input.
* In order to run the GCS by using the input text file use: `cargo run --release -- --input <path to input text file>`. If you don't pass the `--input` option the cli will start reading from standard input.
* The output is always printed on the console. The user can also specify the output text file by using `--output <path to output text file>`.
* The user can list all the plateaus available in the database using: `cargo run --release -- --list-plateaus`.
* The user can load an existing plateau from the database using: `cargo run --release -- --plateau <plateau id>`.
* Loading a plateau also loads all its rovers and their last pose. The user can then provide motion commands to continue their movement.

### Running the REST API server
`gcs_api` starts a REST API server listening on port 9090.
* In order to start the GCS REST API server use: `cargo run --release --bin gcs_api`.
* Creating a plateau with bounds: `curl -X POST -d '{"x_max": 5, "y_max": 5}' -H "Content-type: application/json" http://localhost:9090/plateaus`.
* Listing the available plateaus: `curl -X GET -H "Content-type: application/json" http://localhost:9090/plateaus`.
* Creating a rover with initial pose: `curl -X POST -d '{"x": 1, "y": 2, "facing": "North"}' -H "Content-type: application/json" http://localhost:9090/plateaus/{plateau_id}/rovers`.
* Listing the available rovers: `curl -X GET -H "Content-type: application/json" http://localhost:9090/plateaus/{plateau_id}/rovers`.
* Moving the rover: `curl -X PATCH -H "Content-type: application/json" http://localhost:9090/plateaus/{plateau_id}/rovers/{rover_id}/{motion_command}`.
