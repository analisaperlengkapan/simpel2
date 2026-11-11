{{/*
  CronJob helper — dipakai oleh layanan-integrasi (per provider) atau workload batch lainnya.

  Usage:
    {{ include "simpel.cronjob" (dict "ctx" . "name" "layanan-integrasi-mysimkari" "values" $values "defaults" .Values.layananIntegrasi.cronjobDefaults) }}

  values struct:
    enabled, suspend, schedule, timeZone, command, args
    image (optional, fallback ke parent values.image)
    env, envFrom, resources, secretonAuth, ...
    inherits container/pod security context dari values.parent
*/}}
{{- define "simpel.cronjob" -}}
{{- $ctx := .ctx -}}
{{- $name := .name -}}
{{- $values := .values -}}
{{- $defaults := default (dict) .defaults -}}
{{- /* Pod Job TIDAK boleh membawa sidecar Istio kecuali diminta eksplisit (istioInjection: true).
       Terukur di rehearsal produksi: tanpa anotasi, percobaan PERTAMA keluar dengan kode 1 dalam 3 ms
       (`Failed to connect to database: error connecting to server`, Envoy belum mendengar); restart
       OnFailure berikutnya berhasil tetapi `backoffLimit: 1` sudah habis, jadi Job tercatat GAGAL
       walau pekerjaannya selesai — `lastSuccessfulTime` di staging kosong sejak awal. Sidecar juga
       membuat pod Job tak pernah selesai setelah aplikasinya keluar. Job lain di chart ini
       (db-create, *-migrate, authenc-bootstrap) sudah `sidecar.istio.io/inject: "false"`. */ -}}
{{- $podValues := mergeOverwrite (deepCopy $values) (dict "istioInjection" (ternary $values.istioInjection false (hasKey $values "istioInjection"))) -}}
{{- if $values.enabled }}
apiVersion: batch/v1
kind: CronJob
metadata:
  name: {{ $name }}
  namespace: {{ include "simpel.namespace" $ctx }}
  labels:
    {{- include "simpel.componentLabels" (dict "top" $ctx "name" $name "component" (default $name $values.component)) | nindent 4 }}
