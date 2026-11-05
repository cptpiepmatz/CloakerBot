def builder [] {
  let builders = which podman docker
  if ($builders | is-empty) {
    error make { msg: "Install Podman or Docker to build container images." }
  }
  $builders | get 0.path
}

def tag [platform: string] {
  let cargo_toml = open Cargo.toml
  let short_sha = ^git rev-parse --short HEAD

  let version = $cargo_toml.workspace.package.version + "-" + $short_sha
  return ("cloaker-bot/" + $platform + ":" + $version)
}

export def "build" [] {
  let tag = tag "amd64"
  ^(builder) build --tag $tag --platform linux/amd64 .
}

export def "build arm64" [] {
  let tag = tag "arm64"
  ^(builder) build --tag $tag --platform linux/arm64 .
}
