{{/*
  Topology spread constraints — distribusi pod merata antar node/zone.
  Default: maxSkew=1, topologyKey=kubernetes.io/hostname, whenUnsatisfiable=ScheduleAnyway.
  Per-workload override via .Values.<svc>.topologySpreadConstraints.

  Usage:
    {{- with include "simpel.topologySpreadConstraints" (dict "name" "authenc" "spec" .Values.authenc.topologySpreadConstraints) }}
    {{- . | nindent 6 }}
    {{- end }}
*/}}

{{- define "simpel.topologySpreadConstraints" -}}
{{- $name := .name -}}
{{- $spec := default (dict) .spec -}}
{{- if $spec.enabled -}}
topologySpreadConstraints:
  - maxSkew: {{ default 1 $spec.maxSkew }}
    topologyKey: {{ default "kubernetes.io/hostname" $spec.topologyKey }}
    whenUnsatisfiable: {{ default "ScheduleAnyway" $spec.whenUnsatisfiable }}
    labelSelector:
      matchLabels:
        app.kubernetes.io/name: {{ $name }}
{{- if $spec.extra }}
{{- range $spec.extra }}
  - {{ toYaml . | nindent 4 | trim }}
{{- end }}
{{- end }}
{{- end -}}
{{- end -}}

{{/*
  Pod anti-affinity (preferred or required) berdasarkan app.kubernetes.io/name.
  Args: dict { name: <svc>, type: preferred|required }
*/}}
{{- define "simpel.podAntiAffinity" -}}
{{- $name := .name -}}
{{- $type := default "preferred" .type -}}
{{- if eq $type "required" }}
podAntiAffinity:
  requiredDuringSchedulingIgnoredDuringExecution:
    - labelSelector:
        matchLabels:
          app.kubernetes.io/name: {{ $name }}
      topologyKey: kubernetes.io/hostname
{{- else if eq $type "preferred" }}
podAntiAffinity:
  preferredDuringSchedulingIgnoredDuringExecution:
    - weight: 100
      podAffinityTerm:
        labelSelector:
          matchLabels:
            app.kubernetes.io/name: {{ $name }}
        topologyKey: kubernetes.io/hostname
{{- end -}}
{{- end -}}
