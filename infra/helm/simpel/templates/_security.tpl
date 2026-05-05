{{/*
  Security context helpers — pod & container level.
  Baseline: runAsNonRoot, drop ALL capabilities, seccompProfile RuntimeDefault.
  Per-workload override via .Values.<svc>.podSecurityContext / containerSecurityContext.
*/}}

{{/* simpel.podSecurityContext — render PodSecurityContext fragment.
     Args: dict (workload values .Values.<svc>) */}}
{{- define "simpel.podSecurityContext" -}}
{{- $defaults := dict
    "runAsNonRoot" true
    "fsGroupChangePolicy" "OnRootMismatch"
    "seccompProfile" (dict "type" "RuntimeDefault")
-}}
{{- $merged := merge (default (dict) .podSecurityContext) $defaults -}}
securityContext:
  {{- toYaml $merged | nindent 2 }}
{{- end -}}

{{/* simpel.containerSecurityContext — render container SecurityContext fragment.
     Args: dict (workload values .Values.<svc>) */}}
{{- define "simpel.containerSecurityContext" -}}
{{- $defaults := dict
    "allowPrivilegeEscalation" false
    "privileged" false
    "readOnlyRootFilesystem" true
    "runAsNonRoot" true
    "capabilities" (dict "drop" (list "ALL"))
    "seccompProfile" (dict "type" "RuntimeDefault")
-}}
{{- $merged := merge (default (dict) .containerSecurityContext) $defaults -}}
securityContext:
  {{- toYaml $merged | nindent 2 }}
{{- end -}}
