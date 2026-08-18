{{/* Nama namespace object store. */}}
{{- define "backup.namespace" -}}
{{- .Values.namespace.name -}}
{{- end -}}

{{/* Label bersama. */}}
{{- define "backup.labels" -}}
app.kubernetes.io/part-of: simpel
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/* Endpoint S3 in-cluster yang dipakai Velero. */}}
{{- define "backup.s3Endpoint" -}}
{{- printf "http://minio.%s.svc.cluster.local:9000" (include "backup.namespace" .) -}}
{{- end -}}
