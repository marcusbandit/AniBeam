cask "anibeam" do
  version "2.1.0"
  sha256 :no_check

  url "https://github.com/marcusbandit/AniBeam/releases/download/v#{version}/AniBeam-#{version}-macos-arm64.dmg"
  name "AniBeam"
  desc "Browse, play and track a local anime library"
  homepage "https://github.com/marcusbandit/AniBeam"

  livecheck do
    url :url
    strategy :github_latest
  end

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
