# Redis Protocol Bridge

This is a short proof of concept showing how we can use the redis-protocol crate
to provide an API endpoint a redis client can connect to.

The binary creates an TCP server using tokios `TcpListener` and handles request asynchronously.
The TCP server listens on Redis' standard port 6379, so redis clients connect to it automatically.

## Testing

For testing, 'install' the redis-cli:

```bash
wget http://download.redis.io/redis-stable.tar.gz
tar xvzf redis-stable.tar.gz
cd redis-stable
make redis-cli
sudo cp src/redis-cli /usr/local/bin/
```
