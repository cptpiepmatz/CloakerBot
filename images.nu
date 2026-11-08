def builder [] {
  let builders = which podman docker
  if ($builders | is-empty) {
    error make { msg: "Install Podman or Docker to build or run container images." }
  }
  $builders | get 0.path
}

def tag [platform: string] {
  let cargo_toml = open Cargo.toml
  let short_sha = ^git rev-parse --short HEAD

  let version = $cargo_toml.workspace.package.version + "-" + $short_sha
  return ("cloaker-bot/" + $platform + ":" + $version)
}

export def main [] {
  help main
}

export def "main build" [] {
  let tag = tag "amd64"
  ^(builder) build --tag $tag --platform linux/amd64 .
}

export def "main build arm64" [] {
  let tag = tag "arm64"
  ^(builder) build --tag $tag --platform linux/arm64 .
}

export def "main run" [platform: string = "amd64"] {
  if $platform not-in [amd64 arm64] {
    error make { msg: "Supported platforms are amd64 and arm64." }
  }

  let builder = builder
  let tag = tag $platform
  let data_dir = (pwd | path join data)
  mkdir $data_dir
  mut runtime_env = {}
  for name in [DISCORD_TOKEN DISCORD_CLIENT_SECRET COOKIE_SECRET] {
    mut value = ($env | get --optional $name | default "")
    while ($value | str trim | is-empty) {
      $value = (input --suppress-output $"($name): ")
      print ""
    }
    $runtime_env = ($runtime_env | insert $name $value)
  }

  with-env $runtime_env {
    ^($builder) run --rm --init --tty --sig-proxy=true --platform $"linux/($platform)" --publish 0.0.0.0:25625:25625 --volume $"($data_dir):/cloaker-bot/data" --env DISCORD_TOKEN --env DISCORD_CLIENT_SECRET --env COOKIE_SECRET $tag
  }
}
