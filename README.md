# Bento Alephium Indexer

This project is a Rust-based application that uses Diesel for database migrations and interacts with a PostgreSQL database. The application is containerized using Docker and can be managed using Docker Compose and a Taskfile for convenience.

## Prerequisites

- Docker
- Docker Compose
- [Task](https://taskfile.dev/) (a task runner for executing tasks defined in `Taskfile.yml`)

## DevOps (Local Development)

- **Dockerfile**: Defines the Docker image for the Rust application.
- **docker-compose.yml**: Configures the services, including the PostgreSQL database and the Rust application.
- **Taskfile.yml**: Provides a set of tasks for managing the application lifecycle.

### Getting Started

Copy the `.env.example` file to `.env` and fill in the values.
To build and start the application the first time, use the following command:

```sh
task genesis
```

To continue development, use the following command:

```sh
task up
```

You can find more details on the *Taskfile.yml* file.

Remember the docker-compose has support to hot-reload so if you change the source code, the application will be reloaded.
