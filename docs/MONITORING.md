# Monitoring

Substrate provides detailed metrics about your network's operations, these metrics can help to have insights about the number of peers connects, block finalized, peaks of transactions, among others.

The client presents telemetry in a format compatible with a Prometheus endpoint. This allows for its visualization through charts or integration with Grafana dashboards.

This guide focuses solely on configuring a local environment to monitor our development chain. For additional details, please refer to the provided link.

- https://docs.substrate.io/tutorials/build-a-blockchain/monitor-node-metrics/

## Quick start

To monitor your local client, we offer a `docker-compose` file that facilitates the setup of both `grafana` and `prometheus` services:

- Prometheus is an open-source tool tailored for monitoring and alerting in cloud-native ecosystems. It gathers and retains metrics, and features an API that can be tapped by other applications, such as dashboards.
- Grafana aids in visualizing the collected monitoring data. It offers tools to craft charts and tables, enhancing business intelligence operations.

For this last one, we also offer a pre-set dashboard that displays various metrics sourced from Substrate.

To begin with, we'll execute the `docker-compose` that houses the two previously mentioned services.

```bash
$ docker-compose -f docker/monitoring/docker-compose.yml
```

After it's up and running, proceed to launch the client with the `--prometheus-external` flag activated. The client metrics can be viewed at: http://localhost:9165/metrics

At this point, we're all set to begin monitoring the node. To do this, access Grafana locally via: http://localhost:3000.
The username and password are both set to `admin`.

You can locate the pre-configured dashboard by navigating to the left menu and selecting **Dashboards > General**.

![screenshot](https://docs.substrate.io/static/0e72182eb3c2bcb4c4d4112729e9b39a/1c7bd/grafana-template-dashboard.avif)
