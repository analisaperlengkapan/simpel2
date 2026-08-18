{{/*
  Workload helpers — Deployment, StatefulSet, dan associated Service.

  Pola pemanggilan utama (dari templates/services/<svc>.yaml):

    {{ include "simpel.workload" (dict "ctx" . "name" "authenc" "values" .Values.authenc) }}

  `values` punya struktur:
    enabled, image, replicas, kind (Deployment|StatefulSet),
    service: { ports: [{name, port, targetPort, protocol}] },
    env: [{name, value} | {name, valueFrom}],
    envFrom: [{configMapRef|secretRef}],
    volumes, volumeMounts,
    resources, podSecurityContext, containerSecurityContext,
    probes: { startup, liveness, readiness },
    topologySpreadConstraints, podAntiAffinity,
    nodeSelector, tolerations, affinity,
    initContainers, sidecars,
    istioInjection, istioProxyResources, istioExcludeOutboundPorts,
    serviceAccountName, secretonAuth: { enabled, role, audience, tokenPath }
*/}}

{{/* simpel.workload — render Service + (Deployment|StatefulSet) untuk satu component */}}
{{- define "simpel.workload" -}}
{{- $ctx := .ctx -}}
{{- $name := .name -}}
{{- $values := .values -}}
{{- if $values.enabled }}
{{- include "simpel.service" (dict "ctx" $ctx "name" $name "values" $values) }}
---
{{- $kind := default "Deployment" $values.kind }}
apiVersion: {{ if eq $kind "StatefulSet" }}apps/v1{{ else }}apps/v1{{ end }}
kind: {{ $kind }}
metadata:
  name: {{ $name }}
  namespace: {{ include "simpel.namespace" $ctx }}
  labels:
    {{- include "simpel.componentLabels" (dict "top" $ctx "name" $name "component" (default $name $values.component)) | nindent 4 }}
