{{/*
  Probe helpers — supports HTTP, gRPC, TCP, exec.
  Includes startup/liveness/readiness with sensible defaults.

  Usage examples:
    {{ include "simpel.probes" (dict "spec" .Values.authenc.probes) | nindent 8 }}

  Probe spec keys (any of):
    startup:    { type: http|grpc|tcp|exec, ... }
    liveness:   { ... }
    readiness:  { ... }

  Each probe item supports:
    type: http (default) | grpc | tcp | exec
    path:        HTTP only — required
    port:        port number or named port — required for http/grpc/tcp
    scheme:      HTTP | HTTPS (http only)
    httpHeaders: list of {name,value}
    service:     gRPC only — service name to check (defaults to "")
    command:     exec only — list of strings
    initialDelaySeconds, periodSeconds, timeoutSeconds, successThreshold, failureThreshold
*/}}

{{- define "simpel.probe" -}}
{{- $type := default "http" .type -}}
{{- if eq $type "http" -}}
httpGet:
  path: {{ .path }}
  port: {{ .port }}
  scheme: {{ default "HTTP" .scheme }}
  {{- with .httpHeaders }}
  httpHeaders:
    {{- toYaml . | nindent 4 }}
  {{- end }}
{{- else if eq $type "grpc" -}}
grpc:
  port: {{ .port }}
  {{- with .service }}
  service: {{ . | quote }}
  {{- end }}
{{- else if eq $type "tcp" -}}
tcpSocket:
  port: {{ .port }}
{{- else if eq $type "exec" -}}
exec:
  command:
    {{- range .command }}
    - {{ . | quote }}
    {{- end }}
{{- else -}}
{{- fail (printf "simpel.probe: unknown probe type %q" $type) -}}
{{- end }}
{{- with .initialDelaySeconds }}
initialDelaySeconds: {{ . }}
{{- end }}
{{- with .periodSeconds }}
periodSeconds: {{ . }}
{{- end }}
{{- with .timeoutSeconds }}
timeoutSeconds: {{ . }}
{{- end }}
{{- with .successThreshold }}
successThreshold: {{ . }}
{{- end }}
{{- with .failureThreshold }}
failureThreshold: {{ . }}
{{- end }}
{{- end -}}

{{/* Render full probes block (startup/liveness/readiness) for a container.
     Args: dict { spec: <probes> }
     Spec is .Values.<svc>.probes — keys: startup, liveness, readiness */}}
{{- define "simpel.probes" -}}
{{- $spec := .spec -}}
{{- with $spec.startup }}
startupProbe:
  {{- include "simpel.probe" . | nindent 2 }}
{{- end }}
{{- with $spec.liveness }}
livenessProbe:
  {{- include "simpel.probe" . | nindent 2 }}
{{- end }}
{{- with $spec.readiness }}
readinessProbe:
  {{- include "simpel.probe" . | nindent 2 }}
{{- end }}
{{- end -}}
