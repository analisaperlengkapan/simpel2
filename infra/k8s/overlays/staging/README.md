Staging Istio routing files

This directory contains staging overlay resources and the staging Istio routing configuration. The Istio routing file should be applied separately (not via `kubectl apply -k`) because the routing resources must be created in `istio-system` namespace:

Command:

  kubectl apply -f infra/k8s/overlays/staging/staging-istio.yaml

Notes:
- The routing file creates `VirtualService` resources in `istio-system` that route to services in the `simpelv2-staging` namespace via FQDNs or IP hostnames (10.1.7.121).
- Keep this file under source control and update whenever routes change.
