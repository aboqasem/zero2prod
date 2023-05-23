# Deploying to DigitalOcean

## Deploy

```bash
doctl apps create --spec ./deploy/digitalocean/spec.yml
```

## Update deployment

```bash
doctl apps update <APP_ID> --spec ./deploy/digitalocean/spec.yml
```
