Production Istio routing files

This directory contains production overlay resources and the production Istio routing configuration. The Istio routing file should be applied separately (not via `kubectl apply -k`) because the routing resources must be created in `istio-system` namespace:

Command:

  kubectl apply -f infra/k8s/overlays/production/production-istio.yaml

Notes:
- The routing file creates `VirtualService` resources in `istio-system` that route to services in the `simpelv2-production` namespace via FQDNs.
- Keep this file under source control and update whenever routes change.