spec:
  {{- if eq $kind "Deployment" }}
  replicas: {{ default 1 $values.replicas }}
  strategy:
    {{- with $values.strategy }}
    {{- toYaml . | nindent 4 }}
    {{- else }}
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
    {{- end }}
  {{- else }}
  replicas: {{ default 1 $values.replicas }}
  serviceName: {{ default $name $values.service.headlessName }}
  # SAFETY: retain the data PVC if this StatefulSet is deleted/scaled down — data
  # must outlive the workload object (e.g. Secreton seal-state). Defense-in-depth
  # with the namespace `resource-policy: keep`.
  persistentVolumeClaimRetentionPolicy:
    whenDeleted: Retain
    whenScaled: Retain
  {{- with $values.updateStrategy }}
  updateStrategy:
    {{- toYaml . | nindent 4 }}
  {{- end }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "simpel.selectorLabels" (dict "top" $ctx "name" $name) | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "simpel.selectorLabels" (dict "top" $ctx "name" $name) | nindent 8 }}
        app.kubernetes.io/component: {{ default $name $values.component }}
        {{- with $values.podLabels }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
      annotations:
        {{- include "simpel.istioPodAnnotations" $values | nindent 8 }}
        {{- include "simpel.veleroExcludedVolumes" $values | nindent 8 }}
        {{- with $values.podAnnotations }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
    spec:
      serviceAccountName: {{ default $name $values.serviceAccountName }}
      automountServiceAccountToken: {{ default false $values.automountServiceAccountToken }}
      {{- include "simpel.imagePullSecrets" $ctx | nindent 6 }}
      {{- with default $ctx.Values.global.priorityClassName $values.priorityClassName }}
      priorityClassName: {{ . }}
      {{- end }}
      terminationGracePeriodSeconds: {{ default 30 $values.terminationGracePeriodSeconds }}
      {{- include "simpel.podSecurityContext" $values | nindent 6 }}
      {{- with default $ctx.Values.global.nodeSelector $values.nodeSelector }}
      nodeSelector:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with default $ctx.Values.global.tolerations $values.tolerations }}
      tolerations:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- $tsc := include "simpel.topologySpreadConstraints" (dict "name" $name "spec" $values.topologySpreadConstraints) }}
      {{- if $tsc }}
      {{- $tsc | nindent 6 }}
      {{- end }}
      {{- if or $values.affinity $values.podAntiAffinity }}
      affinity:
        {{- with $values.affinity }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
        {{- with $values.podAntiAffinity }}
        {{- include "simpel.podAntiAffinity" (dict "name" $name "type" .type) | nindent 8 }}
        {{- end }}
      {{- end }}
      {{- with $values.initContainers }}
      initContainers:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      containers:
        - name: {{ $name }}
          image: {{ include "simpel.image" (dict "top" $ctx "spec" $values.image) }}
          imagePullPolicy: {{ default (include "simpel.imagePullPolicy" $ctx) $values.image.pullPolicy }}
          {{- with $values.command }}
          command:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with $values.args }}
          args:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with $values.service.ports }}
          ports:
            {{- range . }}
            - name: {{ .name }}
              containerPort: {{ default .port .targetPort }}
              protocol: {{ default "TCP" .protocol }}
            {{- end }}
          {{- end }}
          {{- with $values.envFrom }}
          envFrom:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- if or $values.env (and $values.secretonAuth $values.secretonAuth.enabled) }}
          env:
            {{- with $values.env }}
            {{- toYaml . | nindent 12 }}
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
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- include "simpel.containerSecurityContext" $values | nindent 10 }}
          {{- include "simpel.probes" (dict "spec" (default (dict) $values.probes)) | nindent 10 }}
          {{- if or $values.volumeMounts (and $values.secretonAuth $values.secretonAuth.enabled) $values.tmpVolumeSizeLimit }}
          volumeMounts:
            {{- with $values.volumeMounts }}
            {{- toYaml . | nindent 12 }}
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
          {{- with $values.lifecycle }}
          lifecycle:
            {{- toYaml . | nindent 12 }}
          {{- end }}
        {{- with $values.sidecars }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
      {{- if or $values.volumes (and $values.secretonAuth $values.secretonAuth.enabled) $values.tmpVolumeSizeLimit }}
      volumes:
        {{- with $values.volumes }}
        {{- toYaml . | nindent 8 }}
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
  {{- if eq $kind "StatefulSet" }}
  {{- with $values.volumeClaimTemplates }}
  volumeClaimTemplates:
    {{- /*
      storageClassName DITURUNKAN dari `global.storageClass`, tidak ditulis
      ulang per-komponen. Entri boleh menimpanya secara eksplisit; bila absen
      atau kosong, kelas global yang berlaku.

      WHY: sebelumnya tiap komponen menulis literal `longhorn` di sini dan di
      `<komponen>.storage.storageClassName`, sehingga knob `global.storageClass`
      TAMPAK hidup tapi tak pernah menang — `default <global> <literal>`
      selalu mengembalikan literal-nya. Mengganti kelas penyimpanan klaster
      berarti mengedit 4 tempat, dan melewatkan satu berarti satu volume diam-diam
      tertinggal di kelas lama. Satu sumber, banyak pembaca.
    */}}
    {{- range . }}
    - metadata:
        {{- toYaml .metadata | nindent 8 }}
      spec:
        {{- toYaml (omit .spec "storageClassName") | nindent 8 }}
        storageClassName: {{ default $ctx.Values.global.storageClass .spec.storageClassName }}
    {{- end }}
  {{- end }}
  {{- end }}
{{- end }}
{{- end -}}

{{/* simpel.service — render Service untuk component (ClusterIP, optional headless) */}}
{{- define "simpel.service" -}}
{{- $ctx := .ctx -}}
{{- $name := .name -}}
{{- $values := .values -}}
{{- $svc := default (dict) $values.service -}}
{{- if not $svc.disabled }}
apiVersion: v1
kind: Service
metadata:
  name: {{ $name }}
  namespace: {{ include "simpel.namespace" $ctx }}
  labels:
    {{- include "simpel.componentLabels" (dict "top" $ctx "name" $name "component" (default $name $values.component)) | nindent 4 }}
  {{- with $svc.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
spec:
  type: {{ default "ClusterIP" $svc.type }}
  {{- if $svc.headless }}
  clusterIP: None
  {{- end }}
  selector:
    {{- include "simpel.selectorLabels" (dict "top" $ctx "name" $name) | nindent 4 }}
  ports:
    {{- range $svc.ports }}
    - name: {{ .name }}
      port: {{ .port }}
      targetPort: {{ default .port .targetPort }}
      protocol: {{ default "TCP" .protocol }}
      {{- with .appProtocol }}
      appProtocol: {{ . }}
      {{- end }}
    {{- end }}
{{- if $svc.headlessName }}
---
apiVersion: v1
kind: Service
metadata:
  name: {{ $svc.headlessName }}
  namespace: {{ include "simpel.namespace" $ctx }}
  labels:
    {{- include "simpel.componentLabels" (dict "top" $ctx "name" $name "component" (default $name $values.component)) | nindent 4 }}
spec:
  type: ClusterIP
  clusterIP: None
  publishNotReadyAddresses: true
  selector:
    {{- include "simpel.selectorLabels" (dict "top" $ctx "name" $name) | nindent 4 }}
  ports:
    {{- range $svc.ports }}
    - name: {{ .name }}
      port: {{ .port }}
      targetPort: {{ default .port .targetPort }}
      protocol: {{ default "TCP" .protocol }}
    {{- end }}
{{- end }}
{{- end }}
{{- end -}}

{{/*
  simpel.veleroIstioVolumes — volume scratch yang DISUNTIKKAN sidecar Istio.

  Tak bisa diturunkan dari values kita: yang membuatnya adalah injector, bukan
  template ini. Jadi namanya disebut eksplisit — itu kontrak Istio, bukan daftar
  milik kita yang bisa ikut tumbuh.
*/}}
{{- define "simpel.veleroIstioVolumes" -}}
istio-envoy,istio-data,istio-podinfo,istio-token,workload-certs,workload-socket,credential-socket
{{- end -}}

{{/*
  simpel.veleroExcludedVolumes — volume yang TIDAK ikut File System Backup.

  Velero berjalan `defaultVolumesToFsBackup: true` (opt-out), dan itu benar:
  volume baru otomatis ter-backup tanpa perlu diingat siapa pun. Konsekuensinya,
  volume yang TAK PERNAH berisi data ikut tersalin — scratch sidecar Istio, cache
  nginx, direktori soket.

  Latihan restore 2026-08-18 menunjukkan biayanya: dari 74 volume, 19 di antaranya
  sampah semacam itu; semuanya gagal disalin saat klaster sedang sibuk dan restore
  dilaporkan `PartiallyFailed` — padahal SELURUH volume data berhasil (checksum
  dbsimpelv2 identik dengan sumbernya). Sinyal merah yang tidak menandakan
  kehilangan data adalah sinyal yang akan diabaikan orang, dan itu jauh lebih
  mahal daripada 19 volume.

  Nama emptyDir DITURUNKAN dari `volumes` komponen + emptyDir yang dirender helper
  ini (`tmp`), jadi menambah emptyDir baru otomatis terkecuali. Workload yang
  merender pod-spec sendiri (postgres/redis/simpelv1) menulis daftarnya di berkas
  masing-masing, tepat di sebelah volume yang dinamainya.
*/}}
{{- define "simpel.veleroExcludedVolumes" -}}
{{- $names := list -}}
{{- range (default (list) .volumes) }}
  {{- if hasKey . "emptyDir" }}{{ $names = append $names .name }}{{ end }}
{{- end }}
{{- if .tmpVolumeSizeLimit }}{{ $names = append $names "tmp" }}{{ end }}
backup.velero.io/backup-volumes-excludes: {{ printf "%s,%s" (join "," (uniq $names)) (include "simpel.veleroIstioVolumes" .) | trimPrefix "," | quote }}
{{- end -}}
