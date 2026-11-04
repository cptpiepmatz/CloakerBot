def builder [] {
  # errors if none is available
  which podman docker | get 0.command
}

def tag [platform: string] {
  let cargo_toml = open Cargo.toml
  let short_sha = ^git rev-parse --short HEAD

  let version = $cargo_toml.workspace.package.version + "-" + $short_sha
  return ("cloaker-bot/" + $platform + ":" + $version)
}

export def "docker" [] {
  let tag = tag "amd64"
  ^(builder) build --tag $tag --platform linux/amd64 .
}

export def "docker arm64" [] {
  let tag = tag "arm64"
  ^(builder) build --tag $tag --platform linux/arm64 .
}
