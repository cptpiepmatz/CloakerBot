def builder [] {
  let builders = which podman docker
  if ($builders | is-empty) {
    error make { msg: "Install Podman or Docker to build or run container images." }
  }
  $builders | get 0.path
}

def tag [] {
  let cargo_toml = open Cargo.toml
  let short_sha = ^git rev-parse --short HEAD

  let version = $cargo_toml.workspace.package.version + "-" + $short_sha
  return ("cloaker-bot:" + $version)
}

export def main [] {
  help main
}

export def "main build" [] {
  let tag = tag
  ^(builder) build --tag $tag .
}

export def "main run" [] {
  let builder = builder
  let tag = tag
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
    ^($builder) run --rm --init --tty --sig-proxy=true --publish 0.0.0.0:25625:25625 --volume $"($data_dir):/cloaker-bot/data" --env DISCORD_TOKEN --env DISCORD_CLIENT_SECRET --env COOKIE_SECRET $tag
  }
}
