use std::fs;

pub fn init_directory(path: &str) {
    let mut full_path = path.to_owned();
    full_path.push_str("/gitcomet");
    // create gitcomet folder
    fs::create_dir(full_path.clone()).expect("oh no it wasnt possible to create a directory");
    // create templates folder
    full_path.push_str("/templates");
    fs::create_dir(full_path.clone()).expect("oh no it wasnt possible to create a directory");
    // create argocd folder
    full_path.push_str("/argocd");
    fs::create_dir(full_path.clone()).expect("oh no it wasnt possible to create a directory");

    //create overlays folder
    let mut overlays_folder = full_path.clone();
    overlays_folder.push_str("/overlays");
    fs::create_dir(overlays_folder.clone()).expect("oh no it wasnt possible to create a directory");
    //create example helm_app.yaml
    overlays_folder.push_str("/helm_app.yaml");
    fs::write(overlays_folder, ARGOCD_OVERLAY_FILE).expect("couldn't write file");

    //create example schema file
    let mut schema_file = full_path.clone();
    schema_file.push_str("/schema.yaml");
    fs::write(schema_file, ARGOCD_JSONSCHEMA).expect("couldn't write file");

    //create example base file
    full_path.push_str("/base.yaml");
    fs::write(full_path.clone(), ARGOCD_BASE_FILE).expect("couldn't write file");
}

const ARGOCD_BASE_FILE: &str = r#"apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: default"#;

const ARGOCD_OVERLAY_FILE: &str = r#"metadata:
  name: guestbook
  labels:
    name: guestbook
spec:
  source:
    repoURL: "https://github.com/argoproj/argocd-example-apps.git"
    path: guestbook
    helm: 
      passCredentials: false # If true then adds --pass-credentials to Helm commands to pass credentials to all domains
      # Extra parameters to set (same as setting through values.yaml, but these take precedence)
      parameters:
      - name: "nginx-ingress.controller.service.annotations.external-dns\\.alpha\\.kubernetes\\.io/hostname"
        value: mydomain.example.com
      - name: "ingress.annotations.kubernetes\\.io/tls-acme"
        value: "true"
        forceString: true # ensures that value is treated as a string

      # Use the contents of files as parameters (uses Helm's --set-file)
      fileParameters:
      - name: config
        path: files/config.json

      # Release name override (defaults to application name)
      releaseName: guestbook

      # Helm values files for overriding values in the helm chart
      # The path is relative to the spec.source.path directory defined above
      valueFiles:
      - values-prod.yaml;

      # Ignore locally missing valueFiles when installing Helm chart. Defaults to false
      ignoreMissingValueFiles: false

      # Values file as block file. Prefer to use valuesObject if possible (see below)
      values: |
        ingress:
          enabled: true
          path: /
          hosts:
            - mydomain.example.com
          annotations:
            kubernetes.io/ingress.class: nginx
            kubernetes.io/tls-acme: "true"
          labels: {}
          tls:
            - secretName: mydomain-tls
              hosts:
                - mydomain.example.com

      # Values file as block file. This takes precedence over values
      valuesObject:
        ingress:
          enabled: true
          path: /
          hosts:
            - mydomain.example.com
          annotations:
            kubernetes.io/ingress.class: nginx
            kubernetes.io/tls-acme: "true"
          labels: {}
          tls:
            - secretName: mydomain-tls
              hosts:
                - mydomain.example.com"#;

