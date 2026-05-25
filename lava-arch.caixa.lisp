(defcaixa
  :name
  "lava-arch"
  :kind
  :Biblioteca
  :ecosystem
  :rust-single-crate
  :package
  {:name "lava-arch"
   :version "0.1.0"
   :description "Composition layer for the lava suite. deflava-architecture form + Rust builders. Architectures consume + return typed ResourceRef chains; downstream architectures slot into upstream outputs at composition time (not apply time). Pangea Architecture/ResourceBuilder analog."
   :license "MIT"
   :repository "https://github.com/pleme-io/lava-arch"}
  :ci-config
  {:bump {:default-type "patch"}
   :publish {:no-verify true}}
  :workflows
  [:auto-release :pre-merge-gate :security-gate])
