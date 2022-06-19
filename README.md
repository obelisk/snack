# snack
A simple Slack command router for fronting various services.

## Intention
Several times I've ended up writing a few services that need to accept Slack commands or just have services that I don't want to expose directly to the internet. Snack is a small and configurable proxy for Slack commands that will verify the requests then route them to the configured backend service.

## Request
The request is passed as is so the backend service can choose to reverify if they'd like. The only thing that Snack changes is it will remove the first path component of an incoming request. The idea behind this is that it's only needed for routing purposes and then can be removed so Snack is transparent to any proxied services.

### Example
Let's say Snack is configured to front `service1` with the backend being available at `192.168.100.40:8329`. A Slack command might be set up to post to:
```
https://snack.some.domain/service1/slack_call
```
Snack (once it validates the request) will make a call to:
```
http://192.168.100.40:8329/slack_call
```
