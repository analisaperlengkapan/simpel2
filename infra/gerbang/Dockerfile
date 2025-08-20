FROM envoyproxy/envoy:v1.28-latest

# Copy envoy configuration
COPY envoy.yaml /etc/envoy/envoy.yaml

# Expose ports
EXPOSE 8080 9901

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:9901/stats || exit 1

# Run Envoy
CMD ["/usr/local/bin/envoy", "-c", "/etc/envoy/envoy.yaml"] 