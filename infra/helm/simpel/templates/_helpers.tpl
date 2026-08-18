{{/*
  SIMPEL Helm chart helpers.

  Naming convention:
    simpel.<area>.<thing>     → identifier or value
    simpel.<area>.tpl         → fragment that renders YAML
*/}}

{{/* ────────── Names & namespaces ────────── */}}
{{- define "simpel.namespace" -}}
{{- default .Release.Namespace .Values.namespace.name -}}
{{- end -}}

{{- define "simpel.fullname" -}}
{{- $name := default .Chart.Name .Values.fullnameOverride -}}
{{- $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "simpel.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/* ────────── Common labels & selector labels ────────── */}}
{{- define "simpel.labels" -}}
helm.sh/chart: {{ include "simpel.chart" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/environment: {{ .Values.global.environment }}
{{- range $k, $v := .Values.global.labels }}
{{ $k }}: {{ $v | quote }}
{{- end }}
{{- end -}}

{{/* Component-specific labels: pass dict with `name` and `component`. */}}
{{- define "simpel.componentLabels" -}}
{{- $top := .top -}}
app.kubernetes.io/name: {{ .name }}
app.kubernetes.io/component: {{ .component }}
{{ include "simpel.labels" $top }}
{{- end -}}

{{- define "simpel.selectorLabels" -}}
app.kubernetes.io/name: {{ .name }}
app.kubernetes.io/instance: {{ .top.Release.Name }}
{{- end -}}

{{/* ────────── Image reference ──────────
     Usage: {{ include "simpel.image" (dict "top" $ "spec" .Values.authenc.image) }}
     Spec keys: name, tag (optional), repository (optional, full path), pullPolicy (optional).
*/}}
{{- define "simpel.image" -}}
{{- $top := .top -}}
{{- $spec := .spec -}}
{{- $registry := $top.Values.global.registry -}}
{{- $tag := default $top.Values.global.imageTag $spec.tag -}}
{{- $prefix := default "simpelv2-" $top.Values.global.imageNamePrefix -}}
{{- if $spec.repository -}}
{{- printf "%s:%s" $spec.repository $tag -}}
{{- else if $spec.name -}}
{{- /* Pakai prefix kalau spec.name tidak include "/" (full path override) */ -}}
{{- if contains "/" $spec.name -}}
{{- printf "%s/%s:%s" $registry $spec.name $tag -}}
{{- else -}}
{{- printf "%s/%s%s:%s" $registry $prefix $spec.name $tag -}}
{{- end -}}
{{- else -}}
{{- fail (printf "image spec missing both repository and name: %v" $spec) -}}
{{- end -}}
{{- end -}}

{{/* ────────── Klien postgres (psql/pg_isready) ────────── */}}
{{- /*
  Image klien untuk job yang HANYA menjalankan `psql`/`pg_isready` terhadap
  server postgres chart ini.

  Diturunkan dari `.Values.postgres.image` — sumber yang sama dengan server —
  supaya versi klien tak bisa menyimpang darinya. Sebelum 2026-08-18 empat
  template menuliskan `postgres:15-alpine` sebagai literal sementara server-nya
  `16-alpine`; skew itu tak menggigit karena keempatnya hanya `psql`/
  `pg_isready` (yang toleran lintas versi), tapi ia adalah pola kegagalan yang
  berulang di repo ini: satu nilai punya SATU sumber untuk komponen utamanya
  dan N salinan tulis-tangan di tempat lain, lalu yang tertinggal baru
  ketahuan saat sudah telat. `db-create-job.yaml` bahkan sempat memuat komentar
  yang menyebut `16-alpine` tepat di atas literal `15-alpine`-nya sendiri.

  Catatan yang membuat ini lebih dari kerapian: begitu ada job yang memakai
  `pg_dump`, skew mulai FATAL — pg_dump menolak berjalan bila server lebih baru
  daripada dirinya. Menurunkannya sekarang menutup pintu itu sebelum dibuka.
*/ -}}
{{- define "simpel.postgresClientImage" -}}
{{- $img := .Values.postgres.image -}}
{{- printf "%s:%s" (required "postgres.image.repository wajib diisi" $img.repository) (required "postgres.image.tag wajib diisi" $img.tag) -}}
{{- end -}}

{{/* ────────── Pull policy & secrets ────────── */}}
{{- define "simpel.imagePullPolicy" -}}
{{- default "IfNotPresent" .Values.global.imagePullPolicy -}}
{{- end -}}

{{- define "simpel.imagePullSecrets" -}}
{{- with .Values.global.imagePullSecrets }}
imagePullSecrets:
  {{- range . }}
  - name: {{ . }}
  {{- end }}
{{- end }}
{{- end -}}

{{/* ────────── Common scheduling / pod metadata ────────── */}}
{{- define "simpel.nodeSelector" -}}
{{- with .Values.global.nodeSelector }}
nodeSelector:
  {{- toYaml . | nindent 2 }}
{{- end }}
{{- end -}}

{{- define "simpel.tolerations" -}}
{{- with .Values.global.tolerations }}
tolerations:
  {{- toYaml . | nindent 2 }}
{{- end }}
{{- end -}}

{{/* ────────── Service URLs (defaults derived from namespace) ────────── */}}
{{- define "simpel.postgresHost" -}}
{{- default (printf "postgres.%s.svc.cluster.local" (include "simpel.namespace" .)) .Values.config.postgresHost -}}
{{- end -}}

{{- define "simpel.redisUrl" -}}
{{- default (printf "redis://redis-service.%s.svc.cluster.local:6379" (include "simpel.namespace" .)) .Values.config.redisUrl -}}
{{- end -}}

{{- define "simpel.authencGrpcUrl" -}}
{{- default (printf "http://authenc.%s.svc.cluster.local:9088" (include "simpel.namespace" .)) .Values.backendConfig.authencGrpcUrl -}}
{{- end -}}

{{- define "simpel.secretonGrpcUrl" -}}
{{- default (printf "http://secreton.%s.svc.cluster.local:9000" (include "simpel.namespace" .)) .Values.backendConfig.secretonGrpcUrl -}}
{{- end -}}

{{- define "simpel.secretonEndpoint" -}}
{{- default (printf "http://secreton.%s.svc.cluster.local:8200" (include "simpel.namespace" .)) .Values.backendConfig.secretonEndpoint -}}
{{- end -}}

{{- define "simpel.integrasiGrpcUrl" -}}
{{- default (printf "http://layanan-integrasi.%s.svc.cluster.local:50051" (include "simpel.namespace" .)) .Values.backendConfig.integrasiGrpcUrl -}}
{{- end -}}

{{- define "simpel.simpelv1DbHost" -}}
{{- $env := .Values.simpelv1.env -}}
{{- default (printf "postgres.%s.svc.cluster.local" (include "simpel.namespace" .)) $env.DB_HOST -}}
{{- end -}}

{{- define "simpel.simpelv1RedisHost" -}}
{{- $env := .Values.simpelv1.env -}}
{{- default (printf "redis.%s.svc.cluster.local" (include "simpel.namespace" .)) $env.REDIS_HOST -}}
{{- end -}}

{{- define "simpel.mtlsMode" -}}
{{- default .Values.mtls.mode .Values.config.mtlsMode -}}
{{- end -}}

{{/* ────────── Standard probes ──────────
     Generate HTTP probe from dict: scheme, path, port, initialDelay, period, timeout, failureThreshold, successThreshold.
*/}}
{{- define "simpel.httpProbe" -}}
httpGet:
  path: {{ .path }}
  port: {{ .port }}
  scheme: {{ default "HTTP" .scheme }}
{{- if .initialDelay }}
initialDelaySeconds: {{ .initialDelay }}
{{- end }}
{{- if .period }}
periodSeconds: {{ .period }}
{{- end }}
{{- if .timeout }}
timeoutSeconds: {{ .timeout }}
{{- end }}
{{- if .successThreshold }}
successThreshold: {{ .successThreshold }}
{{- end }}
{{- if .failureThreshold }}
failureThreshold: {{ .failureThreshold }}
{{- end }}
{{- end -}}

{{/* ────────── Istio sidecar annotations for a workload ────────── */}}
{{- define "simpel.istioPodAnnotations" -}}
{{- $cfg := . -}}
{{- if hasKey $cfg "istioInjection" }}
sidecar.istio.io/inject: {{ ternary "\"true\"" "\"false\"" $cfg.istioInjection }}
{{- end -}}
{{- if and (hasKey $cfg "istioInjection") $cfg.istioInjection (hasKey $cfg "istioProxyResources") }}
{{- with $cfg.istioProxyResources }}
sidecar.istio.io/proxyCPU: {{ (default (dict) .cpu).request | default "" | quote }}
sidecar.istio.io/proxyCPULimit: {{ (default (dict) .cpu).limit | default "" | quote }}
sidecar.istio.io/proxyMemory: {{ (default (dict) .memory).request | default "" | quote }}
sidecar.istio.io/proxyMemoryLimit: {{ (default (dict) .memory).limit | default "" | quote }}
{{- end }}
{{- end -}}
{{- if and (hasKey $cfg "istioExcludeOutboundPorts") $cfg.istioExcludeOutboundPorts }}
traffic.sidecar.istio.io/excludeOutboundPorts: {{ $cfg.istioExcludeOutboundPorts | quote }}
{{- end -}}
{{- end -}}
