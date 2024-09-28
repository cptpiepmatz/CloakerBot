def tag [platform: string] {
  let cargo_toml = open Cargo.toml
  let short_sha = ^git rev-parse --short HEAD

  let version = $cargo_toml.package.version + "-" + $short_sha
  return ("cloaker-bot/" + $platform + ":" + $version)
}

export def "docker" [] {
  let tag = tag "amd64"
  ^docker build --tag $tag -f docker\amd64.Dockerfile .
}

export def "docker arm64" [] {
  let tag = tag "arm64"
  ^docker build --tag $tag --platform linux/arm64 -f docker\arm64.Dockerfile .
}
