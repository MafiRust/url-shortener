# URL Shortener

## Development

Run with `cargo run --features dotenv`.

Start database with:

```shell
docker build -t url-shortener-db:latest database
docker run --rm --env POSTGRES_PASSWORD=password url-shortener-db:latest
```