const ARGOCD_JSONSCHEMA: &str = r##""$schema": http://json-schema.org/draft-06/schema#
"$ref": "#/definitions/ArgoCDApplication"
definitions:
  ArgoCDApplication:
    type: object
    additionalProperties: false
    properties:
      apiVersion:
        const: argoproj.io/v1alpha1
      kind:
        const: Application
      metadata:
        "$ref": "#/definitions/Metadata"
      spec:
        "$ref": "#/definitions/Spec"
    required:
    - apiVersion
    - kind
    - metadata
    - spec
    title: ArgoCDApplication
  Metadata:
    type: object
    additionalProperties: false
    properties:
      name:
        type: string
      namespace:
        type: string
        default: default
      finalizers:
        type: array
        items:
          type: string
      labels:
        "$ref": "#/definitions/MetadataLabels"
      annotations:
        "$ref": "#/definitions/MetadataLabels"
    required:
    - name
    title: Metadata
  MetadataLabels:
    type: object
    additionalProperties: false
    properties:
      name:
        type: string
    required:
    - name
    title: MetadataLabels
  Spec:
    type: object
    additionalProperties: false
    properties:
      project:
        type: string
      source:
        "$ref": "#/definitions/PurpleSource"
      sources:
        type: array
        items:
          "$ref": "#/definitions/SourceElement"
      destination:
        "$ref": "#/definitions/Destination"
      info:
        type: array
        items:
          "$ref": "#/definitions/Info"
      syncPolicy:
        "$ref": "#/definitions/SyncPolicy"
      ignoreDifferences:
        type: array
        items:
          "$ref": "#/definitions/IgnoreDifference"
      revisionHistoryLimit:
        type: integer
    required:
    - destination
    - project
    title: Spec
  Destination:
    type: object
    additionalProperties: false
    properties:
      server:
        type: string
        format: uri
        qt-uri-protocols:
        - https
      namespace:
        type: string
    required:
    - namespace
    - server
    title: Destination
  IgnoreDifference:
    type: object
    additionalProperties: false
    properties:
      group:
        type: string
      kind:
        type: string
      jsonPointers:
        type: array
        items:
          type: string
      jqPathExpressions:
        type: array
        items:
          type: string
      managedFieldsManagers:
        type: array
        items:
          type: string
      name:
        type: string
      namespace:
        type: string
    required:
    - kind
    title: IgnoreDifference
  Info:
    type: object
    additionalProperties: false
    properties:
      name:
        type: string
      value:
        type: string
        qt-uri-protocols:
        - https
    required:
    - name
    - value
    title: Info
  PurpleSource:
    type: object
    additionalProperties: false
    properties:
      repoURL:
        type: string
        format: uri
        qt-uri-protocols:
        - https
        qt-uri-extensions:
        - ".git"
      targetRevision:
        type: string
      path:
        type: string
      chart:
        type: string
      helm:
        "$ref": "#/definitions/Helm"
      kustomize:
        "$ref": "#/definitions/Kustomize"
      directory:
        "$ref": "#/definitions/Directory"
      plugin:
        "$ref": "#/definitions/Plugin"
    required:
    - repoURL
    - targetRevision
    title: PurpleSource
  Directory:
    type: object
    additionalProperties: false
    properties:
      recurse:
        type: boolean
      jsonnet:
        "$ref": "#/definitions/Jsonnet"
      exclude:
        type: string
      include:
        type: string
    required:
    - exclude
    - include
    - jsonnet
    - recurse
    title: Directory
  Jsonnet:
    type: object
    additionalProperties: false
    properties:
      extVars:
        type: array
        items:
          "$ref": "#/definitions/EXTVar"
      tlas:
        type: array
        items:
          "$ref": "#/definitions/EXTVar"
    required:
    - extVars
    - tlas
    title: Jsonnet
  EXTVar:
    type: object
    additionalProperties: false
    properties:
      name:
        type: string
      value:
        type: string
      code:
        type: boolean
    required:
    - name
    - value
    title: EXTVar
  Helm:
    type: object
    additionalProperties: false
    properties:
      passCredentials:
        type: boolean
      parameters:
        type: array
        items:
          "$ref": "#/definitions/HelmParameter"
      fileParameters:
        type: array
        items:
          "$ref": "#/definitions/FileParameter"
      releaseName:
        type: string
      valueFiles:
        type: array
        items:
          type: string
      ignoreMissingValueFiles:
        type: boolean
      values:
        type: string
      valuesObject:
        "$ref": "#/definitions/ValuesObject"
      skipCrds:
        type: boolean
      version:
        type: string
      kubeVersion:
        type: string
      apiVersions:
        type: array
        items:
          type: string
      namespace:
        type: string
    required:
    - namespace
    - version
    title: Helm
  FileParameter:
    type: object
    additionalProperties: false
    properties:
      name:
        type: string
      path:
        type: string
    required:
    - name
    - path
    title: FileParameter
  HelmParameter:
    type: object
    additionalProperties: false
    properties:
      name:
        type: string
      value:
        type: string
      forceString:
        type: boolean
    required:
    - name
    - value
    title: HelmParameter
  ValuesObject:
    type: object
    additionalProperties: false
    properties:
      ingress:
        "$ref": "#/definitions/Ingress"
    required:
    - ingress
    title: ValuesObject
  Ingress:
    type: object
    additionalProperties: false
    properties:
      enabled:
        type: boolean
      path:
        type: string
      hosts:
        type: array
        items:
          type: string
      annotations:
        "$ref": "#/definitions/IngressAnnotations"
      labels:
        "$ref": "#/definitions/IngressLabels"
      tls:
        type: array
        items:
          "$ref": "#/definitions/Tl"
    required:
    - annotations
    - enabled
    - hosts
    - labels
    - path
    - tls
    title: Ingress
  IngressAnnotations:
    type: object
    additionalProperties: false
    properties:
      kubernetes.io/ingress.class:
        type: string
      kubernetes.io/tls-acme:
        type: string
        format: boolean
    required:
    - kubernetes.io/ingress.class
    - kubernetes.io/tls-acme
    title: IngressAnnotations
  IngressLabels:
    type: object
    additionalProperties: false
    title: IngressLabels
  Tl:
    type: object
    additionalProperties: false
    properties:
      secretName:
        type: string
      hosts:
        type: array
        items:
          type: string
    required:
    - hosts
    - secretName
    title: Tl
  Kustomize:
    type: object
    additionalProperties: false
    properties:
      version:
        type: string
      namePrefix:
        type: string
      nameSuffix:
        type: string
      commonLabels:
        "$ref": "#/definitions/CommonLabels"
      commonAnnotations:
        "$ref": "#/definitions/CommonAnnotations"
      commonAnnotationsEnvsubst:
        type: boolean
      forceCommonLabels:
        type: boolean
      forceCommonAnnotations:
        type: boolean
      images:
        type: array
        items:
          type: string
      namespace:
        type: string
      replicas:
        type: array
        items:
          "$ref": "#/definitions/Replica"
      components:
        type: array
        items:
          type: string
      patches:
        type: array
        items:
          "$ref": "#/definitions/Patch"
      kubeVersion:
        type: string
      apiVersions:
        type: array
        items:
          type: string
    required:
    - apiVersions
    - commonAnnotations
    - commonAnnotationsEnvsubst
    - commonLabels
    - components
    - forceCommonAnnotations
    - forceCommonLabels
    - images
    - kubeVersion
    - namePrefix
    - nameSuffix
    - namespace
    - patches
    - replicas
    - version
    title: Kustomize
  CommonAnnotations:
    type: object
    additionalProperties: false
    properties:
      beep:
        type: string
    required:
    - beep
    title: CommonAnnotations
  CommonLabels:
    type: object
    additionalProperties: false
    properties:
      foo:
        type: string
    required:
    - foo
    title: CommonLabels
  Patch:
    type: object
    additionalProperties: false
    properties:
      target:
        "$ref": "#/definitions/Target"
      patch:
        type: string
    required:
    - patch
    - target
    title: Patch
  Target:
    type: object
    additionalProperties: false
    properties:
      kind:
        type: string
      name:
        type: string
    required:
    - kind
    - name
    title: Target
  Replica:
    type: objec
    additionalProperties: false
    properties:
      name:
        type: string
      count:
        type: integer
    required:
    - count
    - name
    title: Replica
  Plugin:
    type: object
    additionalProperties: false
    properties:
      name:
        type: string
      env:
        type: array
        items:
          "$ref": "#/definitions/Info"
      parameters:
        type: array
        items:
          "$ref": "#/definitions/PluginParameter"
    required:
    - env
    - name
    - parameters
    title: Plugin
  PluginParameter:
    type: object
    additionalProperties: false
    properties:
      name:
        type: string
      string:
        type: string
      array:
        type: array
        items:
          type: string
      map:
        "$ref": "#/definitions/Map"
    required:
    - name
    title: PluginParameter
  Map:
    type: object
    additionalProperties: false
    properties:
      param-name:
        type: string
    required:
    - param-name
    title: Map
  SourceElement:
    type: object
    additionalProperties: false
    properties:
      repoURL:
        type: string
        format: uri
        qt-uri-protocols:
        - https
        qt-uri-extensions:
        - ".git"
      targetRevision:
        type: string
      path:
        type: string
      ref:
        type: string
    required:
    - path
    - ref
    - repoURL
    - targetRevision
    title: SourceElement
  SyncPolicy:
    type: object
    additionalProperties: false
    properties:
      automated:
        "$ref": "#/definitions/Automated"
      syncOptions:
        type: array
        items:
          type: string
      managedNamespaceMetadata:
        "$ref": "#/definitions/ManagedNamespaceMetadata"
      retry:
        "$ref": "#/definitions/Retry"
    required:
    - automated
    - managedNamespaceMetadata
    - retry
    - syncOptions
    title: SyncPolicy
  Automated:
    type: object
    additionalProperties: false
    properties:
      prune:
        type: boolean
      selfHeal:
        type: boolean
      allowEmpty:
        type: boolean
    required:
    - allowEmpty
    - prune
    - selfHeal
    title: Automated
  ManagedNamespaceMetadata:
    type: object
    additionalProperties: false
    properties:
      labels:
        "$ref": "#/definitions/ManagedNamespaceMetadataLabels"
      annotations:
        "$ref": "#/definitions/ManagedNamespaceMetadataAnnotations"
    required:
    - annotations
    - labels
    title: ManagedNamespaceMetadata
  ManagedNamespaceMetadataAnnotations:
    type: object
    additionalProperties: false
    properties:
      the:
        type: string
      applies:
        type: string
      annotations:
        type: string
    required:
    - annotations
    - applies
    - the
    title: ManagedNamespaceMetadataAnnotations
  ManagedNamespaceMetadataLabels:
    type: object
    additionalProperties: false
    properties:
      any:
        type: string
      you:
        type: string
    required:
    - any
    - you
    title: ManagedNamespaceMetadataLabels
  Retry:
    type: object
    additionalProperties: false
    properties:
      limit:
        type: integer
      backoff:
        "$ref": "#/definitions/Backoff"
    required:
    - backoff
    - limit
    title: Retry
  Backoff:
    type: object
    additionalProperties: false
    properties:
      duration:
        type: string
      factor:
        type: integer
      maxDuration:
        type: string
    required:
    - duration
    - factor
    - maxDuration"##;
