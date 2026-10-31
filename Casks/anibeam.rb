cask "anibeam" do
  version "2.2.0"
  sha256 "12e3d7aa8e5367e72aa6fb3d09befda347b19e22c6ab76d64723b41bd4ac3274"

  url "https://github.com/marcusbandit/AniBeam/releases/download/v#{version}/AniBeam-#{version}-macos-arm64.dmg"
  name "AniBeam"
  desc "Browse, play and track a local anime library"
  homepage "https://github.com/marcusbandit/AniBeam"

  livecheck do
    url :url
    strategy :github_latest
  end

  # The app downloads and installs its own updates
  auto_updates true

  depends_on arch: :arm64
  depends_on macos: :sequoia

  app "AniBeam.app"

  zap trash: [
    "~/Library/Application Support/anibeam",
    "~/Library/Caches/anibeam",
  ]

  caveats <<~EOS
    AniBeam is not notarized, so macOS stops it the first time it opens. Go to
    System Settings > Privacy & Security, scroll to the AniBeam line and click Open Anyway.
  EOS
end