spec:
  schedule: {{ $values.schedule | quote }}
  {{- with $values.timeZone }}
  timeZone: {{ . | quote }}
  {{- end }}
  suspend: {{ default false $values.suspend }}
  successfulJobsHistoryLimit: {{ default 3 (default $defaults.successfulJobsHistoryLimit $values.successfulJobsHistoryLimit) }}
  failedJobsHistoryLimit: {{ default 1 (default $defaults.failedJobsHistoryLimit $values.failedJobsHistoryLimit) }}
  concurrencyPolicy: {{ default "Forbid" (default $defaults.concurrencyPolicy $values.concurrencyPolicy) }}
  {{- with default $defaults.startingDeadlineSeconds $values.startingDeadlineSeconds }}
  startingDeadlineSeconds: {{ . }}
  {{- end }}
  jobTemplate:
    spec:
      backoffLimit: {{ default 1 (default $defaults.backoffLimit $values.backoffLimit) }}
      {{- with default $defaults.activeDeadlineSeconds $values.activeDeadlineSeconds }}
      activeDeadlineSeconds: {{ . }}
      {{- end }}
      template:
        metadata:
          labels:
            {{- include "simpel.selectorLabels" (dict "top" $ctx "name" $name) | nindent 12 }}
            app.kubernetes.io/component: {{ default $name $values.component }}
          annotations:
            {{- include "simpel.istioPodAnnotations" $podValues | nindent 12 }}
        spec:
          serviceAccountName: {{ default $name $values.serviceAccountName }}
          automountServiceAccountToken: {{ default false $values.automountServiceAccountToken }}
          {{- include "simpel.imagePullSecrets" $ctx | nindent 10 }}
          {{- /* `Never`, dan sengaja BUKAN nilai yang bisa diatur. Dengan `OnFailure` container diulang
                 di dalam pod yang sama; begitu `backoffLimit` habis, Job controller MENGHAPUS pod itu
                 (perilaku terdokumentasi Kubernetes) dan log-nya ikut hilang. Terukur di rehearsal
                 produksi rc47 (2026-11-11): tiga sinkronisasi gagal berturut-turut, lalu `kubectl logs`
                 menjawab "No resources found" — satu-satunya jejak adalah `BackoffLimitExceeded` pada
                 Job; klaster ini tak punya pengumpul log. Dengan `Never` tiap percobaan adalah pod
                 sendiri yang tetap ada (fase Failed, tak memakai kuota) selama Job-nya disimpan
                 `failedJobsHistoryLimit`. Jumlah percobaan tak berubah: backoffLimit + 1.
                 Dijaga `infra/scripts/check-failed-job-keeps-its-log.py`. */}}
          restartPolicy: Never
          {{- with default $ctx.Values.global.priorityClassName $values.priorityClassName }}
          priorityClassName: {{ . }}
          {{- end }}
          {{- include "simpel.podSecurityContext" $values | nindent 10 }}
          {{- with default $ctx.Values.global.nodeSelector $values.nodeSelector }}
          nodeSelector:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with default $ctx.Values.global.tolerations $values.tolerations }}
          tolerations:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          containers:
            - name: {{ $name }}
              image: {{ include "simpel.image" (dict "top" $ctx "spec" $values.image) }}
              imagePullPolicy: {{ default (include "simpel.imagePullPolicy" $ctx) $values.image.pullPolicy }}
              {{- with $values.command }}
              command:
                {{- toYaml . | nindent 16 }}
              {{- end }}
              {{- with $values.args }}
              args:
                {{- toYaml . | nindent 16 }}
              {{- end }}
              {{- with $values.envFrom }}
              envFrom:
                {{- toYaml . | nindent 16 }}
              {{- end }}
              {{- if or $values.env (and $values.secretonAuth $values.secretonAuth.enabled) }}
              env:
                {{- with $values.env }}
                {{- toYaml . | nindent 16 }}
                {{- end }}
                {{- if and $values.secretonAuth $values.secretonAuth.enabled }}
                - name: SECRETON_ADDR
                  value: {{ default (include "simpel.secretonEndpoint" $ctx) $values.secretonAuth.addr | quote }}
                - name: SECRETON_AUTH_METHOD
                  value: kubernetes
                - name: SECRETON_AUTH_ROLE
                  value: {{ default $name $values.secretonAuth.role | quote }}
                - name: SECRETON_K8S_TOKEN_PATH
                  value: {{ default "/var/run/secrets/tokens/secreton-token" $values.secretonAuth.tokenPath | quote }}
                {{- end }}
              {{- end }}
              {{- with $values.resources }}
              resources:
                {{- toYaml . | nindent 16 }}
              {{- end }}
              {{- include "simpel.containerSecurityContext" $values | nindent 14 }}
              {{- if or $values.volumeMounts (and $values.secretonAuth $values.secretonAuth.enabled) $values.tmpVolumeSizeLimit }}
              volumeMounts:
                {{- with $values.volumeMounts }}
                {{- toYaml . | nindent 16 }}
                {{- end }}
                {{- if and $values.secretonAuth $values.secretonAuth.enabled }}
                - name: secreton-token
                  mountPath: /var/run/secrets/tokens
                  readOnly: true
                {{- end }}
                {{- if $values.tmpVolumeSizeLimit }}
                - name: tmp
                  mountPath: /tmp
                {{- end }}
              {{- end }}
          {{- if or $values.volumes (and $values.secretonAuth $values.secretonAuth.enabled) $values.tmpVolumeSizeLimit }}
          volumes:
            {{- with $values.volumes }}
            {{- toYaml . | nindent 12 }}
            {{- end }}
            {{- if and $values.secretonAuth $values.secretonAuth.enabled }}
            - name: secreton-token
              projected:
                sources:
                  - serviceAccountToken:
                      path: secreton-token
                      audience: {{ default "secreton" $values.secretonAuth.audience }}
                      expirationSeconds: {{ default 3600 $values.secretonAuth.tokenTTL }}
            {{- end }}
            {{- if $values.tmpVolumeSizeLimit }}
            - name: tmp
              emptyDir:
                sizeLimit: {{ $values.tmpVolumeSizeLimit }}
            {{- end }}
          {{- end }}
{{- end }}
{{- end -}}
